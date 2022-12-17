// Clippy's suggested fix is ugly.
#![allow(clippy::explicit_counter_loop)]
// Often we write (x << 0) or (x | 0) for symmetry with other code.
#![allow(clippy::identity_op)]
// Often we write lifetimes explicitly for better readability.
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::redundant_static_lifetimes)]
// Refactoring is easier if the shorthand syntax is avoided.
#![allow(clippy::redundant_field_names)]

mod token_stream_parser;
mod token_writer;

use token_stream_parser::*;
use token_writer::*;

static mut REGISTERED_DIAG_STRUCTS: Vec<RegisteredDiag> = vec![];

const DIAGNOSTIC_MAX_MESSAGE_COUNT: usize = 2;
const DIAGNOSTIC_MAX_ARG_COUNT: usize = 3;

// Uses of qljs_diagnostic should have the following signature:
//
// #[qljs_diagnostic(error_code, severity, message_0)]
// struct DiagName { ... }
//
// or
//
// #[qljs_diagnostic(error_code, severity, message_0, message_1)]
// struct DiagName { ... }
//
// * error_code: string literal (e.g. "E0001")
// * severity: DiagnosticSeverity value (e.g. DiagnosticSeverity::Error)
// * message_0, message_1: parenthesized format (see below)
//
// A format (*message_0* or *message_1*) should look like the following:
//
//    (qljs_translatable!("format string"), source_location)
//
// Within a format:
//
// * The tuple's first member must be qljs_translatable!(...)
// * The tuple's second argument must be a field of the attributed struct
//   (without "self.")
// * The tuple's second argument must have type *Identifier* or *SourceCodeSpan*
//
// Adding the qljs_diagnostic attribute will automatically derive Clone.
//
// Example:
//
// #[qljs_diagnostic(
//     "E0005",
//     DiagnosticSeverity::Error,
//     (qljs_translatable!("BigInt literal contains decimal point"), where_),
// )]
// struct DiagBigIntLiteralContainsDecimalPoint<'code> {
//   where_: SourceCodeSpan<'code>,
// }
#[proc_macro_attribute]
pub fn qljs_diagnostic(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut parser = TokenStreamParser::new(item.clone());
    parser.try_parse_keyword("pub");
    parser.skip_keyword("struct");
    let struct_name: proc_macro::Ident = parser.try_parse_ident().expect("expected struct name");

    if parser
        .try_parse_punct_token('<', proc_macro::Spacing::Alone)
        .is_some()
    {
        parser.skip_lifetime();
        parser.skip_punct(">");
    }

    let struct_body: proc_macro::TokenStream =
        parser.try_parse_brace().expect("expected struct body");
    let mut struct_body_parser = TokenStreamParser::new(struct_body);

    let mut fields: Vec<QLJSDiagnosticField> = vec![];
    while !struct_body_parser.is_eof() {
        struct_body_parser.try_parse_keyword("pub");
        let field_name: proc_macro::Ident = struct_body_parser
            .try_parse_ident()
            .expect("expected field name");
        struct_body_parser.skip_punct(":");
        let arg_type: DiagnosticArgType = parse_arg_type(&mut struct_body_parser);
        fields.push(QLJSDiagnosticField {
            name: field_name.to_string(),
            type_: arg_type,
        });
        if struct_body_parser
            .try_parse_punct_token(',', proc_macro::Spacing::Alone)
            .is_none()
        {
            break;
        }
    }
    struct_body_parser.expect_eof();
    parser.expect_eof();

    unsafe {
        REGISTERED_DIAG_STRUCTS.push(RegisteredDiag {
            name: struct_name.to_string(),
            fields: fields,
            attribute: parse_qljs_diagnostic_attribute(attr),
        })
    };

    let mut derive = TokenWriter::new();
    derive.derive_attribute(&[
        "Clone", "Debug", // TODO(strager): Instead, only implement Debug on AnyDiag.
    ]);

    let mut tokens: proc_macro::TokenStream = derive.to_token_stream();
    tokens.extend([item]);
    tokens
}

