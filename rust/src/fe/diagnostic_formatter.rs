use crate::fe::diagnostic::*;
use crate::fe::identifier::*;
use crate::fe::language::*;
use crate::fe::source_code_span::*;
use crate::i18n::translation::*;
use crate::qljs_assert;
use crate::qljs_translatable;
use crate::util::narrow_cast::*;

pub trait DiagnosticFormatter {
    fn write_before_message(
        &mut self,
        code: &str,
        severity: DiagnosticSeverity,
        origin: SourceCodeSpan<'_>,
    );
    fn write_message_part(&mut self, code: &str, severity: DiagnosticSeverity, message_part: &str);
    fn write_after_message(
        &mut self,
        code: &str,
        severity: DiagnosticSeverity,
        origin: SourceCodeSpan<'_>,
    );
    fn translator(&self) -> Translator;

    fn format(&mut self, info: &DiagnosticInfo, diagnostic: *const u8) {
        let code_string = info.code_string();
        let code_string_view: &str = unsafe { std::str::from_utf8_unchecked(&code_string) };

        self.format_message(
            code_string_view,
            info.severity(),
            info.message_formats[0],
            &info.message_args[0],
            diagnostic,
        );
        if info.message_formats[1].valid() {
            self.format_message(
                code_string_view,
                DiagnosticSeverity::Note,
                info.message_formats[1],
                &info.message_args[1],
                diagnostic,
            );
        }
    }

    fn format_message(
        &mut self,
        code: &str,
        severity: DiagnosticSeverity,
        message_format: TranslatableMessage,
        args: &DiagnosticMessageArgs,
        diagnostic: *const u8,
    ) {
        let origin_span: SourceCodeSpan =
            unsafe { get_argument_source_code_span(args, diagnostic, 0) };
        self.write_before_message(code, severity, origin_span);

        let mut remaining_message: &str = self.translator().translate(message_format);
        loop {
            let Some((before_left_curly, after_left_curly))
                = remaining_message.split_once('{') else {break;};
            qljs_assert!(
                !after_left_curly.is_empty(),
                "invalid message format: { at end of string has no matching }"
            );

            if after_left_curly.as_bytes()[0] == b'{' {
                // "{{"; the '{' is escaped.
                // TODO(port): Combine these two calls. Doing so would need split_once_inclusive, I
                // think.
                self.write_message_part(code, severity, before_left_curly);
                self.write_message_part(code, severity, "{");
                remaining_message = &after_left_curly[1..];
                continue;
            }

            self.write_message_part(code, severity, before_left_curly);

            let Some((curly_content, after_right_curly))
                = after_left_curly.split_once('}') else {
                    panic!("invalid message format: missing }}"); };

            let expanded_parameter: &str = unsafe {
                if curly_content == "0" {
                    expand_argument(args, diagnostic, 0)
                } else if curly_content == "1" {
                    expand_argument(args, diagnostic, 1)
                } else if curly_content == "1:headlinese" {
                    expand_argument_headlinese(self.translator(), args, diagnostic, 1)
                } else if curly_content == "1:singular" {
                    expand_argument_singular(self.translator(), args, diagnostic, 1)
                } else if curly_content == "2" {
                    expand_argument(args, diagnostic, 2)
                } else {
                    panic!("invalid message format: unrecognized placeholder");
                }
            };

            self.write_message_part(code, severity, expanded_parameter);
            remaining_message = after_right_curly;
        }
        self.write_message_part(code, severity, remaining_message);

        self.write_after_message(code, severity, origin_span);
    }
}

unsafe fn get_argument_source_code_span<'code>(
    args: &DiagnosticMessageArgs,
    diagnostic: *const u8,
    arg_index: i32,
) -> SourceCodeSpan<'code> {
    let (arg_data, arg_type) = get_arg(args, diagnostic, arg_index);
    match arg_type {
        DiagnosticArgType::Identifier => (&*(arg_data as *const Identifier)).span(),

        DiagnosticArgType::SourceCodeSpan => *(arg_data as *const SourceCodeSpan),

        DiagnosticArgType::Char8
        | DiagnosticArgType::EnumKind
        | DiagnosticArgType::Invalid
        | DiagnosticArgType::StatementKind
        | DiagnosticArgType::String8View
        | DiagnosticArgType::VariableKind => {
            unreachable!();
        }
    }
}

