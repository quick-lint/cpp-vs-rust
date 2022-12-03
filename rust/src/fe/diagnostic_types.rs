use crate::fe::diagnostic::*;
use crate::fe::source_code_span::*;
use crate::i18n::translation::*;
use crate::qljs_translatable;
use cpp_vs_rust_proc_diagnostic_types::*;

macro_rules! qljs_offset_of {
    ($type:ty, $field:tt $(,)?) => {
        unsafe {
            let temp: std::mem::MaybeUninit<$type> = std::mem::MaybeUninit::uninit();
            let base_ptr = temp.assume_init_ref() as *const _ as *const u8;
            let field_ptr = &temp.assume_init_ref().$field as *const _ as *const u8;
            (field_ptr.offset_from(base_ptr)) as usize
        }
    };
}

#[qljs_diagnostic(
    "E0005", DiagnosticSeverity::Error,
    (qljs_translatable!("BigInt literal contains decimal point"), where_),
)]
pub struct DiagBigIntLiteralContainsDecimalPoint<'code> {
    pub where_: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0006", DiagnosticSeverity::Error,
    (qljs_translatable!("BigInt literal contains exponent"), where_),
)]
pub struct DiagBigIntLiteralContainsExponent<'code> {
    pub where_: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0011", DiagnosticSeverity::Error,
    (qljs_translatable!("character is not allowed in identifiers"), character),
)]
pub struct DiagCharacterDisallowedInIdentifiers<'code> {
    pub character: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0012", DiagnosticSeverity::Error,
    (qljs_translatable!("escaped character is not allowed in identifiers"), escape_sequence),
)]
pub struct DiagEscapedCharacterDisallowedInIdentifiers<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0013", DiagnosticSeverity::Error,
    (qljs_translatable!("code point out of range"), escape_sequence),
)]
pub struct DiagEscapedCodePointInIdentifierOutOfRange<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0207", DiagnosticSeverity::Error,
    (qljs_translatable!("code point in Unicode escape sequence must not be greater than U+10FFFF"), escape_sequence),
)]
pub struct DiagEscapedCodePointInUnicodeOutOfRange<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0016", DiagnosticSeverity::Error,
    (qljs_translatable!("expected hexadecimal digits in Unicode escape sequence"), escape_sequence),
)]
pub struct DiagExpectedHexDigitsInUnicodeEscape<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0060", DiagnosticSeverity::Error,
    (qljs_translatable!("invalid hex escape sequence: {0}"), escape_sequence),
)]
pub struct DiagInvalidHexEscapeSequence<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0197", DiagnosticSeverity::Error,
    (qljs_translatable!("'{0}' is not allowed for strings; use {1} instead"), opening_quote, suggested_quote),
)]
pub struct DiagInvalidQuotesAroundStringLiteral<'code> {
    pub opening_quote: SourceCodeSpan<'code>,
    pub suggested_quote: u8,
}

#[qljs_diagnostic(
    "E0022", DiagnosticSeverity::Error,
    (qljs_translatable!("invalid UTF-8 sequence"), sequence),
)]
pub struct DiagInvalidUTF8Sequence<'code> {
    pub sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0023", DiagnosticSeverity::Error,
    (qljs_translatable!("keywords cannot contain escape sequences"), escape_sequence),
)]
pub struct DiagKeywordsCannotContainEscapeSequences<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0032", DiagnosticSeverity::Error,
    (qljs_translatable!("legacy octal literal may not be BigInt"), characters),
)]
pub struct DiagLegacyOctalLiteralMayNotBeBigInt<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0152", DiagnosticSeverity::Error,
    (qljs_translatable!("legacy octal literals may not contain underscores"), underscores),
)]
pub struct DiagLegacyOctalLiteralMayNotContainUnderscores<'code> {
    pub underscores: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0028", DiagnosticSeverity::Error,
    (qljs_translatable!("number literal contains consecutive underscores"), underscores),
)]
pub struct DiagNumberLiteralContainsConsecutiveUnderscores<'code> {
    pub underscores: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0030", DiagnosticSeverity::Error,
    (qljs_translatable!("octal literal may not have exponent"), characters),
)]
pub struct DiagOctalLiteralMayNotHaveExponent<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0031", DiagnosticSeverity::Error,
    (qljs_translatable!("octal literal may not have decimal"), characters),
)]
pub struct DiagOctalLiteralMayNotHaveDecimal<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0212", DiagnosticSeverity::Warning,
    (qljs_translatable!("integer cannot be represented and will be rounded to '{1}'"), characters, rounded_val),
)]
pub struct DiagIntegerLiteralWillLosePrecision<'code> {
    pub characters: SourceCodeSpan<'code>,
    pub rounded_val: &'code [u8],
}