fn parse_qljs_diagnostic_attribute(stream: proc_macro::TokenStream) -> QLJSDiagnosticAttribute {
    let mut parser = TokenStreamParser::new(stream);
    let code: String = parser
        .try_parse_string()
        .expect("expected error code string as first argument");
    parser.skip_comma();
    parser.skip_keyword("DiagnosticSeverity");
    parser.skip_punct("::");
    let ident: proc_macro::Ident = parser
        .try_parse_ident()
        .expect("expected error severity as second argument");
    parser.skip_comma();

    let mut messages = vec![];
    loop {
        match parser.try_parse_paren() {
            Some(message_stream) => {
                let mut message_parser = TokenStreamParser::new(message_stream);
                message_parser.skip_keyword("qljs_translatable");
                message_parser.skip_punct("!");

                let arguments: proc_macro::TokenStream = message_parser
                    .try_parse_paren()
                    .expect("expected argument for qljs_translatable");
                let mut arguments_parser = TokenStreamParser::new(arguments);
                let format: String = arguments_parser
                    .try_parse_string()
                    .expect("expected string argument for qljs_translatable");
                arguments_parser.expect_eof();

                let mut fields: Vec<String> = vec![];
                while message_parser.try_parse_comma().is_some() {
                    fields.push(
                        message_parser
                            .try_parse_ident()
                            .expect("expected field name")
                            .to_string(),
                    );
                }
                message_parser.expect_eof();

                messages.push(QLJSDiagnosticAttributeMessage {
                    format: format,
                    fields: fields,
                });
            }
            None => {
                parser.expect_eof();
                break;
            }
        }

        if parser.try_parse_comma().is_none() {
            break;
        }
    }
    parser.expect_eof();

    QLJSDiagnosticAttribute {
        code_string: code,
        diagnostic_severity: ident.to_string(),
        messages: messages,
    }
}

fn parse_arg_type(parser: &mut TokenStreamParser) -> DiagnosticArgType {
    let skip_generic_args = |parser: &mut TokenStreamParser| {
        if parser
            .try_parse_punct_token('<', proc_macro::Spacing::Alone)
            .is_some()
        {
            parser.skip_lifetime();
            if parser
                .try_parse_punct_token('>', proc_macro::Spacing::Alone)
                .is_none()
            {
                // HACK(strager): For some reason, '>' and the following ',' are often fused into
                // one token.
                parser.skip_punct_token('>', proc_macro::Spacing::Joint);
            }
        }
    };

    if parser.try_parse_keyword("SourceCodeSpan").is_some() {
        skip_generic_args(parser);
        return DiagnosticArgType::SourceCodeSpan;
    }

    if parser.try_parse_keyword("u8").is_some() {
        return DiagnosticArgType::Char8;
    }

    // &'code [u8]
    if parser
        .try_parse_punct_token('&', proc_macro::Spacing::Alone)
        .is_some()
    {
        parser.skip_lifetime();
        let _slice = parser
            .try_parse_bracket()
            .expect("expected slice in field type");
        return DiagnosticArgType::String8View;
    }

    panic!(
        "unexpected field type: {}",
        parser.current.as_ref().unwrap()
    );
}

// Write:
//
// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// #[repr(u16)]
// pub enum $name {
//     Diag1,
//     Diag2,
//     /* ... */
// }
#[proc_macro]
pub fn qljs_make_diag_type_enum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut parser = TokenStreamParser::new(item);
    let name: proc_macro::Ident = parser
        .try_parse_ident()
        .expect("expected parameter: enum name");
    parser.expect_eof();

    let mut enum_writer = TokenWriter::new();
    enum_writer.derive_attribute(&["Clone", "Copy", "Debug", "Eq", "PartialEq"]);
    enum_writer.punct("#");
    enum_writer.build_bracket(|attribute: &mut TokenWriter| {
        attribute.ident("repr");
        attribute.build_paren(|repr: &mut TokenWriter| {
            repr.ident("u16");
        });
    });
    enum_writer.ident("pub");
    enum_writer.ident("enum");
    enum_writer.token(proc_macro::TokenTree::Ident(name));
    enum_writer.build_brace(|enum_members: &mut TokenWriter| {
        for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
            enum_members.ident(&diag_struct.name);
            enum_members.punct(",");
        }
    });
    enum_writer.to_token_stream()
}