unsafe fn expand_argument<'diag>(
    args: &DiagnosticMessageArgs,
    diagnostic: *const u8,
    arg_index: i32,
) -> &'diag str {
    let (arg_data, arg_type) = get_arg(args, diagnostic, arg_index);
    match arg_type {
        DiagnosticArgType::Char8 => {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(arg_data, 1))
        }

        // FIXME(port): The following from_utf8_unchecked calls are wrong. The data might not be
        // UTF-8. We should perhaps change the interface of DiagnosticFormatter to emit u8s. Or
        // maybe we should write placeholders here on invalid UTF-8?
        DiagnosticArgType::Identifier => {
            std::str::from_utf8_unchecked((&*(arg_data as *const Identifier)).span().as_slice())
        }
        DiagnosticArgType::SourceCodeSpan => {
            std::str::from_utf8_unchecked((&*(arg_data as *const SourceCodeSpan)).as_slice())
        }
        DiagnosticArgType::String8View => {
            std::str::from_utf8_unchecked(*(arg_data as *const &[u8]))
        }

        DiagnosticArgType::EnumKind
        | DiagnosticArgType::Invalid
        | DiagnosticArgType::StatementKind
        | DiagnosticArgType::VariableKind => {
            unreachable!();
        }
    }
}

fn expand_argument_headlinese(
    translator: Translator,
    args: &DiagnosticMessageArgs,
    diagnostic: *const u8,
    arg_index: i32,
) -> &'static str {
    let (arg_data, arg_type) = get_arg(args, diagnostic, arg_index);
    match arg_type {
        DiagnosticArgType::EnumKind => {
            headlinese_enum_kind(unsafe { *(arg_data as *const EnumKind) })
        }

        DiagnosticArgType::StatementKind => {
            translator.translate(headlinese_statement_kind(unsafe {
                *(arg_data as *const StatementKind)
            }))
        }

        DiagnosticArgType::Char8
        | DiagnosticArgType::Identifier
        | DiagnosticArgType::Invalid
        | DiagnosticArgType::SourceCodeSpan
        | DiagnosticArgType::String8View
        | DiagnosticArgType::VariableKind => {
            unreachable!();
        }
    }
}

fn expand_argument_singular(
    translator: Translator,
    args: &DiagnosticMessageArgs,
    diagnostic: *const u8,
    arg_index: i32,
) -> &'static str {
    let (arg_data, arg_type) = get_arg(args, diagnostic, arg_index);
    match arg_type {
        DiagnosticArgType::StatementKind => translator.translate(singular_statement_kind(unsafe {
            *(arg_data as *const StatementKind)
        })),

        DiagnosticArgType::EnumKind => {
            unimplemented!();
        }

        DiagnosticArgType::Char8
        | DiagnosticArgType::Identifier
        | DiagnosticArgType::Invalid
        | DiagnosticArgType::SourceCodeSpan
        | DiagnosticArgType::String8View
        | DiagnosticArgType::VariableKind => {
            unreachable!();
        }
    }
}

fn get_arg(
    args: &DiagnosticMessageArgs,
    diagnostic: *const u8,
    arg_index: i32,
) -> (*const u8, DiagnosticArgType) {
    let arg_info: &DiagnosticMessageArgInfo = &args[narrow_cast::<usize, _>(arg_index)];
    let arg_data: *const u8 = unsafe { diagnostic.offset(arg_info.offset() as isize) };
    (arg_data, arg_info.type_())
}

fn headlinese_enum_kind(ek: EnumKind) -> &'static str {
    match ek {
        EnumKind::ConstEnum => "const enum",
        EnumKind::DeclareConstEnum => "declare const enum",
        EnumKind::DeclareEnum => "declare enum",
        EnumKind::Normal => "enum",
    }
}

fn headlinese_statement_kind(sk: StatementKind) -> TranslatableMessage {
    match sk {
        StatementKind::DoWhileLoop => qljs_translatable!("'do-while' loop"),
        StatementKind::ForLoop => qljs_translatable!("'for' loop"),
        StatementKind::IfStatement => qljs_translatable!("'if' statement"),
        StatementKind::WhileLoop => qljs_translatable!("'while' loop"),
        StatementKind::WithStatement => qljs_translatable!("'with' statement"),
        StatementKind::LabelledStatement => qljs_translatable!("labelled statement"),
    }
}

fn singular_statement_kind(sk: StatementKind) -> TranslatableMessage {
    match sk {
        StatementKind::DoWhileLoop => qljs_translatable!("a 'do-while' loop"),
        StatementKind::ForLoop => qljs_translatable!("a 'for' loop"),
        StatementKind::IfStatement => qljs_translatable!("an 'if' statement"),
        StatementKind::WhileLoop => qljs_translatable!("a 'while' loop"),
        StatementKind::WithStatement => qljs_translatable!("a 'with' statement"),
        StatementKind::LabelledStatement => qljs_translatable!("a labelled statement"),
    }
}