#[qljs_diagnostic(
    "E0035", DiagnosticSeverity::Error,
    (qljs_translatable!("RegExp literal flags cannot contain Unicode escapes"), escape_sequence),
)]
pub struct DiagRegexpLiteralFlagsCannotContainUnicodeEscapes<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0037", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed block comment"), comment_open),
)]
pub struct DiagUnclosedBlockComment<'code> {
    pub comment_open: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0038", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed identifier escape sequence"), escape_sequence),
)]
pub struct DiagUnclosedIdentifierEscapeSequence<'code> {
    pub escape_sequence: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0039", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed regexp literal"), regexp_literal),
)]
pub struct DiagUnclosedRegexpLiteral<'code> {
    pub regexp_literal: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0040", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed string literal"), string_literal),
)]
pub struct DiagUnclosedStringLiteral<'code> {
    pub string_literal: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0181", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed string literal"), string_literal_begin),
)]
pub struct DiagUnclosedJsxStringLiteral<'code> {
    pub string_literal_begin: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0041", DiagnosticSeverity::Error,
    (qljs_translatable!("unclosed template"), incomplete_template),
)]
pub struct DiagUnclosedTemplate<'code> {
    pub incomplete_template: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0042", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected '@'"), character),
)]
pub struct DiagUnexpectedAtCharacter<'code> {
    pub character: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0043", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected '\\' in identifier"), backslash),
)]
pub struct DiagUnexpectedBackslashInIdentifier<'code> {
    pub backslash: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0044", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected characters in number literal"), characters),
)]
pub struct DiagUnexpectedCharactersInNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0045", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected control character"), character),
)]
pub struct DiagUnexpectedControlCharacter<'code> {
    pub character: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0046", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected characters in binary literal"), characters),
)]
pub struct DiagUnexpectedCharactersInBinaryNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0047", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected characters in octal literal"), characters),
)]
pub struct DiagUnexpectedCharactersInOctalNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0048", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected characters in hex literal"), characters),
)]
pub struct DiagUnexpectedCharactersInHexNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0182", DiagnosticSeverity::Error,
    (qljs_translatable!("'>' is not allowed directly in JSX text; write {{'>'} or &gt; instead"), greater),
)]
pub struct DiagUnexpectedGreaterInJsxText<'code> {
    pub greater: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0183", DiagnosticSeverity::Error,
    (qljs_translatable!("'}' is not allowed directly in JSX text; write {{'}'} instead"), right_curly),
)]
pub struct DiagUnexpectedRightCurlyInJsxText<'code> {
    pub right_curly: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0210", DiagnosticSeverity::Error,
    (qljs_translatable!("unopened block comment"), comment_close),
)]
pub struct DiagUnopenedBlockComment<'code> {
    pub comment_close: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0049", DiagnosticSeverity::Error,
    (qljs_translatable!("binary number literal has no digits"), characters),
)]
pub struct DiagNoDigitsInBinaryNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0050", DiagnosticSeverity::Error,
    (qljs_translatable!("hex number literal has no digits"), characters),
)]
pub struct DiagNoDigitsInHexNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0051", DiagnosticSeverity::Error,
    (qljs_translatable!("octal number literal has no digits"), characters),
)]
pub struct DiagNoDigitsInOctalNumber<'code> {
    pub characters: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0052", DiagnosticSeverity::Error,
    (qljs_translatable!("unexpected '#'"), where_),
)]
pub struct DiagUnexpectedHashCharacter<'code> {
    pub where_: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E0095", DiagnosticSeverity::Error,
    (qljs_translatable!("unicode byte order mark (BOM) cannot appear before #! at beginning of script"), bom),
)]
pub struct DiagUnexpectedBomBeforeShebang<'code> {
    pub bom: SourceCodeSpan<'code>,
}

#[qljs_diagnostic(
    "E6969", DiagnosticSeverity::Error,
    (qljs_translatable!("test for multiple messages"), a),
    (qljs_translatable!("second message here"), b),
)]
pub struct DiagMultipleMessageTest<'code> {
    pub a: SourceCodeSpan<'code>,
    pub b: SourceCodeSpan<'code>,
}

qljs_make_diag_type_enum!(DiagType);

// NOTE(port): In C++, this was a function template called diag_type_from_type.
pub trait HasDiagType {
    const TYPE_: DiagType;
}

qljs_make_has_diag_type_impls!();

const DIAG_TYPE_COUNT: i32 = qljs_diag_type_count!() as i32;

pub(crate) const ALL_DIAGNOSTIC_INFOS: &[DiagnosticInfo] = &qljs_make_diag_type_infos!();