// Write:
//
// #[derive(Clone, Debug)]
// pub enum AnyDiag<'code> {
//     Diag1(Diag1<'code>),
//     Diag2(Diag2<'code>),
//     /* ... */
// }
//
// impl<'code> AnyDiag<'code> {
//     pub unsafe fn from_raw_parts(type_: DiagType, diag: *const u8) -> Self {
//         match type_ {
//             DiagType::Diag1 => AnyDiag::Diag1((&*(diag as *const Diag1)).clone()),
//             DiagType::Diag2 => AnyDiag::Diag2((&*(diag as *const Diag2)).clone()),
//             /* ... */
//         }
//     }
// }
#[proc_macro]
pub fn qljs_make_any_diag_enum(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut writer = TokenWriter::new();

    writer.derive_attribute(&[
        "Clone",
        // TODO(strager): Instead, debug print without the enum name, to avoid duplicating the diag
        // struct name.
        "Debug",
    ]);
    writer.ident("pub");
    writer.ident("enum");
    writer.ident("AnyDiag");
    writer.punct("<");
    writer.lifetime("code");
    writer.punct(">");
    writer.build_brace(|enum_members: &mut TokenWriter| {
        for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
            enum_members.ident(&diag_struct.name);
            enum_members.build_paren(|member: &mut TokenWriter| {
                member.ident(&diag_struct.name);
                member.punct("<");
                member.lifetime("code");
                member.punct(">");
            });
            enum_members.punct(",");
        }
    });

    writer.ident("impl");
    writer.punct("<");
    writer.lifetime("code");
    writer.punct(">");
    writer.ident("AnyDiag");
    writer.punct("<");
    writer.lifetime("code");
    writer.punct(">");
    writer.build_brace(|impl_body: &mut TokenWriter| {
        impl_body.ident("pub");
        impl_body.ident("unsafe");
        impl_body.ident("fn");
        impl_body.ident("from_raw_parts");
        impl_body.build_paren(|parameters: &mut TokenWriter| {
            parameters.ident("type_");
            parameters.punct(":");
            parameters.ident("DiagType");
            parameters.punct(",");
            parameters.ident("diag");
            parameters.punct(":");
            parameters.punct("*");
            parameters.ident("const");
            parameters.ident("u8");
        });
        impl_body.punct("->");
        impl_body.ident("Self");
        impl_body.build_brace(|fn_body: &mut TokenWriter| {
            fn_body.ident("match");
            fn_body.ident("type_");
            fn_body.build_brace(|match_body: &mut TokenWriter| {
                for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
                    match_body.ident("DiagType");
                    match_body.punct("::");
                    match_body.ident(&diag_struct.name);
                    match_body.punct("=>");
                    match_body.ident("AnyDiag");
                    match_body.punct("::");
                    match_body.ident(&diag_struct.name);
                    match_body.build_paren(|construct_args: &mut TokenWriter| {
                        construct_args.build_paren(|diag: &mut TokenWriter| {
                            diag.punct("&");
                            diag.punct("*");
                            diag.build_paren(|diag_ptr: &mut TokenWriter| {
                                diag_ptr.ident("diag");
                                diag_ptr.ident("as");
                                diag_ptr.punct("*");
                                diag_ptr.ident("const");
                                diag_ptr.ident(&diag_struct.name);
                            });
                        });
                        construct_args.punct(".");
                        construct_args.ident("clone");
                        construct_args.empty_paren();
                    });
                    match_body.punct(",");
                }
            });
        });
    });

    writer.to_token_stream()
}

// For each registered diagnostic struct, write the following:
//
// impl<'code> HasDiagType for $diag<'code> {
//     const TYPE_: DiagType = DiagType::$diag;
// }
#[proc_macro]
pub fn qljs_make_has_diag_type_impls(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut writer = TokenWriter::new();
    for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
        writer.ident("impl");
        writer.punct("<");
        writer.lifetime("code");
        writer.punct(">");
        writer.ident("HasDiagType");
        writer.ident("for");
        writer.ident(&diag_struct.name);
        writer.punct("<");
        writer.lifetime("code");
        writer.punct(">");
        writer.build_brace(|impl_members: &mut TokenWriter| {
            impl_members.ident("const");
            impl_members.ident("TYPE_");
            impl_members.punct(":");
            impl_members.ident("DiagType");
            impl_members.punct("=");
            impl_members.ident("DiagType");
            impl_members.punct("::");
            impl_members.ident(&diag_struct.name);
            impl_members.punct(";");
        });
    }

    writer.to_token_stream()
}

#[proc_macro]
pub fn qljs_diag_type_count(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStreamParser::new(item).expect_eof();

    let mut size = TokenWriter::new();
    size.literal_usize(unsafe { REGISTERED_DIAG_STRUCTS.len() });
    size.to_token_stream()
}

