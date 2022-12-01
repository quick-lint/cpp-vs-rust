use cpp_vs_rust::fe::diagnostic::*;
use cpp_vs_rust::fe::diagnostic_types::*;
use cpp_vs_rust::i18n::translation::*;
#[cfg(feature = "qljs_memoffset")]
use memoffset::offset_of;

#[test]
fn diagnostic_info() {
    let mut source_code_translator = Translator::new_using_messages_from_source_code();
    source_code_translator.use_messages_from_source_code();

    {
        let info: &DiagnosticInfo =
            get_diagnostic_info(DiagType::DiagBigIntLiteralContainsDecimalPoint);
        assert_eq!(info.code(), 5);
        assert_eq!(info.severity(), DiagnosticSeverity::Error);
        assert_eq!(
            source_code_translator.translate(info.message_formats[0]),
            "BigInt literal contains decimal point"
        );
        #[cfg(feature = "qljs_memoffset")]
        assert_eq!(
            info.message_args[0][0].offset(),
            offset_of!(DiagBigIntLiteralContainsDecimalPoint, where_)
        );
        assert_eq!(
            info.message_args[0][0].type_(),
            DiagnosticArgType::SourceCodeSpan
        );
        assert!(!info.message_formats[1].valid());
    }

    {
        let info: &DiagnosticInfo =
            get_diagnostic_info(DiagType::DiagInvalidQuotesAroundStringLiteral);
        assert_eq!(info.code(), 197);
        assert_eq!(info.severity(), DiagnosticSeverity::Error);
        assert_eq!(
            source_code_translator.translate(info.message_formats[0]),
            "'{0}' is not allowed for strings; use {1} instead"
        );
        #[cfg(feature = "qljs_memoffset")]
        assert_eq!(
            info.message_args[0][0].offset(),
            offset_of!(DiagInvalidQuotesAroundStringLiteral, opening_quote)
        );
        assert_eq!(
            info.message_args[0][0].type_(),
            DiagnosticArgType::SourceCodeSpan
        );
        #[cfg(feature = "qljs_memoffset")]
        assert_eq!(
            info.message_args[0][1].offset(),
            offset_of!(DiagInvalidQuotesAroundStringLiteral, suggested_quote)
        );
        assert_eq!(info.message_args[0][1].type_(), DiagnosticArgType::Char8);
        assert!(!info.message_formats[1].valid());
    }

    {
        let info: &DiagnosticInfo = get_diagnostic_info(DiagType::DiagMultipleMessageTest);
        assert_eq!(info.code(), 6969);
        assert_eq!(info.severity(), DiagnosticSeverity::Error);
        assert_eq!(
            source_code_translator.translate(info.message_formats[0]),
            "test for multiple messages"
        );
        #[cfg(feature = "qljs_memoffset")]
        assert_eq!(
            info.message_args[0][0].offset(),
            offset_of!(DiagMultipleMessageTest, a)
        );
        assert_eq!(
            info.message_args[0][0].type_(),
            DiagnosticArgType::SourceCodeSpan
        );
        assert_eq!(
            source_code_translator.translate(info.message_formats[1]),
            "second message here"
        );
        #[cfg(feature = "qljs_memoffset")]
        assert_eq!(
            info.message_args[1][0].offset(),
            offset_of!(DiagMultipleMessageTest, b)
        );
        assert_eq!(
            info.message_args[1][0].type_(),
            DiagnosticArgType::SourceCodeSpan
        );
    }
}

#[test]
fn diagnostic_message_arg_info_type() {
    for arg_type in [
        DiagnosticArgType::Invalid,
        DiagnosticArgType::Char8,
        DiagnosticArgType::EnumKind,
        DiagnosticArgType::Identifier,
        DiagnosticArgType::SourceCodeSpan,
        DiagnosticArgType::StatementKind,
        DiagnosticArgType::String8View,
        DiagnosticArgType::VariableKind,
    ] {
        for offset in [0, 2, 4, 6, 8, 10, 12, 14, 16, 24, 32] {
            assert_eq!(
                DiagnosticMessageArgInfo::new(offset, arg_type).type_(),
                arg_type,
                "arg_type={:?} offset={:?}",
                arg_type,
                offset
            );
        }
    }
}