// Write:
//
// pub const DIAG_SIZES: [u8; $diag_count] = [
//     std::mem::size_of::<Diag1>() as u8,
//     std::mem::size_of::<Diag2>() as u8,
//     /* ... */
// ];
#[proc_macro]
pub fn qljs_diag_sizes_array(args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStreamParser::new(args).expect_eof();

    let mut writer = TokenWriter::new();
    writer.ident("pub");
    writer.ident("const");
    writer.ident("DIAG_SIZES");
    writer.punct(":");
    writer.build_bracket(|array_type: &mut TokenWriter| {
        array_type.ident("u8");
        array_type.punct(";");
        array_type.literal_usize(unsafe { REGISTERED_DIAG_STRUCTS.len() });
    });
    writer.punct("=");
    writer.build_bracket(|array: &mut TokenWriter| {
        for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
            array.ident("std");
            array.punct("::");
            array.ident("mem");
            array.punct("::");
            array.ident("size_of");
            array.punct("::");
            array.punct("<");
            array.ident(&diag_struct.name);
            array.punct(">");
            array.empty_paren();
            array.ident("as");
            array.ident("u8");
            array.punct(",");
        }
    });
    writer.punct(";");
    writer.to_token_stream()
}

#[proc_macro]
pub fn qljs_make_diag_type_infos(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TokenStreamParser::new(item).expect_eof();

    let mut infos = TokenWriter::new();
    for diag_struct in unsafe { &REGISTERED_DIAG_STRUCTS } {
        write_diagnostic_message_new(
            &mut infos,
            diag_struct.attribute.code(),
            &diag_struct.attribute.diagnostic_severity,
            // message_formats
            |formats: &mut TokenWriter| {
                for message in &diag_struct.attribute.messages {
                    write_qljs_translatable_call(formats, &message.format);
                    formats.punct(",");
                }
                for _ in diag_struct.attribute.messages.len()..DIAGNOSTIC_MAX_MESSAGE_COUNT {
                    write_translatable_message_unallocated(formats);
                    formats.punct(",");
                }
            },
            // message_args
            |arg_infos: &mut TokenWriter| {
                let write_filler_entries = |out: &mut TokenWriter, count: usize| {
                    for _ in 0..count {
                        write_diagnostic_message_arg_info_empty(out);
                        out.punct(",");
                    }
                };
                for message in &diag_struct.attribute.messages {
                    arg_infos.build_bracket(|arg_info: &mut TokenWriter| {
                        for field in &message.fields {
                            write_diagnostic_message_arg_info_new(arg_info, diag_struct, field);
                            arg_info.punct(",");
                        }
                        write_filler_entries(
                            arg_info,
                            DIAGNOSTIC_MAX_ARG_COUNT - message.fields.len(),
                        );
                    });
                    arg_infos.punct(",");
                }
                for _ in diag_struct.attribute.messages.len()..DIAGNOSTIC_MAX_MESSAGE_COUNT {
                    arg_infos.build_bracket(|arg_info: &mut TokenWriter| {
                        write_filler_entries(arg_info, DIAGNOSTIC_MAX_ARG_COUNT);
                    });
                    arg_infos.punct(",");
                }
            },
        );
        infos.punct(",");
    }

    let mut infos_array = TokenWriter::new();
    infos_array.bracket(infos);
    infos_array.to_token_stream()
}

struct RegisteredDiag {
    name: String,
    fields: Vec<QLJSDiagnosticField>,
    attribute: QLJSDiagnosticAttribute,
}

struct QLJSDiagnosticField {
    name: String,
    type_: DiagnosticArgType,
}

#[allow(dead_code)] // TODO(port)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiagnosticArgType {
    Char8,          // u8
    EnumKind,       // EnumKind
    Identifier,     // Identifier<'code>
    SourceCodeSpan, // SourceCodeSpan<'code>
    StatementKind,  // StatementKind
    String8View,    // &'code [u8]
    VariableKind,   // VariableKind
}

impl RegisteredDiag {
    fn arg_type_string_for_field(&self, field_name: &str) -> &'static str {
        for field in &self.fields {
            if field.name == field_name {
                return get_diagnostic_message_arg_type(field.type_);
            }
        }
        panic!("could not find field {}::{}", self.name, field_name);
    }
}

struct QLJSDiagnosticAttribute {
    code_string: String,
    diagnostic_severity: String, // "Error", "Warning", or "Note"
    messages: Vec<QLJSDiagnosticAttributeMessage>,
}

impl QLJSDiagnosticAttribute {
    // NOTE(port): This function was called parse_code_string in diagnostic.cpp.
    fn code(&self) -> u16 {
        let code_string: &[u8] = self.code_string.as_bytes();
        assert_eq!(code_string.len(), 5);
        assert_eq!(code_string[0], b'E');
        let zero: u8 = b'0';
        assert!(zero <= code_string[1] && code_string[1] <= b'9');
        assert!(zero <= code_string[2] && code_string[2] <= b'9');
        assert!(zero <= code_string[3] && code_string[3] <= b'9');
        assert!(zero <= code_string[4] && code_string[4] <= b'9');
        ((code_string[1] - zero) as u16) * 1000
            + ((code_string[2] - zero) as u16) * 100
            + ((code_string[3] - zero) as u16) * 10
            + ((code_string[4] - zero) as u16) * 1
    }
}

struct QLJSDiagnosticAttributeMessage {
    format: String, // qljs_translatable! removed.
    fields: Vec<String>,
}

// NOTE(port): This was get_diagnostic_message_arg_type from diagnostic.h.
fn get_diagnostic_message_arg_type(field_type: DiagnosticArgType) -> &'static str {
    match field_type {
        DiagnosticArgType::Char8 => "Char8",
        DiagnosticArgType::EnumKind => "EnumKind",
        DiagnosticArgType::Identifier => "Identifier",
        DiagnosticArgType::SourceCodeSpan => "SourceCodeSpan",
        DiagnosticArgType::StatementKind => "StatementKind",
        DiagnosticArgType::String8View => "String8View",
        DiagnosticArgType::VariableKind => "VariableKind",
    }
}

// Write:
//
// DiagnosticMessage::new(
//   $code,
//   $severity,
//   $message_formats,
//   $message_args,
// )
fn write_diagnostic_message_new<
    FormatsBuilder: FnOnce(&mut TokenWriter),
    ArgsBuilder: FnOnce(&mut TokenWriter),
>(
    out: &mut TokenWriter,
    code: u16,
    severity: &str,
    message_formats: FormatsBuilder,
    message_args: ArgsBuilder,
) {
    out.ident("DiagnosticInfo");
    out.punct("::");
    out.ident("new");

    out.build_paren(|args: &mut TokenWriter| {
        // code
        args.literal_u16(code);
        args.punct(",");

        // severity
        args.ident("DiagnosticSeverity");
        args.punct("::");
        args.ident(severity);
        args.punct(",");

        // message_formats
        args.build_bracket(message_formats);
        args.punct(",");

        // message_args
        args.build_bracket(message_args);
    });
}

// Write:
//
// DiagnosticMessageArgInfo::empty()
fn write_diagnostic_message_arg_info_empty(out: &mut TokenWriter) {
    out.ident("DiagnosticMessageArgInfo");
    out.punct("::");
    out.ident("empty");
    out.empty_paren();
}

// Write:
//
// DiagnosticMessageArgInfo::new(qljs_offset_of!($name, $field), DiagnosticArgTypes::$type)
fn write_diagnostic_message_arg_info_new(
    out: &mut TokenWriter,
    diag_struct: &RegisteredDiag,
    field_name: &str,
) {
    out.ident("DiagnosticMessageArgInfo");
    out.punct("::");
    out.ident("new");
    out.build_paren(|args: &mut TokenWriter| {
        write_offset_of_call(args, &diag_struct.name, field_name);
        args.punct(",");

        args.ident("DiagnosticArgType");
        args.punct("::");
        args.ident(diag_struct.arg_type_string_for_field(field_name));
    });
}

// Write:
//
// qljs_offset_of!($name, $field)
fn write_offset_of_call(out: &mut TokenWriter, name: &str, field: &str) {
    out.ident("qljs_offset_of");
    out.punct("!");
    out.build_paren(|args: &mut TokenWriter| {
        args.ident(name);
        args.punct(",");
        args.ident(field);
    });
}

// Write:
//
// qljs_translatable!($message)
fn write_qljs_translatable_call(out: &mut TokenWriter, message: &str) {
    out.ident("qljs_translatable");
    out.punct("!");
    out.build_paren(|args: &mut TokenWriter| {
        args.string(message);
    });
}

// Write:
//
// TranslatableMessage::unallocated()
fn write_translatable_message_unallocated(out: &mut TokenWriter) {
    out.ident("TranslatableMessage");
    out.punct("::");
    out.ident("unallocated");
    out.empty_paren();
}
