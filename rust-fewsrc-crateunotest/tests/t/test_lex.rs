#![allow(clippy::redundant_static_lifetimes)]

use cpp_vs_rust::fe::diag_reporter::*;
use cpp_vs_rust::fe::diagnostic_types::*;
use cpp_vs_rust::fe::identifier::*;
use cpp_vs_rust::fe::lex::*;
use cpp_vs_rust::fe::source_code_span::*;
use cpp_vs_rust::fe::token::*;
use cpp_vs_rust::assert_matches;
use cpp_vs_rust_test::characters::*;
use cpp_vs_rust_test::diag_collector::*;
use cpp_vs_rust_test::diag_matcher::*;
use cpp_vs_rust_test::parse_support::*;
use cpp_vs_rust_test::qljs_assert_diags;
use cpp_vs_rust_test::qljs_assert_no_diags;
use cpp_vs_rust_test::scoped_trace;
use cpp_vs_rust::util::padded_string::*;
use cpp_vs_rust::util::utf_8::*;

#[test]
fn lex_block_comments() {
    let mut f = Fixture::new();

    f.check_single_token(b"/* */ hello", b"hello");
    f.check_single_token(b"/*/ comment */ hi", b"hi");
    f.check_single_token(b"/* comment /*/ hi", b"hi");
    f.check_single_token(b"/* not /* nested */ ident", b"ident");
    assert_eq!(f.lex_to_eof_types(b"/**/"), vec![]);

    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"hello /* unterminated comment ");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnclosedBlockComment {
                comment_open: b"hello "..b"/*",
            },
        );
    }
}

#[test]
fn lex_unopened_block_comment() {
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"hello */");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator); // identifier
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip(); // end of file
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnopenedBlockComment {
                comment_close: b"hello "..b"*/",
            },
        );
    }
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"*-----*/");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);

        while l.peek().type_ != TokenType::EndOfFile {
            l.skip();
        }
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnopenedBlockComment {
                comment_close: b"*-----"..b"*/",
            },
        );
    }
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"*******/");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::StarStar);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::StarStar);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::StarStar);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnopenedBlockComment {
                comment_close: b"******"..b"*/",
            },
        );
    }
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"*/");
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnopenedBlockComment {
                comment_close: 0..b"*/",
            },
        );
    }
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"**/");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Star);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnopenedBlockComment {
                comment_close: b"*"..b"*/",
            },
        );
    }
}

#[test]
fn lex_regexp_literal_starting_with_star_slash() {
    {
        // '/*' is not an end of block comment because it precedes a regexp literal
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"*/ hello/");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Star);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, unsafe { input.c_str().add(1) });
        assert_eq!(l.peek().end, input.null_terminator());
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        qljs_assert_no_diags!(v.clone_errors(), code.view());
    }
}

#[test]
fn lex_regexp_literal_starting_with_star_star_slash() {
    {
        let input = PaddedString::from_slice(b"3 **/ banana/");
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Number);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::StarStar);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, unsafe { input.c_str().add(4) });
        assert_eq!(l.peek().end, input.null_terminator());
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        qljs_assert_no_diags!(v.clone_errors(), code.view());
    }
}

#[test]
fn lex_line_comments() {
    let mut f = Fixture::new();

    assert_eq!(f.lex_to_eof_types(b"// hello"), vec![]);
    for line_terminator in LINE_TERMINATORS {
        f.check_single_token(
            format!("// hello{line_terminator}world").as_bytes(),
            b"world",
        );
    }
    assert_eq!(f.lex_to_eof_types(b"// hello\n// world"), vec![]);
    f.check_tokens(
        b"hello//*/\n \n \nworld",
        &[TokenType::Identifier, TokenType::Identifier],
    );

    /*
     * Also test for a unicode sign that starts with 0xe280, because the
     * skip_line_comment() will also look for U+2028 and U+2029
     *  > U+2028 Line Separator      (0xe280a8)
     *  > U+2029 Paragraph Separator (0xe280a9)
     *  > U+2030 Per Mille Sign      (0xe280b0)
     */
    assert_eq!(f.lex_to_eof_types("// 123‰".as_bytes()), vec![]);
}

#[test]
fn lex_line_comments_with_control_characters() {
    let mut f = Fixture::new();
    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input: String = format!("// hello {control_character} world\n42.0");
        scoped_trace!(input);
        f.check_tokens(input.as_bytes(), &[TokenType::Number]);
    }
}

#[test]
fn lex_html_open_comments() {
    let mut f = Fixture::new();

    assert_eq!(f.lex_to_eof_types(b"<!-- --> hello"), vec![]);
    for line_terminator in LINE_TERMINATORS {
        f.check_single_token(
            format!("<!-- hello{line_terminator}world").as_bytes(),
            b"world",
        );
    }
    assert_eq!(f.lex_to_eof_types(b"<!-- hello\n<!-- world"), vec![]);
    assert_eq!(f.lex_to_eof_types(b"<!--// hello"), vec![]);
    f.check_tokens(
        b"hello<!--->\n \n \nworld",
        &[TokenType::Identifier, TokenType::Identifier],
    );
    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input = PaddedString::from_slice(
            format!("<!-- hello {control_character} world\n42.0").as_bytes(),
        );
        scoped_trace!(input);
        f.check_tokens(input.as_slice(), &[TokenType::Number]);
    }

    f.check_tokens(
        b"hello<!world",
        &[
            TokenType::Identifier,
            TokenType::Less,
            TokenType::Bang,
            TokenType::Identifier,
        ],
    );
    f.check_tokens(
        b"hello<!-world",
        &[
            TokenType::Identifier,
            TokenType::Less,
            TokenType::Bang,
            TokenType::Minus,
            TokenType::Identifier,
        ],
    );
}

#[test]
fn lex_html_close_comments() {
    let mut f = Fixture::new();

    assert_eq!(f.lex_to_eof_types(b"--> comment"), vec![]);
    assert_eq!(f.lex_to_eof_types(b"     --> comment"), vec![]);
    assert_eq!(f.lex_to_eof_types(b"/* */--> comment"), vec![]);
    assert_eq!(f.lex_to_eof_types(b"/**//**/--> comment"), vec![]);

    for eol in LINE_TERMINATORS {
        f.check_single_token(format!("-->{eol}hello").as_bytes(), b"hello");
        f.check_single_token(format!("--> comment{eol}hello").as_bytes(), b"hello");
        f.check_single_token(
            format!("--> comment1{eol}--> comment2{eol}hello").as_bytes(),
            b"hello",
        );

        f.check_single_token(
            format!("/*{eol}*/--> comment{eol}hello").as_bytes(),
            b"hello",
        );
        f.check_single_token(
            format!("/* */ /*{eol}*/ --> comment{eol}hello").as_bytes(),
            b"hello",
        );
        f.check_single_token(
            format!("/*{eol}*/ /* */ --> comment{eol}hello").as_bytes(),
            b"hello",
        );
    }

    // Not an HTML close comment because non-whitespace non-comment preceeds.
    f.check_tokens(
        b"3 --> 4",
        &[
            TokenType::Number,
            TokenType::MinusMinus,
            TokenType::Greater,
            TokenType::Number,
        ],
    );
}

#[test]
fn lex_numbers() {
    let mut f = Fixture::new();

    f.check_tokens(b"0", &[TokenType::Number]);
    f.check_tokens(b"2", &[TokenType::Number]);
    f.check_tokens(b"42", &[TokenType::Number]);
    f.check_tokens(b"12.34", &[TokenType::Number]);
    f.check_tokens(b".34", &[TokenType::Number]);

    f.check_tokens(b"1e3", &[TokenType::Number]);
    f.check_tokens(b".1e3", &[TokenType::Number]);
    f.check_tokens(b"1.e3", &[TokenType::Number]);
    f.check_tokens(b"1.0e3", &[TokenType::Number]);
    f.check_tokens(b"1e-3", &[TokenType::Number]);
    f.check_tokens(b"1e+3", &[TokenType::Number]);
    f.check_tokens(b"1E+3", &[TokenType::Number]);
    f.check_tokens(b"1E123_233_22", &[TokenType::Number]);

    f.check_tokens(b"0n", &[TokenType::Number]);
    f.check_tokens(b"123456789n", &[TokenType::Number]);

    f.check_tokens(b"123_123_123", &[TokenType::Number]);
    f.check_tokens(b"123.123_123", &[TokenType::Number]);

    f.check_tokens(b"123. 456", &[TokenType::Number, TokenType::Number]);

    f.check_tokens(b"1.2.3", &[TokenType::Number, TokenType::Number]);
    f.check_tokens(b".2.3", &[TokenType::Number, TokenType::Number]);
    f.check_tokens(b"0.3", &[TokenType::Number]);
}

#[test]
fn lex_binary_numbers() {
    let mut f = Fixture::new();

    f.check_tokens(b"0b0", &[TokenType::Number]);
    f.check_tokens(b"0b1", &[TokenType::Number]);
    f.check_tokens(b"0b010101010101010", &[TokenType::Number]);
    f.check_tokens(b"0B010101010101010", &[TokenType::Number]);
    f.check_tokens(b"0b01_11_00_10", &[TokenType::Number]);
    f.check_tokens(b"0b01n", &[TokenType::Number]);

    f.check_tokens(
        b"0b0.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
    f.check_tokens(
        b"0b0101010101.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
}

#[test]
fn fail_lex_integer_loses_precision() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"9007199254740993",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagIntegerLiteralWillLosePrecision {
                    characters: 0..b"9007199254740993",
                    rounded_val: b"9007199254740992",
                },
            );
        },
    );
    f.check_tokens(b"999999999999999", &[TokenType::Number]);
    f.check_tokens(
      b"179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368",
      &[TokenType::Number]);
    f.check_tokens_with_errors(
        format!("1{}", "0".repeat(309)).as_bytes(),
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagIntegerLiteralWillLosePrecision {
                    characters: 0..310,
                    rounded_val: b"inf",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"179769313486231580000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagIntegerLiteralWillLosePrecision {
                    characters: 0..309,
                    rounded_val: b"179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"179769313486231589999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagIntegerLiteralWillLosePrecision {
                    characters: 0..309,
                    rounded_val: b"inf",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"18014398509481986",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagIntegerLiteralWillLosePrecision {
                    characters: 0..b"18014398509481986",
                    rounded_val: b"18014398509481984",
                },
            );
        },
    );
}

#[test]
fn fail_lex_binary_number_no_digits() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"0b",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInBinaryNumber {
                    characters: 0..b"0b",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0bn",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInBinaryNumber {
                    characters: 0..b"0bn",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0b;",
        &[TokenType::Number, TokenType::Semicolon],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInBinaryNumber {
                    characters: 0..b"0b",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"[0b]",
        &[
            TokenType::LeftSquare,
            TokenType::Number,
            TokenType::RightSquare,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInBinaryNumber {
                    characters: b"["..b"0b",
                },
            );
        },
    );
}

#[test]
fn fail_lex_binary_number() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"0b1ee",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInBinaryNumber {
                    characters: b"0b1"..b"ee",
                },
            );
        },
    );
}

#[test]
fn lex_modern_octal_numbers() {
    let mut f = Fixture::new();
    f.check_tokens(b"0o51", &[TokenType::Number]);
    f.check_tokens(b"0o0", &[TokenType::Number]);
    f.check_tokens(b"0O0", &[TokenType::Number]);
    f.check_tokens(b"0O12345670", &[TokenType::Number]);
    f.check_tokens(b"0o775_775", &[TokenType::Number]);
    f.check_tokens(b"0o0n", &[TokenType::Number]);
    f.check_tokens(b"0o01", &[TokenType::Number]);
    f.check_tokens(b"0o123n", &[TokenType::Number]);
}

#[test]
fn fail_lex_modern_octal_number_no_digits() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"0o",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInOctalNumber {
                    characters: 0..b"0o",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0o;",
        &[TokenType::Number, TokenType::Semicolon],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInOctalNumber {
                    characters: 0..b"0o",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"[0o]",
        &[
            TokenType::LeftSquare,
            TokenType::Number,
            TokenType::RightSquare,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInOctalNumber {
                    characters: b"["..b"0o",
                },
            );
        },
    );
}

#[test]
fn fail_lex_modern_octal_numbers() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"0o58",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInOctalNumber {
                    characters: b"0o5"..b"8",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"0o58.2",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInOctalNumber {
                    characters: b"0o5"..b"8.2",
                },
            );
        },
    );
}

#[test]
fn lex_legacy_octal_numbers_strict() {
    let mut f = Fixture::new();
    f.check_tokens(b"000", &[TokenType::Number]);
    f.check_tokens(b"001", &[TokenType::Number]);
    f.check_tokens(b"00010101010101010", &[TokenType::Number]);
    f.check_tokens(b"051", &[TokenType::Number]);

    // Legacy octal number literals which ended up actually being octal support
    // method calls with '.'.
    f.check_tokens(
        b"0123.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
    f.check_tokens(
        b"00.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
}

#[test]
fn lex_legacy_octal_numbers_lax() {
    let mut f = Fixture::new();
    f.check_tokens(b"058", &[TokenType::Number]);
    f.check_tokens(b"058.9", &[TokenType::Number]);
    f.check_tokens(b"08", &[TokenType::Number]);
}

#[test]
fn fail_lex_legacy_octal_numbers() {
    let mut f = Fixture::new();

    f.check_tokens_with_errors(
        b"0123n",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagLegacyOctalLiteralMayNotBeBigInt {
                    characters: b"0123"..b"n",
                }
            );
        },
    );

    f.check_tokens_with_errors(
        b"052.2",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagOctalLiteralMayNotHaveDecimal {
                    characters: b"052"..b".",
                }
            );
        },
    );
}

#[test]
fn legacy_octal_numbers_cannot_contain_underscores() {
    let mut f = Fixture::new();

    f.check_tokens_with_errors(
        b"0775_775",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagLegacyOctalLiteralMayNotContainUnderscores {
                    underscores: b"0775"..b"_",
                }
            );
        },
    );

    f.check_tokens_with_errors(
        b"0775____775",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagLegacyOctalLiteralMayNotContainUnderscores {
                    underscores: b"0775"..b"____",
                }
            );
        },
    );
}

#[test]
fn lex_hex_numbers() {
    let mut f = Fixture::new();

    f.check_tokens(b"0x0", &[TokenType::Number]);
    f.check_tokens(b"0x123456789abcdef", &[TokenType::Number]);
    f.check_tokens(b"0X123456789ABCDEF", &[TokenType::Number]);
    f.check_tokens(b"0X123_4567_89AB_CDEF", &[TokenType::Number]);
    f.check_tokens(b"0x1n", &[TokenType::Number]);
    f.check_tokens(b"0xfn", &[TokenType::Number]);

    f.check_tokens(
        b"0x0.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
    f.check_tokens(
        b"0xe.toString",
        &[TokenType::Number, TokenType::Dot, TokenType::Identifier],
    );
}

#[test]
fn fail_lex_hex_number_no_digits() {
    let mut f = Fixture::new();

    f.check_tokens_with_errors(
        b"0x",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInHexNumber {
                    characters: 0..b"0x",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0xn",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInHexNumber {
                    characters: 0..b"0xn",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0x;",
        &[TokenType::Number, TokenType::Semicolon],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInHexNumber {
                    characters: 0..b"0x",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"[0x]",
        &[
            TokenType::LeftSquare,
            TokenType::Number,
            TokenType::RightSquare,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNoDigitsInHexNumber {
                    characters: b"["..b"0x",
                },
            );
        },
    );
}

#[test]
fn fail_lex_hex_number() {
    let mut f = Fixture::new();

    f.check_tokens_with_errors(
        b"0xfqqn",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInHexNumber {
                    characters: b"0xf"..b"qqn",
                },
            );
        },
    );
}

#[test]
fn lex_number_with_trailing_garbage() {
    let mut f = Fixture::new();

    f.check_tokens_with_errors(
        b"123abcd",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInNumber {
                    characters: b"123"..b"abcd",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"123e f",
        &[TokenType::Number, TokenType::Identifier],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInNumber {
                    characters: b"123"..b"e",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"123e-f",
        &[TokenType::Number, TokenType::Minus, TokenType::Identifier],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInNumber {
                    characters: b"123"..b"e",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"0b01234",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInBinaryNumber {
                    characters: b"0b01"..b"234",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"0b0h0lla",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInBinaryNumber {
                    characters: b"0b0"..b"h0lla",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"0xabjjw",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInHexNumber {
                    characters: b"0xab"..b"jjw",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"0o69",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInOctalNumber {
                    characters: b"0o6"..b"9",
                }
            );
        },
    );
}

#[test]
fn lex_decimal_number_with_dot_method_call_is_invalid() {
    let mut f = Fixture::new();

    // TODO(strager): Perhaps a better diagnostic would suggest adding parentheses
    // or another '.' to make a valid method call.
    f.check_tokens_with_errors(
        b"0.toString()",
        &[
            TokenType::Number,
            TokenType::LeftParen,
            TokenType::RightParen,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInNumber {
                    characters: b"0."..b"toString",
                }
            );
        },
    );
    f.check_tokens_with_errors(
        b"09.toString",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedCharactersInNumber {
                    characters: b"09."..b"toString",
                }
            );
        },
    );

    // NOTE(strager): Other numbers with leading zeroes, like '00' and '012345',
    // are legacy octal literals and *can* have a dot method call.
}

#[test]
fn lex_invalid_big_int_number() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"12.34n",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagBigIntLiteralContainsDecimalPoint {
                    where_: 0..b"12.34n",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"1e3n",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagBigIntLiteralContainsExponent { where_: 0..b"1e3n" },
            );
        },
    );

    // Only complain about the decimal point, not the leading 0 digit.
    f.check_tokens_with_errors(
        b"0.1n",
        &[TokenType::Number],
        |_input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(errors, DiagBigIntLiteralContainsDecimalPoint,);
        },
    );

    // Complain about both the decimal point and the leading 0 digit.
    f.check_tokens_with_errors(
        b"01.2n",
        &[TokenType::Number],
        |_input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                DiagOctalLiteralMayNotHaveDecimal,
                DiagLegacyOctalLiteralMayNotBeBigInt,
            );
        },
    );

    // Complain about everything. What a disaster.
    f.check_tokens_with_errors(
        b"01.2e+3n",
        &[TokenType::Number],
        |_input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                DiagOctalLiteralMayNotHaveDecimal,
                DiagOctalLiteralMayNotHaveExponent,
                DiagLegacyOctalLiteralMayNotBeBigInt,
            );
        },
    );
}

#[test]
fn lex_number_with_double_underscore() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"123__000",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsConsecutiveUnderscores {
                    underscores: b"123"..b"__",
                },
            );
        },
    );
}

#[test]
fn lex_number_with_many_underscores() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"123_____000",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsConsecutiveUnderscores {
                    underscores: b"123"..b"_____",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0xfee_____eed",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsConsecutiveUnderscores {
                    underscores: b"0xfee"..b"_____",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0o777_____000",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsConsecutiveUnderscores {
                    underscores: b"0o777"..b"_____",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"0b111_____000",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsConsecutiveUnderscores {
                    underscores: b"0b111"..b"_____",
                },
            );
        },
    );
}

#[test]
fn lex_number_with_multiple_groups_of_consecutive_underscores() {
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"123__45___6");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Number);
        assert_eq!(unsafe { *l.peek().begin }, b'1');
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagNumberLiteralContainsConsecutiveUnderscores {
                underscores: b"123"..b"__",
            },
            DiagNumberLiteralContainsConsecutiveUnderscores {
                underscores: b"123__45"..b"___",
            },
        );
    }
}

#[test]
fn lex_number_with_trailing_underscore() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"123456_",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsTrailingUnderscores {
                    underscores: b"123456"..b"_",
                },
            );
        },
    );
}

#[test]
fn lex_number_with_trailing_underscores() {
    let mut f = Fixture::new();
    f.check_tokens_with_errors(
        b"123456___",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagNumberLiteralContainsTrailingUnderscores {
                    underscores: b"123456"..b"___",
                },
            );
        },
    );
}

#[test]
fn lex_strings() {
    let mut f = Fixture::new();

    f.check_tokens(br#"'hello'"#, &[TokenType::String]);
    f.check_tokens(br#""hello""#, &[TokenType::String]);
    f.check_tokens(br#""hello\"world""#, &[TokenType::String]);
    f.check_tokens(br#"'hello\'world'"#, &[TokenType::String]);
    f.check_tokens(br#"'hello"world'"#, &[TokenType::String]);
    f.check_tokens(br#""hello'world""#, &[TokenType::String]);
    f.check_tokens(b"'hello\\\nworld'", &[TokenType::String]);
    f.check_tokens(b"\"hello\\\nworld\"", &[TokenType::String]);
    f.check_tokens(b"'hello\\x0aworld'", &[TokenType::String]);
    f.check_tokens(br#"'\x68\x65\x6c\x6C\x6f'"#, &[TokenType::String]);
    f.check_tokens(br#"'\uabcd'"#, &[TokenType::String]);
    f.check_tokens(br#"'\u{abcd}'"#, &[TokenType::String]);

    f.check_tokens_with_errors(
        br#""unterminated"#,
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedStringLiteral {
                    string_literal: 0..br#""unterminated"#,
                },
            );
        },
    );

    for line_terminator in LINE_TERMINATORS {
        for quotation_mark in &["'", "\""] {
            let input: String =
                format!("{quotation_mark}line1\\{line_terminator}line2{quotation_mark}");
            f.check_tokens(input.as_bytes(), &[TokenType::String]);
        }
    }

    for line_terminator in LINE_TERMINATORS_EXCEPT_LS_PS {
        let v = DiagCollector::new();
        let input =
            PaddedString::from_slice(format!("'unterminated{line_terminator}hello").as_bytes());
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"hello");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnclosedStringLiteral {
                string_literal: 0..b"'unterminated",
            },
        );
    }

    for line_terminator in LINE_TERMINATORS_EXCEPT_LS_PS {
        let v = DiagCollector::new();
        let input =
            PaddedString::from_slice(format!("'separated{line_terminator}hello'").as_bytes());
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnclosedStringLiteral {
                string_literal: 0..(input.as_slice()),
            },
        );
    }

    for line_terminator in LINE_TERMINATORS_EXCEPT_LS_PS {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(
            format!("'separated{line_terminator}{line_terminator}hello'").as_bytes(),
        );
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnclosedStringLiteral {
                string_literal: 0..b"'separated",
            },
            DiagUnclosedStringLiteral {
                string_literal: (b"'separatedhello".len() + 2 * line_terminator.as_bytes().len())
                    ..b"'",
            },
        );
    }

    for line_terminator in LINE_TERMINATORS_EXCEPT_LS_PS {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(
            format!("let x = 'hello{line_terminator}let y = 'world'").as_bytes(),
        );
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::KWLet);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Equal);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::KWLet);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Equal);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::String);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnclosedStringLiteral {
                string_literal: b"let x = "..b"'hello",
            },
        );
    }

    f.check_tokens_with_errors(
        b"'unterminated\\",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedStringLiteral {
                    string_literal: 0..b"'unterminated\\",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'\\x",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'"..b"\\x",
                },
                DiagUnclosedStringLiteral {
                    string_literal: 0..b"'\\x",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'\\x1",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'"..b"\\x",
                },
                DiagUnclosedStringLiteral {
                    string_literal: 0..b"'\\x1",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'\\x'",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'"..b"\\x",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'\\x\\xyz'",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'"..b"\\x",
                },
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'\\x"..b"\\x",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'\\x1 \\xff \\xg '",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'"..b"\\x",
                },
                DiagInvalidHexEscapeSequence {
                    escape_sequence: b"'\\x1 \\xff "..b"\\x",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'hello\\u'",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"'hello"..b"\\u'",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"'hello\\u{110000}'",
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCodePointInUnicodeOutOfRange {
                    escape_sequence: b"'hello"..b"\\u{110000}",
                },
            );
        },
    );

    // TODO(#187): Report octal escape sequences in strict mode.
    // TODO(#187): Report invalid octal escape sequences in non-strict mode.
}

#[test]
fn lex_string_with_ascii_control_characters() {
    let mut f = Fixture::new();

    for control_character in [
        CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS.as_slice(),
        LS_AND_PS.as_slice(),
    ]
    .concat()
    {
        let input: String = format!("'hello{control_character}world'");
        scoped_trace!(input);
        f.check_tokens(input.as_bytes(), &[TokenType::String]);
    }

    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input: String = format!("'hello\\{control_character}world'");
        scoped_trace!(input);
        f.check_tokens(input.as_bytes(), &[TokenType::String]);
    }
}

#[test]
fn string_with_curly_quotes() {
    let mut f = Fixture::new();

    // Curly single quotes:
    f.check_tokens_with_errors(
        "\u{2018}string here\u{2019}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{2018}".as_bytes()),
                    suggested_quote: b'\'',
                },
            );
        },
    );
    f.check_tokens_with_errors(
        "\u{2019}string here\u{2018}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{2019}".as_bytes()),
                    suggested_quote: b'\'',
                },
            );
        },
    );
    f.check_tokens_with_errors(
        "\u{2018}string \u{201c} \" \u{201d} here\u{2019}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{2018}".as_bytes()),
                    suggested_quote: b'\'',
                },
            );
        },
    );

    // Curly double quotes:
    f.check_tokens_with_errors(
        "\u{201c}string here\u{201d}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{201d}".as_bytes()),
                    suggested_quote: b'"'
                },
            );
        },
    );
    f.check_tokens_with_errors(
        "\u{201d}string here\u{201c}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{201c}".as_bytes()),
                    suggested_quote: b'"',
                },
            );
        },
    );
    f.check_tokens_with_errors(
        "\u{201c}string \u{2018} ' \u{2019} here\u{201d}".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{201d}".as_bytes()),
                    suggested_quote: b'"',
                },
            );
        },
    );

    // Start with curly quote, but end with matching straight quote:
    f.check_tokens_with_errors(
        "\u{2018}string here'".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{2018}".as_bytes()),
                    suggested_quote: b'\'',
                },
            );
        },
    );
    f.check_tokens_with_errors(
        "\u{201c}string here\"".as_bytes(),
        &[TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidQuotesAroundStringLiteral {
                    opening_quote: 0..("\u{201d}".as_bytes()),
                    suggested_quote: b'"',
                },
            );
        },
    );

    // Unclosed string:
    for opening_quote in ["\u{2018}", "\u{201c}"] {
        // HACK(strager): Use a static variable to avoid a closure in the lambda.
        static mut OPENING_QUOTE_STATIC: &'static str = "";
        unsafe {
            OPENING_QUOTE_STATIC = opening_quote;
        }

        f.check_tokens_with_errors(
            format!("{opening_quote}string here").as_bytes(),
            &[TokenType::String],
            |input: PaddedStringView, errors: &Vec<AnyDiag>| {
                qljs_assert_diags!(
                    errors,
                    input,
                    DiagInvalidQuotesAroundStringLiteral,
                    DiagUnclosedStringLiteral {
                        string_literal: 0..(unsafe {
                            format!("{OPENING_QUOTE_STATIC}string here")
                        }
                        .as_bytes()),
                    },
                );
            },
        );
        for line_terminator in LINE_TERMINATORS {
            f.check_tokens_with_errors(
                format!("{opening_quote}string here{line_terminator}next_line").as_bytes(),
                &[TokenType::String, TokenType::Identifier],
                |input: PaddedStringView, errors: &Vec<AnyDiag>| {
                    qljs_assert_diags!(
                        errors,
                        input,
                        DiagInvalidQuotesAroundStringLiteral,
                        DiagUnclosedStringLiteral {
                            string_literal: 0..(unsafe {
                                format!("{OPENING_QUOTE_STATIC}string here")
                            }
                            .as_bytes()),
                        },
                    );
                },
            );
        }
    }
}

#[test]
fn lex_templates() {
    let mut f = Fixture::new();

    f.check_tokens(b"``", &[TokenType::CompleteTemplate]);
    f.check_tokens(b"`hello`", &[TokenType::CompleteTemplate]);
    f.check_tokens(b"`hello$world`", &[TokenType::CompleteTemplate]);
    f.check_tokens(b"`hello{world`", &[TokenType::CompleteTemplate]);
    f.check_tokens(br#"`hello\`world`"#, &[TokenType::CompleteTemplate]);
    f.check_tokens(br#"`hello$\{world`"#, &[TokenType::CompleteTemplate]);
    f.check_tokens(br#"`hello\${world`"#, &[TokenType::CompleteTemplate]);
    f.check_tokens(
        br#"`hello
world`"#,
        &[TokenType::CompleteTemplate],
    );
    f.check_tokens(b"`hello\\\nworld`", &[TokenType::CompleteTemplate]);
    f.check_tokens(br#"`\uabcd`"#, &[TokenType::CompleteTemplate]);
    f.check_tokens(br#"`\u{abcd}`"#, &[TokenType::CompleteTemplate]);

    {
        let code = PaddedString::from_slice(b"`hello${42}`");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        assert_eq!(l.peek().span().as_slice(), b"`hello${");
        let template_begin: *const u8 = l.peek().begin;
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Number);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);
        l.skip_in_template(template_begin);
        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        assert_eq!(l.peek().span().as_slice(), b"`");
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    {
        let code = PaddedString::from_slice(b"`${42}world`");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        assert_eq!(l.peek().span().as_slice(), b"`${");
        let template_begin: *const u8 = l.peek().begin;
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Number);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);
        l.skip_in_template(template_begin);
        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        assert_eq!(l.peek().span().as_slice(), b"world`");
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    {
        let code = PaddedString::from_slice(b"`${left}${right}`");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        let template_begin: *const u8 = l.peek().begin;
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);
        l.skip_in_template(template_begin);
        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);
        l.skip_in_template(template_begin);
        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    f.check_tokens_with_errors(
        b"`unterminated",
        &[TokenType::CompleteTemplate],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedTemplate {
                    incomplete_template: 0..b"`unterminated",
                },
            );
        },
    );

    {
        let code = PaddedString::from_slice(b"`${un}terminated");
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        let template_begin: *const u8 = l.peek().begin;
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);
        l.skip_in_template(template_begin);
        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            code.view(),
            DiagUnclosedTemplate {
                incomplete_template: 0..b"`${un}terminated",
            },
        );
    }

    f.check_tokens_with_errors(
        b"`unterminated\\",
        &[TokenType::CompleteTemplate],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedTemplate {
                    incomplete_template: 0..b"`unterminated\\",
                },
            );
        },
    );
}

#[test]
fn templates_buffer_unicode_escape_errors() {
    {
        let input = PaddedString::from_slice(b"`hello\\u`");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());
        l.peek()
            .report_errors_for_escape_sequences_in_template(&errors);
        qljs_assert_diags!(
            errors.clone_errors(),
            input.view(),
            DiagExpectedHexDigitsInUnicodeEscape {
                escape_sequence: b"`hello"..b"\\u`",
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    {
        let input = PaddedString::from_slice(b"`hello\\u{110000}`");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());
        l.peek()
            .report_errors_for_escape_sequences_in_template(&errors);
        qljs_assert_diags!(
            errors.clone_errors(),
            input.view(),
            DiagEscapedCodePointInUnicodeOutOfRange {
                escape_sequence: b"`hello"..b"\\u{110000}",
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    {
        let input = PaddedString::from_slice(b"`hello\\u${expr}`");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());
        l.peek()
            .report_errors_for_escape_sequences_in_template(&errors);
        qljs_assert_diags!(
            errors.clone_errors(),
            input.view(),
            DiagExpectedHexDigitsInUnicodeEscape {
                escape_sequence: b"`hello"..b"\\u`",
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
    }
}

#[test]
fn templates_do_not_buffer_valid_unicode_escapes() {
    {
        let input = PaddedString::from_slice(b"`hell\\u{6f}`");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::CompleteTemplate);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());
        l.peek()
            .report_errors_for_escape_sequences_in_template(&errors);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());

        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    {
        let input = PaddedString::from_slice(b"`hell\\u{6f}${expr}`");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::IncompleteTemplate);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());
        l.peek()
            .report_errors_for_escape_sequences_in_template(&errors);
        qljs_assert_no_diags!(errors.clone_errors(), input.view());

        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
    }
}

#[test]
fn lex_template_literal_with_ascii_control_characters() {
    let mut f = Fixture::new();

    for control_character in [
        CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS.as_slice(),
        LINE_TERMINATORS.as_slice(),
    ]
    .concat()
    {
        let input = PaddedString::from_slice(format!("`hello{control_character}world`").as_bytes());
        scoped_trace!(input);
        f.check_tokens(input.as_slice(), &[TokenType::CompleteTemplate]);
    }

    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input =
            PaddedString::from_slice(format!("`hello\\{control_character}world`").as_bytes());
        scoped_trace!(input);
        f.check_tokens(input.as_slice(), &[TokenType::CompleteTemplate]);
    }
}

#[test]
fn lex_regular_expression_literals() {
    fn check_regexp(raw_code: &[u8]) {
        let code = PaddedString::from_slice(raw_code);
        scoped_trace!(code);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);

        assert_matches!(l.peek().type_, TokenType::Slash | TokenType::SlashEqual);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, code.c_str());
        assert_eq!(l.peek().end, code.null_terminator());
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_no_diags!(errors.clone_errors(), input.view());
    }

    check_regexp(b"/ /");
    check_regexp(br#"/hello\/world/"#);
    check_regexp(b"/re/g");
    check_regexp(br#"/[/]/"#);
    check_regexp(br#"/[\]/]/"#);
    check_regexp(br#"/[\\]/"#);
    check_regexp(b"/=/");

    for raw_code in [b"/end_of_file", br#"/eof\"# as &[u8]] {
        let code = PaddedString::from_slice(raw_code);
        scoped_trace!(code);
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, code.c_str());
        assert_eq!(l.peek().end, code.null_terminator());

        qljs_assert_diags!(
            v.clone_errors(),
            code.view(),
            DiagUnclosedRegexpLiteral {
                regexp_literal: 0..(code.as_slice()),
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }

    for line_terminator in LINE_TERMINATORS {
        let code = PaddedString::from_slice(
            format!("/first_line{line_terminator}second_line/").as_bytes(),
        );
        scoped_trace!(code);
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, code.c_str());
        assert_eq!(l.peek().end, unsafe {
            code.c_str().add(b"/first_line".len())
        });

        qljs_assert_diags!(
            v.clone_errors(),
            code.view(),
            DiagUnclosedRegexpLiteral {
                regexp_literal: 0..b"/first_line",
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"second_line");
    }

    for line_terminator in LINE_TERMINATORS {
        let code = PaddedString::from_slice(
            format!("/first[line{line_terminator}second]line/").as_bytes(),
        );
        scoped_trace!(code);
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert_eq!(l.peek().begin, code.c_str());
        assert_eq!(l.peek().end, unsafe {
            code.c_str().add(b"/first[line".len())
        });

        qljs_assert_diags!(
            v.clone_errors(),
            code.view(),
            DiagUnclosedRegexpLiteral {
                regexp_literal: 0..b"/first[line",
            },
        );

        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"second");
    }

    // TODO(#187): Report invalid escape sequences.

    // TODO(#203): Report invalid characters and mismatched brackets.
}

#[test]
fn lex_regular_expression_literal_with_digit_flag() {
    let input = PaddedString::from_slice(b"/cellular/3g");

    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
    assert_eq!(l.peek().type_, TokenType::Slash);
    l.reparse_as_regexp();
    assert_eq!(l.peek().type_, TokenType::Regexp);
    assert_eq!(l.peek().begin, input.c_str());
    assert_eq!(l.peek().end, input.null_terminator());
    l.skip();
    assert_eq!(l.peek().type_, TokenType::EndOfFile);

    // TODO(#47): Report an error, because '3' is an invalid flag.
}

#[test]
fn lex_unicode_escape_in_regular_expression_literal_flags() {
    let errors = DiagCollector::new();
    let input = PaddedString::from_slice(b"/hello/\\u{67}i");

    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(input.view(), &errors, &allocator);
    l.reparse_as_regexp();
    assert_eq!(l.peek().type_, TokenType::Regexp);
    assert_eq!(l.peek().begin, input.c_str());
    assert_eq!(l.peek().end, input.null_terminator());
    l.skip();
    assert_eq!(l.peek().type_, TokenType::EndOfFile);

    qljs_assert_diags!(
        errors.clone_errors(),
        input.view(),
        DiagRegexpLiteralFlagsCannotContainUnicodeEscapes {
            escape_sequence: b"/hello/"..b"\\u{67}",
        },
    );
}

#[test]
fn lex_non_ascii_in_regular_expression_literal_flags() {
    let errors = DiagCollector::new();
    let input = PaddedString::from_slice("/hello/\u{05d0}".as_bytes());

    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(input.view(), &errors, &allocator);
    l.reparse_as_regexp();
    assert_eq!(l.peek().type_, TokenType::Regexp);
    assert_eq!(l.peek().begin, input.c_str());
    assert_eq!(l.peek().end, input.null_terminator());
    l.skip();
    assert_eq!(l.peek().type_, TokenType::EndOfFile);

    // TODO(#47): Report an error, because '\u05d0' is an invalid flag.
}

#[test]
fn lex_regular_expression_literals_preserves_leading_newline_flag() {
    {
        let code = PaddedString::from_slice(b"\n/ /");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert!(l.peek().has_leading_newline);
    }

    {
        let code = PaddedString::from_slice(b"/ /");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        assert!(!(l.peek().has_leading_newline));
    }
}

#[test]
fn lex_regular_expression_literal_with_ascii_control_characters() {
    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input = PaddedString::from_slice(format!("/hello{control_character}world/").as_bytes());
        scoped_trace!(input);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_no_diags!(errors.clone_errors(), input.view());
    }

    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        let input =
            PaddedString::from_slice(format!("/hello\\{control_character}world/").as_bytes());
        scoped_trace!(input);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &errors, &allocator);

        l.reparse_as_regexp();
        assert_eq!(l.peek().type_, TokenType::Regexp);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_no_diags!(errors.clone_errors(), input.view());
    }
}

#[test]
fn split_less_less_into_two_tokens() {
    let input = PaddedString::from_slice(b"<<T>() => T>");

    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
    assert_eq!(l.peek().type_, TokenType::LessLess);
    l.skip_less_less_as_less();
    assert_eq!(l.peek().type_, TokenType::Less);
    assert_eq!(l.peek().begin, unsafe { input.c_str().add(1) });
    assert_eq!(l.peek().end, unsafe { input.c_str().add(2) });
    assert_eq!(l.end_of_previous_token(), unsafe { input.c_str().add(1) });
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier, "T");
}

#[test]
fn split_less_less_has_no_leading_newline() {
    let input = PaddedString::from_slice(b"\n<<");

    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
    assert_eq!(l.peek().type_, TokenType::LessLess);
    assert!(l.peek().has_leading_newline);
    l.skip_less_less_as_less();
    assert_eq!(l.peek().type_, TokenType::Less);
    assert!(!(l.peek().has_leading_newline));
}

#[test]
fn split_greater_from_bigger_token() {
    {
        let input = PaddedString::from_slice(b">>;");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::GreaterGreater);

        l.skip_as_greater();
        assert_eq!(l.peek().type_, TokenType::Greater);
        assert_eq!(l.peek().begin, unsafe { input.c_str().add(1) });
        assert_eq!(l.peek().end, unsafe { input.c_str().add(2) });
        assert_eq!(l.end_of_previous_token(), unsafe { input.c_str().add(1) });
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Semicolon);
    }

    {
        let input = PaddedString::from_slice(b">>>;");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::GreaterGreaterGreater);

        l.skip_as_greater();
        assert_eq!(l.peek().type_, TokenType::GreaterGreater);
        assert_eq!(l.peek().begin, unsafe { input.c_str().add(1) });
        assert_eq!(l.peek().end, unsafe { input.c_str().add(3) });
        assert_eq!(l.end_of_previous_token(), unsafe { input.c_str().add(1) });
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Semicolon);
    }
}

#[test]
fn split_greater_from_bigger_token_has_no_leading_newline() {
    {
        let input = PaddedString::from_slice(b"\n>>");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
        assert_eq!(l.peek().type_, TokenType::GreaterGreater);
        assert!(l.peek().has_leading_newline);
        l.skip_as_greater();
        assert_eq!(l.peek().type_, TokenType::Greater);
        assert!(!(l.peek().has_leading_newline));
    }
}

#[test]
fn lex_identifiers() {
    let mut f = Fixture::new();
    f.check_tokens(b"i", &[TokenType::Identifier]);
    f.check_tokens(b"_", &[TokenType::Identifier]);
    f.check_tokens(b"$", &[TokenType::Identifier]);
    f.check_single_token(b"id", b"id");
    f.check_single_token(b"id ", b"id");
    f.check_single_token(b"this_is_an_identifier", b"this_is_an_identifier");
    f.check_single_token(b"MixedCaseIsAllowed", b"MixedCaseIsAllowed");
    f.check_single_token(b"ident$with$dollars", b"ident$with$dollars");
    f.check_single_token(b"digits0123456789", b"digits0123456789");
}

#[test]
fn ascii_identifier_with_escape_sequence() {
    let mut f = Fixture::new();

    f.check_single_token(b"\\u0061", b"a");
    f.check_single_token(b"\\u0041", b"A");
    f.check_single_token(b"\\u004E", b"N");
    f.check_single_token(b"\\u004e", b"N");

    f.check_single_token(b"\\u{41}", b"A");
    f.check_single_token(b"\\u{0041}", b"A");
    f.check_single_token(b"\\u{00000000000000000000041}", b"A");
    f.check_single_token(b"\\u{004E}", b"N");
    f.check_single_token(b"\\u{004e}", b"N");

    f.check_single_token(b"hell\\u006f", b"hello");
    f.check_single_token(b"\\u0068ello", b"hello");
    f.check_single_token(b"w\\u0061t", b"wat");

    f.check_single_token(b"hel\\u006c0", b"hell0");

    f.check_single_token(b"\\u0077\\u0061\\u0074", b"wat");
    f.check_single_token(b"\\u{77}\\u{61}\\u{74}", b"wat");

    // _ and $ are in IdentifierStart, even though they aren't in UnicodeIDStart.
    f.check_single_token(b"\\u{5f}wakka", b"_wakka");
    f.check_single_token(b"\\u{24}wakka", b"$wakka");

    // $, ZWNJ, ZWJ in IdentifierPart, even though they aren't in
    // UnicodeIDContinue.
    f.check_single_token(b"wakka\\u{24}", b"wakka$");
    f.check_single_token(b"wak\\u200cka", "wak\u{200c}ka".as_bytes());
    f.check_single_token(b"wak\\u200dka", "wak\u{200d}ka".as_bytes());
}

#[test]
fn non_ascii_identifier() {
    let mut f = Fixture::new();

    f.check_single_token("\u{013337}".as_bytes(), "\u{013337}".as_bytes());

    f.check_single_token("\u{00b5}".as_bytes(), "\u{00b5}".as_bytes()); // 2 UTF-8 bytes
    f.check_single_token("\u{05d0}".as_bytes(), "\u{05d0}".as_bytes()); // 3 UTF-8 bytes
    f.check_single_token("a\u{0816}".as_bytes(), "a\u{0816}".as_bytes()); // 3 UTF-8 bytes
    f.check_single_token("\u{01e93f}".as_bytes(), "\u{01e93f}".as_bytes()); // 4 UTF-8 bytes

    // KHOJKI LETTER QA, introduced in Unicode 15.
    f.check_single_token("\u{01123f}".as_bytes(), "\u{01123f}".as_bytes());
}

#[test]
fn non_ascii_identifier_with_escape_sequence() {
    let mut f = Fixture::new();

    f.check_single_token(b"\\u{013337}", "\u{013337}".as_bytes());

    f.check_single_token(b"\\u{b5}", "\u{00b5}".as_bytes()); // 2 UTF-8 bytes
    f.check_single_token(b"a\\u{816}", "a\u{0816}".as_bytes()); // 3 UTF-8 bytes
    f.check_single_token(b"a\\u0816", "a\u{0816}".as_bytes()); // 3 UTF-8 bytes
    f.check_single_token(b"\\u{1e93f}", "\u{01e93f}".as_bytes()); // 4 UTF-8 bytes
}

#[test]
fn identifier_with_escape_sequences_source_code_span_is_in_place() {
    let input: PaddedString = PaddedString::from_slice(b"\\u{77}a\\u{74}");
    let allocator = LexerAllocator::new();
    let l = Lexer::new(input.view(), null_diag_reporter(), &allocator);
    let span: SourceCodeSpan = l.peek().identifier_name().span();
    assert_eq!(span.begin_ptr(), input.c_str());
    assert_eq!(span.end_ptr(), input.null_terminator());
}

#[test]
fn lex_identifier_with_malformed_escape_sequence() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        b" are\\ufriendly ",
        b"are\\ufriendly",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b" are"..b"\\ufr",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"are\\uf riendly",
        &[TokenType::Identifier, TokenType::Identifier],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"are"..b"\\uf ",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"stray\\backslash",
        b"stray\\backslash",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedBackslashInIdentifier {
                    backslash: b"stray"..b"\\",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"stray\\",
        b"stray\\",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedBackslashInIdentifier {
                    backslash: b"stray"..b"\\",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"hello\\u}world",
        &[
            TokenType::Identifier,
            TokenType::RightCurly,
            TokenType::Identifier,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"hello"..b"\\u}",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"negative\\u-0041",
        &[TokenType::Identifier, TokenType::Minus, TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"negative"..b"\\u-",
                },
            );
        },
    );

    f.check_single_token_with_errors(
        b"a\\u{}b",
        b"a\\u{}b",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"a"..b"\\u{}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"a\\u{q}b",
        b"a\\u{q}b",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"a"..b"\\u{q}",
                },
            );
        },
    );

    f.check_single_token_with_errors(
        b"unterminated\\u",
        b"unterminated\\u",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"unterminated"..b"\\u",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"unterminated\\u012",
        b"unterminated\\u012",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagExpectedHexDigitsInUnicodeEscape {
                    escape_sequence: b"unterminated"..b"\\u012",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"unterminated\\u{",
        b"unterminated\\u{",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedIdentifierEscapeSequence {
                    escape_sequence: b"unterminated"..b"\\u{",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"unterminated\\u{0123",
        b"unterminated\\u{0123",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedIdentifierEscapeSequence {
                    escape_sequence: b"unterminated"..b"\\u{0123",
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"unclosed\\u{0123 'string'",
        &[TokenType::Identifier, TokenType::String],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedIdentifierEscapeSequence {
                    escape_sequence: b"unclosed"..b"\\u{0123",
                },
            );
        },
    );
    f.check_tokens_with_errors(
        b"unclosed\\u{+=42",
        &[
            TokenType::Identifier,
            TokenType::PlusEqual,
            TokenType::Number,
        ],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnclosedIdentifierEscapeSequence {
                    escape_sequence: b"unclosed"..b"\\u{",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_out_of_range_escaped_character() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        b"too\\u{110000}big",
        b"too\\u{110000}big",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCodePointInUnicodeOutOfRange {
                    escape_sequence: b"too"..b"\\u{110000}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"waytoo\\u{100000000000000}big",
        b"waytoo\\u{100000000000000}big",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCodePointInUnicodeOutOfRange {
                    escape_sequence: b"waytoo"..b"\\u{100000000000000}",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_out_of_range_utf_8_sequence() {
    let mut f = Fixture::new();

    // f4 90 80 80 is U+110000
    f.check_single_token_with_errors(
        b"too\xf4\x90\x80\x80\x62ig",
        b"too\xf4\x90\x80\x80\x62ig",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidUTF8Sequence {
                    sequence: b"too"..b"\xf4\x90\x80\x80",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_malformed_utf_8_sequence() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        b"illegal\xc0\xc1\xc2\xc3\xc4utf8\xfe\xff",
        b"illegal\xc0\xc1\xc2\xc3\xc4utf8\xfe\xff",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagInvalidUTF8Sequence {
                    sequence: b"illegal"..b"\xc0\xc1\xc2\xc3\xc4",
                },
                DiagInvalidUTF8Sequence {
                    sequence: b"illegal\xc0\xc1\xc2\xc3\xc4utf8"..b"\xfe\xff",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_disallowed_character_escape_sequence() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        b"illegal\\u0020",
        b"illegal\\u0020",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"illegal"..b"\\u0020",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"illegal\\u{0020}",
        b"illegal\\u{0020}",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"illegal"..b"\\u{0020}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"\\u{20}illegal",
        b"\\u{20}illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: 0..b"\\u{20}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"illegal\\u{10ffff}",
        b"illegal\\u{10ffff}",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"illegal"..b"\\u{10ffff}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"\\u{10ffff}illegal",
        b"\\u{10ffff}illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: 0..b"\\u{10ffff}",
                },
            );
        },
    );

    // U+005c is \ (backslash)
    f.check_single_token_with_errors(
        b"\\u{5c}u0061illegal",
        b"\\u{5c}u0061illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: 0..b"\\u{5c}",
                },
            );
        },
    );
    f.check_single_token_with_errors(
        b"illegal\\u{5c}u0061",
        b"illegal\\u{5c}u0061",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"illegal"..b"\\u{5c}",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_disallowed_non_ascii_character() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        "illegal\u{10ffff}".as_bytes(),
        "illegal\u{10ffff}".as_bytes(),
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagCharacterDisallowedInIdentifiers {
                    character: b"illegal"..("\u{10ffff}".as_bytes()),
                },
            );
        },
    );
    f.check_single_token_with_errors(
        "\u{10ffff}illegal".as_bytes(),
        "\u{10ffff}illegal".as_bytes(),
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagCharacterDisallowedInIdentifiers {
                    character: 0..("\u{10ffff}".as_bytes()),
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_disallowed_escaped_initial_character() {
    let mut f = Fixture::new();

    // Identifiers cannot start with a digit.
    f.check_single_token_with_errors(
        b"\\u{30}illegal",
        b"\\u{30}illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: 0..b"\\u{30}",
                },
            );
        },
    );

    f.check_single_token_with_errors(
        b"\\u0816illegal",
        b"\\u0816illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: 0..b"\\u0816",
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_disallowed_non_ascii_initial_character() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        "\u{0816}illegal".as_bytes(),
        "\u{0816}illegal".as_bytes(),
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagCharacterDisallowedInIdentifiers {
                    character: 0..("\u{0816}".as_bytes()),
                },
            );
        },
    );
}

#[test]
fn lex_identifier_with_disallowed_initial_character_as_subsequent_character() {
    let mut f = Fixture::new();

    // Identifiers can contain a digit.
    f.check_single_token(b"legal0", b"legal0");
    f.check_single_token(b"legal\\u{30}", b"legal0");

    f.check_single_token(b"legal\\u0816", "legal\u{0816}".as_bytes());
    f.check_single_token("legal\u{0816}".as_bytes(), "legal\u{0816}".as_bytes());
}

#[test]
fn lex_identifiers_which_look_like_keywords() {
    let mut f = Fixture::new();
    f.check_tokens(b"ifelse", &[TokenType::Identifier]);
    f.check_tokens(b"IF", &[TokenType::Identifier]);
}

#[test]
fn private_identifier() {
    let mut f = Fixture::new();

    f.check_tokens(b"#i", &[TokenType::PrivateIdentifier]);
    f.check_tokens(b"#_", &[TokenType::PrivateIdentifier]);
    f.check_tokens(b"#$", &[TokenType::PrivateIdentifier]);
    f.check_tokens(
        b"#Mixed_Case_With_Underscores",
        &[TokenType::PrivateIdentifier],
    );
    f.check_tokens(b"#digits0123456789", &[TokenType::PrivateIdentifier]);

    {
        let code = PaddedString::from_slice(b" #id ");
        let allocator = LexerAllocator::new();
        let errors = DiagCollector::new();
        f.lex_to_eof(code.view(), &allocator, &errors, |tokens: &Vec<Token>| {
            assert_eq!(tokens.len(), 1);
            let ident: Identifier = tokens[0].identifier_name();
            assert_eq!(ident.span().as_slice(), b"#id");
            assert_eq!(ident.normalized_name(), b"#id");
        });
        assert_matches!(errors.clone_errors(), e if e.is_empty());
    }

    f.check_single_token("#\u{00b5}".as_bytes(), "#\u{00b5}".as_bytes()); // 2 UTF-8 bytes
    f.check_single_token("#\u{05d0}".as_bytes(), "#\u{05d0}".as_bytes()); // 2 UTF-8 bytes
    f.check_single_token("#a\u{0816}".as_bytes(), "#a\u{0816}".as_bytes()); // 3 UTF-8 bytes
    f.check_single_token("#\u{01e93f}".as_bytes(), "#\u{01e93f}".as_bytes()); // 4 UTF-8 bytes

    f.check_single_token(b"#\\u{b5}", "#\u{00b5}".as_bytes());
    f.check_single_token(b"#a\\u0816", "#a\u{0816}".as_bytes());
    f.check_single_token(b"#\\u{0001e93f}", "#\u{01e93f}".as_bytes());

    {
        let code = PaddedString::from_slice(b" #\\u{78} ");
        let allocator = LexerAllocator::new();
        let errors = DiagCollector::new();
        f.lex_to_eof(code.view(), &allocator, &errors, |tokens: &Vec<Token>| {
            assert_eq!(tokens.len(), 1);
            let ident: Identifier = tokens[0].identifier_name();
            assert_eq!(ident.span().as_slice(), b"#\\u{78}");
            assert_eq!(ident.normalized_name(), b"#x");
        });
        assert_matches!(errors.clone_errors(), e if e.is_empty());
    }

    // Keywords are allowed.
    f.check_tokens(b"#async", &[TokenType::PrivateIdentifier]);
    f.check_tokens(b"#for", &[TokenType::PrivateIdentifier]);
    f.check_tokens(b"#function", &[TokenType::PrivateIdentifier]);
    f.check_tokens(b"#let", &[TokenType::PrivateIdentifier]);
}

#[test]
fn private_identifier_with_disallowed_non_ascii_initial_character() {
    let mut f = Fixture::new();

    f.check_single_token_with_errors(
        "#\u{0816}illegal".as_bytes(),
        "#\u{0816}illegal".as_bytes(),
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagCharacterDisallowedInIdentifiers {
                    character: b"#"..("\u{0816}".as_bytes()),
                },
            );
        },
    );

    f.check_tokens_with_errors(
        b"#123",
        &[TokenType::Number],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagUnexpectedHashCharacter { where_: 0..b"#" },
            );
        },
    );
}

#[test]
fn private_identifier_with_disallowed_escaped_initial_character() {
    let mut f = Fixture::new();

    // Private identifiers cannot start with a digit.
    f.check_single_token_with_errors(
        b"#\\u{30}illegal",
        b"#\\u{30}illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"#"..b"\\u{30}",
                },
            );
        },
    );

    f.check_single_token_with_errors(
        b"#\\u0816illegal",
        b"#\\u0816illegal",
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedCharacterDisallowedInIdentifiers {
                    escape_sequence: b"#"..b"\\u0816",
                },
            );
        },
    );
}

#[test]
fn lex_reserved_keywords() {
    let mut f = Fixture::new();
    f.check_tokens(b"await", &[TokenType::KWAwait]);
    f.check_tokens(b"break", &[TokenType::KWBreak]);
    f.check_tokens(b"case", &[TokenType::KWCase]);
    f.check_tokens(b"catch", &[TokenType::KWCatch]);
    f.check_tokens(b"class", &[TokenType::KWClass]);
    f.check_tokens(b"const", &[TokenType::KWConst]);
    f.check_tokens(b"continue", &[TokenType::KWContinue]);
    f.check_tokens(b"debugger", &[TokenType::KWDebugger]);
    f.check_tokens(b"default", &[TokenType::KWDefault]);
    f.check_tokens(b"delete", &[TokenType::KWDelete]);
    f.check_tokens(b"do", &[TokenType::KWDo]);
    f.check_tokens(b"else", &[TokenType::KWElse]);
    f.check_tokens(b"enum", &[TokenType::KWEnum]);
    f.check_tokens(b"export", &[TokenType::KWExport]);
    f.check_tokens(b"extends", &[TokenType::KWExtends]);
    f.check_tokens(b"false", &[TokenType::KWFalse]);
    f.check_tokens(b"finally", &[TokenType::KWFinally]);
    f.check_tokens(b"for", &[TokenType::KWFor]);
    f.check_tokens(b"function", &[TokenType::KWFunction]);
    f.check_tokens(b"if", &[TokenType::KWIf]);
    f.check_tokens(b"implements", &[TokenType::KWImplements]);
    f.check_tokens(b"import", &[TokenType::KWImport]);
    f.check_tokens(b"in", &[TokenType::KWIn]);
    f.check_tokens(b"instanceof", &[TokenType::KWInstanceof]);
    f.check_tokens(b"interface", &[TokenType::KWInterface]);
    f.check_tokens(b"new", &[TokenType::KWNew]);
    f.check_tokens(b"null", &[TokenType::KWNull]);
    f.check_tokens(b"package", &[TokenType::KWPackage]);
    f.check_tokens(b"private", &[TokenType::KWPrivate]);
    f.check_tokens(b"protected", &[TokenType::KWProtected]);
    f.check_tokens(b"public", &[TokenType::KWPublic]);
    f.check_tokens(b"return", &[TokenType::KWReturn]);
    f.check_tokens(b"super", &[TokenType::KWSuper]);
    f.check_tokens(b"switch", &[TokenType::KWSwitch]);
    f.check_tokens(b"this", &[TokenType::KWThis]);
    f.check_tokens(b"throw", &[TokenType::KWThrow]);
    f.check_tokens(b"true", &[TokenType::KWTrue]);
    f.check_tokens(b"try", &[TokenType::KWTry]);
    f.check_tokens(b"typeof", &[TokenType::KWTypeof]);
    f.check_tokens(b"var", &[TokenType::KWVar]);
    f.check_tokens(b"void", &[TokenType::KWVoid]);
    f.check_tokens(b"while", &[TokenType::KWWhile]);
    f.check_tokens(b"with", &[TokenType::KWWith]);
    f.check_tokens(b"yield", &[TokenType::KWYield]);
}

#[test]
fn lex_contextual_keywords() {
    let mut f = Fixture::new();
    f.check_tokens(b"as", &[TokenType::KWAs]);
    f.check_tokens(b"async", &[TokenType::KWAsync]);
    f.check_tokens(b"from", &[TokenType::KWFrom]);
    f.check_tokens(b"get", &[TokenType::KWGet]);
    f.check_tokens(b"let", &[TokenType::KWLet]);
    f.check_tokens(b"of", &[TokenType::KWOf]);
    f.check_tokens(b"set", &[TokenType::KWSet]);
    f.check_tokens(b"static", &[TokenType::KWStatic]);
}

#[test]
fn lex_typescript_contextual_keywords() {
    let mut f = Fixture::new();
    f.check_tokens(b"abstract", &[TokenType::KWAbstract]);
    f.check_tokens(b"any", &[TokenType::KWAny]);
    f.check_tokens(b"assert", &[TokenType::KWAssert]);
    f.check_tokens(b"asserts", &[TokenType::KWAsserts]);
    f.check_tokens(b"bigint", &[TokenType::KWBigint]);
    f.check_tokens(b"boolean", &[TokenType::KWBoolean]);
    f.check_tokens(b"constructor", &[TokenType::KWConstructor]);
    f.check_tokens(b"declare", &[TokenType::KWDeclare]);
    f.check_tokens(b"global", &[TokenType::KWGlobal]);
    f.check_tokens(b"infer", &[TokenType::KWInfer]);
    f.check_tokens(b"intrinsic", &[TokenType::KWIntrinsic]);
    f.check_tokens(b"is", &[TokenType::KWIs]);
    f.check_tokens(b"keyof", &[TokenType::KWKeyof]);
    f.check_tokens(b"module", &[TokenType::KWModule]);
    f.check_tokens(b"namespace", &[TokenType::KWNamespace]);
    f.check_tokens(b"never", &[TokenType::KWNever]);
    f.check_tokens(b"number", &[TokenType::KWNumber]);
    f.check_tokens(b"object", &[TokenType::KWObject]);
    f.check_tokens(b"out", &[TokenType::KWOut]);
    f.check_tokens(b"override", &[TokenType::KWOverride]);
    f.check_tokens(b"readonly", &[TokenType::KWReadonly]);
    f.check_tokens(b"require", &[TokenType::KWRequire]);
    f.check_tokens(b"string", &[TokenType::KWString]);
    f.check_tokens(b"symbol", &[TokenType::KWSymbol]);
    f.check_tokens(b"type", &[TokenType::KWType]);
    f.check_tokens(b"undefined", &[TokenType::KWUndefined]);
    f.check_tokens(b"unique", &[TokenType::KWUnique]);
    f.check_tokens(b"unknown", &[TokenType::KWUnknown]);
}

#[test]
fn lex_reserved_keywords_except_await_and_yield_sometimes_cannot_contain_escape_sequences() {
    // TODO(#73): Also lex 'protected', 'implements', etc. as
    // reserved_keyword_with_escape_sequence in strict mode.
    for keyword in DISALLOWED_BINDING_IDENTIFIER_KEYWORDS.iter() {
        let code = PaddedString::from_slice(escape_first_character_in_keyword(keyword).as_bytes());
        scoped_trace!(code);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let l = Lexer::new(code.view(), &errors, &allocator);

        assert_eq!(l.peek().type_, TokenType::ReservedKeywordWithEscapeSequence);
        assert_eq!(
            l.peek().identifier_name().normalized_name(),
            keyword.as_bytes()
        );
        qljs_assert_no_diags!(errors.clone_errors(), code.view());

        l.peek()
            .report_errors_for_escape_sequences_in_keyword(&errors);
        qljs_assert_diags!(
            errors.clone_errors(),
            code.view(),
            DiagKeywordsCannotContainEscapeSequences {
                escape_sequence: 0..b"\\u{??}",
            },
        );
    }
}

#[test]
fn lex_contextual_keywords_and_await_and_yield_can_contain_escape_sequences() {
    let mut f = Fixture::new();
    for keyword in CONTEXTUAL_KEYWORDS.iter() {
        let code: String = escape_first_character_in_keyword(keyword);
        scoped_trace!(code);
        scoped_trace!(keyword);
        f.check_single_token(code.as_bytes(), keyword.as_bytes());
    }
}

#[test]
fn lex_single_character_symbols() {
    let mut f = Fixture::new();
    f.check_tokens(b"+", &[TokenType::Plus]);
    f.check_tokens(b"-", &[TokenType::Minus]);
    f.check_tokens(b"*", &[TokenType::Star]);
    f.check_tokens(b"/", &[TokenType::Slash]);
    f.check_tokens(b"<", &[TokenType::Less]);
    f.check_tokens(b">", &[TokenType::Greater]);
    f.check_tokens(b"=", &[TokenType::Equal]);
    f.check_tokens(b"&", &[TokenType::Ampersand]);
    f.check_tokens(b"^", &[TokenType::Circumflex]);
    f.check_tokens(b"!", &[TokenType::Bang]);
    f.check_tokens(b".", &[TokenType::Dot]);
    f.check_tokens(b",", &[TokenType::Comma]);
    f.check_tokens(b"~", &[TokenType::Tilde]);
    f.check_tokens(b"%", &[TokenType::Percent]);
    f.check_tokens(b"(", &[TokenType::LeftParen]);
    f.check_tokens(b")", &[TokenType::RightParen]);
    f.check_tokens(b"[", &[TokenType::LeftSquare]);
    f.check_tokens(b"]", &[TokenType::RightSquare]);
    f.check_tokens(b"{", &[TokenType::LeftCurly]);
    f.check_tokens(b"}", &[TokenType::RightCurly]);
    f.check_tokens(b":", &[TokenType::Colon]);
    f.check_tokens(b";", &[TokenType::Semicolon]);
    f.check_tokens(b"?", &[TokenType::Question]);
    f.check_tokens(b"|", &[TokenType::Pipe]);
}

#[test]
fn lex_multi_character_symbols() {
    let mut f = Fixture::new();
    f.check_tokens(b"<=", &[TokenType::LessEqual]);
    f.check_tokens(b">=", &[TokenType::GreaterEqual]);
    f.check_tokens(b"==", &[TokenType::EqualEqual]);
    f.check_tokens(b"===", &[TokenType::EqualEqualEqual]);
    f.check_tokens(b"!=", &[TokenType::BangEqual]);
    f.check_tokens(b"!==", &[TokenType::BangEqualEqual]);
    f.check_tokens(b"**", &[TokenType::StarStar]);
    f.check_tokens(b"++", &[TokenType::PlusPlus]);
    f.check_tokens(b"--", &[TokenType::MinusMinus]);
    f.check_tokens(b"<<", &[TokenType::LessLess]);
    f.check_tokens(b">>", &[TokenType::GreaterGreater]);
    f.check_tokens(b">>>", &[TokenType::GreaterGreaterGreater]);
    f.check_tokens(b"&&", &[TokenType::AmpersandAmpersand]);
    f.check_tokens(b"||", &[TokenType::PipePipe]);
    f.check_tokens(b"+=", &[TokenType::PlusEqual]);
    f.check_tokens(b"-=", &[TokenType::MinusEqual]);
    f.check_tokens(b"*=", &[TokenType::StarEqual]);
    f.check_tokens(b"/=", &[TokenType::SlashEqual]);
    f.check_tokens(b"%=", &[TokenType::PercentEqual]);
    f.check_tokens(b"**=", &[TokenType::StarStarEqual]);
    f.check_tokens(b"&&=", &[TokenType::AmpersandAmpersandEqual]);
    f.check_tokens(b"&=", &[TokenType::AmpersandEqual]);
    f.check_tokens(b"?.", &[TokenType::QuestionDot]);
    f.check_tokens(b"??", &[TokenType::QuestionQuestion]);
    f.check_tokens(b"??=", &[TokenType::QuestionQuestionEqual]);
    f.check_tokens(b"^=", &[TokenType::CircumflexEqual]);
    f.check_tokens(b"|=", &[TokenType::PipeEqual]);
    f.check_tokens(b"||=", &[TokenType::PipePipeEqual]);
    f.check_tokens(b"<<=", &[TokenType::LessLessEqual]);
    f.check_tokens(b">>=", &[TokenType::GreaterGreaterEqual]);
    f.check_tokens(b">>>=", &[TokenType::GreaterGreaterGreaterEqual]);
    f.check_tokens(b"=>", &[TokenType::EqualGreater]);
    f.check_tokens(b"...", &[TokenType::DotDotDot]);
}

#[test]
fn lex_adjacent_symbols() {
    let mut f = Fixture::new();
    f.check_tokens(b"{}", &[TokenType::LeftCurly, TokenType::RightCurly]);
    f.check_tokens(b"[]", &[TokenType::LeftSquare, TokenType::RightSquare]);
    f.check_tokens(b"/!", &[TokenType::Slash, TokenType::Bang]);
    f.check_tokens(b"*==", &[TokenType::StarEqual, TokenType::Equal]);
    f.check_tokens(b"^>>", &[TokenType::Circumflex, TokenType::GreaterGreater]);
}

#[test]
fn lex_symbols_separated_by_whitespace() {
    let mut f = Fixture::new();
    f.check_tokens(b"{ }", &[TokenType::LeftCurly, TokenType::RightCurly]);
    f.check_tokens(b"< =", &[TokenType::Less, TokenType::Equal]);
    f.check_tokens(b"? .", &[TokenType::Question, TokenType::Dot]);
    f.check_tokens(b". . .", &[TokenType::Dot, TokenType::Dot, TokenType::Dot]);
}

#[test]
fn question_followed_by_number_is_not_question_dot() {
    let mut f = Fixture::new();
    f.check_tokens(b"?.3", &[TokenType::Question, TokenType::Number]);
}

#[test]
fn question_dot_followed_by_non_digit_is_question_dot() {
    let mut f = Fixture::new();
    f.check_tokens(b"?.e", &[TokenType::QuestionDot, TokenType::Identifier]);
}

#[test]
fn lex_whitespace() {
    let mut f = Fixture::new();
    for whitespace in &[
        "\n",       //
        "\r",       //
        "\r\n",     //
        "\u{2028}", // 0xe2 0x80 0xa8 Line Separator
        "\u{2029}", // 0xe2 0x80 0xa9 Paragraph Separator
        " ",        //
        "\t",       //
        "\u{000c}", // 0x0c Form Feed
        "\u{000b}", // 0x0b Vertical Tab
        "\u{00a0}", // 0xc2 0xa0      No-Break Space (NBSP)
        "\u{1680}", // 0xe1 0x9a 0x80 Ogham Space Mark
        "\u{2000}", // 0xe2 0x80 0x80 En Quad
        "\u{2001}", // 0xe2 0x80 0x81 Em Quad
        "\u{2002}", // 0xe2 0x80 0x82 En Space
        "\u{2003}", // 0xe2 0x80 0x83 Em Space
        "\u{2004}", // 0xe2 0x80 0x84 Three-Per-Em Space
        "\u{2005}", // 0xe2 0x80 0x85 Four-Per-Em Space
        "\u{2006}", // 0xe2 0x80 0x86 Six-Per-Em Space
        "\u{2007}", // 0xe2 0x80 0x87 Figure Space
        "\u{2008}", // 0xe2 0x80 0x88 Punctuation Space
        "\u{2009}", // 0xe2 0x80 0x89 Thin Space
        "\u{200a}", // 0xe2 0x80 0x8a Hair Space
        "\u{202f}", // 0xe2 0x80 0xaf Narrow No-Break Space (NNBSP)
        "\u{205f}", // 0xe2 0x81 0x9f Medium Mathematical Space (MMSP)
        "\u{3000}", // 0xe3 0x80 0x80 Ideographic Space
        "\u{feff}", // 0xef 0xbb 0xbf Zero Width No-Break Space (BOM, ZWNBSP)
    ] {
        {
            let input: String = format!("a{whitespace}b");
            scoped_trace!(input);
            f.check_tokens(
                input.as_bytes(),
                &[TokenType::Identifier, TokenType::Identifier],
            );
        }

        {
            let input: String = format!("{whitespace}10{whitespace}'hi'{whitespace}");
            scoped_trace!(input);
            f.check_tokens(input.as_bytes(), &[TokenType::Number, TokenType::String]);
        }

        {
            let input: String = format!("async{whitespace}function{whitespace}");
            scoped_trace!(input);
            f.check_tokens(
                input.as_bytes(),
                &[TokenType::KWAsync, TokenType::KWFunction],
            );
        }
    }
}

#[test]
fn lex_shebang() {
    let mut f = Fixture::new();
    f.check_tokens(b"#!/usr/bin/env node\nhello", &[TokenType::Identifier]);
    f.check_tokens(b"#!ignored\n123", &[TokenType::Number]);
}

#[test]
fn lex_not_shebang() {
    // Whitespace must not appear between '#' and '!'.
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"# !notashebang");
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Bang, "# should be skipped");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedHashCharacter { where_: 0..b"#" },
        );
    }

    // '#!' must be on the first line.
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"\n#!notashebang\n");
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Bang, "# should be skipped");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedHashCharacter {
                where_: b"\n"..b"#",
            },
        );
    }

    // Whitespace must not appear before '#!'.
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"  #!notashebang\n");
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Bang, "# should be skipped");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedHashCharacter {
                where_: b"  "..b"#",
            },
        );
    }

    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"#\\u{21}\n");
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::PrivateIdentifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"#\\u{21}");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagEscapedCharacterDisallowedInIdentifiers {
                escape_sequence: b"#"..b"\\u{21}",
            },
        );
    }
}

#[test]
fn lex_unexpected_bom_before_shebang() {
    // BOM must not appear before '#!'.
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice("\u{feff}#!notashebang\n".as_bytes());
        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::EndOfFile, "# should be skipped");

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedBomBeforeShebang {
                bom: 0..("\u{feff}".as_bytes()),
            },
        );
    }
}

#[test]
fn lex_invalid_common_characters_are_disallowed() {
    {
        let v = DiagCollector::new();
        let input = PaddedString::from_slice(b"hello @ world");
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier, "@ should be skipped");
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedAtCharacter {
                character: b"hello "..b"@",
            },
        );
    }
}

#[test]
fn ascii_control_characters_are_disallowed() {
    for control_character in CONTROL_CHARACTERS_EXCEPT_WHITESPACE {
        let input = PaddedString::from_slice(format!("{control_character}hello").as_bytes());
        scoped_trace!(input);
        let v = DiagCollector::new();

        let allocator = LexerAllocator::new();
        let l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(
            l.peek().type_,
            TokenType::Identifier,
            "control character should be skipped"
        );
        qljs_assert_diags!(
            v.clone_errors(),
            input.view(),
            DiagUnexpectedControlCharacter {
                character: 0..(control_character.as_bytes()),
            },
        );
    }
}

#[test]
fn ascii_control_characters_sorta_treated_like_whitespace() {
    for control_character in CONTROL_CHARACTERS_EXCEPT_WHITESPACE {
        let input = PaddedString::from_slice(format!("  {control_character}  hello").as_bytes());
        scoped_trace!(input);
        let v = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(input.view(), &v, &allocator);
        assert_eq!(
            l.peek().type_,
            TokenType::Identifier,
            "control character should be skipped"
        );
        l.skip();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);
    }
}

#[test]
fn lex_token_notes_leading_newline() {
    for line_terminator in LINE_TERMINATORS {
        let code = PaddedString::from_slice(format!("a b{line_terminator}c d").as_bytes());
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert!(!l.peek().has_leading_newline); // a
        l.skip();
        assert!(!l.peek().has_leading_newline); // b
        l.skip();
        assert!(l.peek().has_leading_newline); // c
        l.skip();
        assert!(!l.peek().has_leading_newline); // d
    }
}

#[test]
fn lex_token_notes_leading_newline_after_single_line_comment() {
    for line_terminator in LINE_TERMINATORS {
        let code = PaddedString::from_slice(format!("a // hello{line_terminator}b").as_bytes());
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert!(!l.peek().has_leading_newline); // a
        l.skip();
        assert!(l.peek().has_leading_newline); // b
    }
}

#[test]
fn lex_token_notes_leading_newline_after_comment_with_newline() {
    for line_terminator in LINE_TERMINATORS {
        let code = PaddedString::from_slice(format!("a /*{line_terminator}*/ b").as_bytes());
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
        assert!(!l.peek().has_leading_newline); // a
        l.skip();
        assert!(l.peek().has_leading_newline); // b
    }
}

#[test]
fn lex_token_notes_leading_newline_after_comment() {
    let code = PaddedString::from_slice(b"a /* comment */\nb");
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);
    assert!(!l.peek().has_leading_newline); // a
    l.skip();
    assert!(l.peek().has_leading_newline); // b
}

#[test]
fn inserting_semicolon_at_newline_remembers_next_token() {
    let code = PaddedString::from_slice(b"hello\nworld");
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"hello");
    assert!(!(l.peek().has_leading_newline));
    let hello_end: *const u8 = l.peek().end;
    l.skip();

    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"world");
    assert!(l.peek().has_leading_newline);
    l.insert_semicolon();
    assert_eq!(l.peek().type_, TokenType::Semicolon);
    assert!(!(l.peek().has_leading_newline));
    assert_eq!(l.peek().begin, hello_end);
    assert_eq!(l.peek().end, hello_end);
    l.skip();

    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"world");
    assert!(l.peek().has_leading_newline);
    l.skip();

    assert_eq!(l.peek().type_, TokenType::EndOfFile);
}

#[test]
fn insert_semicolon_at_beginning_of_input() {
    let code = PaddedString::from_slice(b"hello world");
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), null_diag_reporter(), &allocator);

    l.insert_semicolon();
    assert_eq!(l.peek().type_, TokenType::Semicolon);
    assert!(!l.peek().has_leading_newline);
    assert_eq!(l.peek().begin, code.c_str());
    assert_eq!(l.peek().end, code.c_str());

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"hello");

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"world");

    l.skip();
    assert_eq!(l.peek().type_, TokenType::EndOfFile);
}

#[test]
fn inserting_semicolon_at_right_curly_remembers_next_token() {
    let code = PaddedString::from_slice(b"{ x }");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::LeftCurly);
    assert!(!l.peek().has_leading_newline);
    l.skip();

    assert_eq!(l.peek().type_, TokenType::Identifier);
    assert_eq!(l.peek().identifier_name().normalized_name(), b"x");
    assert!(!l.peek().has_leading_newline);
    let x_end: *const u8 = l.peek().end;
    l.skip();

    assert_eq!(l.peek().type_, TokenType::RightCurly);
    assert!(!(l.peek().has_leading_newline));
    l.insert_semicolon();
    assert_eq!(l.peek().type_, TokenType::Semicolon);
    assert!(!l.peek().has_leading_newline);
    assert_eq!(l.peek().begin, x_end);
    assert_eq!(l.peek().end, x_end);
    l.skip();

    assert_eq!(l.peek().type_, TokenType::RightCurly);
    assert!(!l.peek().has_leading_newline);
    l.skip();

    assert_eq!(l.peek().type_, TokenType::EndOfFile);

    qljs_assert_no_diags!(errors.clone_errors(), code.view());
}

#[test]
fn transaction_buffers_errors_until_commit() {
    let code = PaddedString::from_slice(b"x 0b y");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);
    qljs_assert_no_diags!(
        errors.clone_errors(),
        code.view(),
        "0b error shouldn't be written to error reporter"
    );

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.commit_transaction(transaction);
    qljs_assert_diags!(errors.clone_errors(), DiagNoDigitsInBinaryNumber);
}

#[test]
fn nested_transaction_buffers_errors_until_outer_commit() {
    let code = PaddedString::from_slice(b"x y 0b z");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier); // x
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let outer_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // y

    let inner_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number); // 0b
    qljs_assert_no_diags!(
        errors.clone_errors(),
        code.view(),
        "0b error shouldn't be written to error reporter",
    );

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // z
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.commit_transaction(inner_transaction);
    qljs_assert_no_diags!(
        errors.clone_errors(),
        code.view(),
        "committing inner_transaction should not report 0b error",
    );

    l.commit_transaction(outer_transaction);
    qljs_assert_diags!(
        errors.clone_errors(),
        DiagNoDigitsInBinaryNumber,
        "committing outer_transaction should report 0b error",
    );
}

#[test]
fn rolled_back_inner_transaction_discards_errors() {
    let code = PaddedString::from_slice(b"x y 0b z");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier); // x
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let outer_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // y

    let inner_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number); // 0b

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // z
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.roll_back_transaction(inner_transaction);
    l.commit_transaction(outer_transaction);
    qljs_assert_no_diags!(
        errors.clone_errors(),
        code.view(),
        "0b error shouldn't be written to error reporter",
    );
}

#[test]
fn rolled_back_outer_transaction_discards_errors() {
    let code = PaddedString::from_slice(b"x y 0b z");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier); // x
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let outer_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // y

    let inner_transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number); // 0b

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Identifier); // z
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.commit_transaction(inner_transaction);
    l.roll_back_transaction(outer_transaction);
    qljs_assert_no_diags!(
        errors.clone_errors(),
        code.view(),
        "0b error shouldn't be written to error reporter",
    );
}

#[test]
fn errors_after_transaction_commit_are_reported_unbuffered() {
    let code = PaddedString::from_slice(b"x 'y' 0b");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    l.commit_transaction(transaction);
    assert_eq!(l.peek().type_, TokenType::String);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);
    qljs_assert_diags!(errors.clone_errors(), DiagNoDigitsInBinaryNumber);
}

#[test]
fn errors_after_transaction_rollback_are_reported_unbuffered() {
    let code = PaddedString::from_slice(b"x 'y' 0b");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let transaction: LexerTransaction = l.begin_transaction();
    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    l.roll_back_transaction(transaction);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);
    qljs_assert_diags!(errors.clone_errors(), DiagNoDigitsInBinaryNumber);
}

#[test]
fn rolling_back_transaction() {
    let code = PaddedString::from_slice(b"x 'y' 3");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    let transaction: LexerTransaction = l.begin_transaction();
    assert_eq!(l.peek().type_, TokenType::Identifier);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);

    l.roll_back_transaction(transaction);
    assert_eq!(l.peek().type_, TokenType::Identifier);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);
}

#[test]
fn insert_semicolon_after_rolling_back_transaction() {
    let code = PaddedString::from_slice(b"x 'y' 3");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    assert_eq!(l.peek().type_, TokenType::Identifier);
    qljs_assert_no_diags!(errors.clone_errors(), code.view());

    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    let transaction: LexerTransaction = l.begin_transaction();

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);

    l.roll_back_transaction(transaction);
    l.insert_semicolon();
    assert_eq!(l.peek().type_, TokenType::Semicolon);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::String);

    l.skip();
    assert_eq!(l.peek().type_, TokenType::Number);
}

#[test]
fn unfinished_transaction_does_not_leak_memory() {
    // This test relies on a leak checker such as Valgrind's memtest or
    // Clang's LeakSanitizer.

    let code = PaddedString::from_slice(b"a b c d e f g");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);

    let _outer_transaction: LexerTransaction = l.begin_transaction();
    l.skip();

    let _inner_transaction: LexerTransaction = l.begin_transaction();
    l.skip();

    // Don't end either transaction. The leak checker should report no leaks.
}

#[test]
fn is_initial_identifier_byte_agrees_with_is_initial_identifier_character() {
    const MIN_CODE_POINT: char = '\0';
    const MAX_CODE_POINT: char = '\u{10ffff}';

    let mut is_valid_byte: [bool; 256] = [false; 256];
    is_valid_byte[b'\\' as usize] = true;
    for c in MIN_CODE_POINT..=MAX_CODE_POINT {
        if is_initial_identifier_character(c as u32) {
            let mut utf_8: [u8; 10] = [0; 10];
            encode_utf_8(c as u32, &mut utf_8);
            is_valid_byte[utf_8[0] as usize] = true;
        }
    }

    for byte in 0..is_valid_byte.len() {
        assert_eq!(
            is_initial_identifier_byte(byte as u8),
            is_valid_byte[byte],
            "byte = 0x{:x}",
            byte,
        );
    }
}

#[test]
fn is_identifier_byte_agrees_with_is_identifier_character() {
    const MIN_CODE_POINT: char = '\0';
    const MAX_CODE_POINT: char = '\u{10ffff}';

    let mut is_valid_byte: [bool; 256] = [false; 256];
    is_valid_byte[b'\\' as usize] = true;
    for c in MIN_CODE_POINT..=MAX_CODE_POINT {
        if is_identifier_character(c as u32, IdentifierKind::JavaScript) {
            let mut utf_8: [u8; 10] = [0; 10];
            encode_utf_8(c as u32, &mut utf_8);
            is_valid_byte[utf_8[0] as usize] = true;
        }
    }

    for byte in 0..is_valid_byte.len() {
        assert_eq!(
            is_identifier_byte(byte as u8),
            is_valid_byte[byte],
            "byte = 0x{:x}",
            byte,
        );
    }
}

#[test]
fn jsx_identifier() {
    fn check_identifier(tag_code: &[u8], expected_normalized: &[u8]) {
        scoped_trace!(tag_code);

        let mut code_vec: Vec<u8> = vec![b'!'];
        code_vec.extend_from_slice(tag_code);
        let code = PaddedString::from_slice(&code_vec[..]);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '!'.

        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(
            l.peek().identifier_name().normalized_name(),
            expected_normalized
        );

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    check_identifier(b"div", b"div");
    check_identifier(b"MyComponent", b"MyComponent");
    check_identifier(b"my-web-component", b"my-web-component");
    check_identifier(b"MY-WEB-COMPONENT", b"MY-WEB-COMPONENT");
    check_identifier(b"test-0", b"test-0");
    check_identifier(b"_component_", b"_component_");
    check_identifier(b"$component$", b"$component$");
    check_identifier(b"test-", b"test-");

    // NOTE(strager): Babel [1] and some other tools reject these. TypeScript
    // allows these. My reading of the spec is that these are allowed.
    //
    // [1] https://github.com/babel/babel/issues/14060
    check_identifier(b"\\u{48}ello", b"Hello");
    check_identifier(b"\\u{68}ello-world", b"hello-world");

    check_identifier(b" div ", b"div");
    check_identifier(b"/**/div/**/", b"div");
    check_identifier(b" banana-split ", b"banana-split");
    check_identifier(b"/**/banana-split/**/", b"banana-split");

    for line_terminator in LINE_TERMINATORS {
        check_identifier(
            format!("{line_terminator}banana-split{line_terminator}").as_bytes(),
            b"banana-split",
        );
    }

    check_identifier(b"<!-- line comment\nbanana-split", b"banana-split");
    check_identifier(b"\n--> line comment\nbanana-split", b"banana-split");

    check_identifier("\u{00c1}gua".as_bytes(), "\u{00c1}gua".as_bytes());
    check_identifier("\u{00c1}gua-".as_bytes(), "\u{00c1}gua-".as_bytes());
}

#[test]
fn invalid_jsx_identifier() {
    let mut f = Fixture::new();
    f.lex_jsx_tokens = true;

    f.check_tokens_with_errors(
        b"<hello\\u{2d}world>",
        &[TokenType::Less, TokenType::Identifier, TokenType::Greater],
        |input: PaddedStringView, errors: &Vec<AnyDiag>| {
            qljs_assert_diags!(
                errors,
                input,
                DiagEscapedHyphenNotAllowedInJSXTag {
                    escape_sequence: b"<hello"..b"\\u{2d}",
                },
            );
        },
    );
}

#[test]
fn jsx_string() {
    fn check_string(string_code: &[u8]) {
        scoped_trace!(string_code);

        let mut code_vec: Vec<u8> = vec![b'!'];
        code_vec.extend_from_slice(string_code);
        let code = PaddedString::from_slice(&code_vec[..]);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '!'.

        assert_eq!(l.peek().type_, TokenType::String);

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::EndOfFile);

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    check_string(br#"'hello'"#);
    check_string(br#""hello""#);

    // Backslashes are ignored.
    check_string(br#""hello\nworld""#);
    check_string(br#""hello\""#);
    check_string(br#""hello\\""#);
    check_string(br#"'hello\'"#);
    check_string(br#"'hello\\'"#);
    check_string(br#"'hello\u{}world'"#);
    check_string(br#"'hello\u00xyworld'"#);
    check_string(br#"'hello\u00'"#);

    // Null bytes are allowed.
    check_string(b"'hello\0world'");
    check_string(b"'\0world'");
    check_string(b"'hello\0'");

    // Line terminators are allowed.
    for line_terminator in LINE_TERMINATORS {
        check_string(format!("'hello{line_terminator}world'").as_bytes());
    }
}

// Despite what the JSX specification says, comments are not interpreted in
// attribute strings.
// https://github.com/facebook/jsx/pull/133
#[test]
fn jsx_string_ignores_comments() {
    {
        let code = PaddedString::from_slice(b"! 'hello // '\nworld'");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '!'.

        assert_eq!(l.peek().type_, TokenType::String);
        assert_eq!(l.peek().begin, unsafe { code.c_str().add(b"! ".len()) });
        assert_eq!(
            l.peek().end,
            unsafe { code.c_str().add(b"! 'hello // '".len()) },
            "string should end at ', treating // as part of the string"
        );

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"world");

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    {
        let code = PaddedString::from_slice(br#"! "hello/* not"comment */world""#);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '!'.

        assert_eq!(l.peek().type_, TokenType::String);
        assert_eq!(l.peek().begin, unsafe { code.c_str().add(b"! ".len()) });
        assert_eq!(
            l.peek().end,
            unsafe { code.c_str().add(br#"! "hello/* not""#.len()) },
            "string should end at \", treating /* as part of the string"
        );

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"comment");

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }
}

#[test]
fn unterminated_jsx_string() {
    let code = PaddedString::from_slice(b"! 'hello");
    let errors = DiagCollector::new();
    let allocator = LexerAllocator::new();
    let mut l = Lexer::new(code.view(), &errors, &allocator);
    l.skip_in_jsx(); // Ignore '!'.

    assert_eq!(l.peek().type_, TokenType::String);
    qljs_assert_diags!(
        errors.clone_errors(),
        code.view(),
        DiagUnclosedJSXStringLiteral {
            string_literal_begin: b"! "..b"'",
        },
    );

    l.skip_in_jsx();
    assert_eq!(l.peek().type_, TokenType::EndOfFile);
}

#[test]
fn jsx_tag() {
    {
        let code = PaddedString::from_slice(b"<svg:rect>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"svg");

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Colon);

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"rect");

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    {
        let code = PaddedString::from_slice(b"<myModule.MyComponent>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"myModule");

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Dot);

        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        assert_eq!(l.peek().identifier_name().normalized_name(), b"MyComponent");

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }
}

#[test]
fn jsx_text_children() {
    {
        let code = PaddedString::from_slice(b"<>hello world");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        l.skip_in_jsx_children(); // Skip '>'.

        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    {
        let code = PaddedString::from_slice(b"<>hello</>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        l.skip_in_jsx_children(); // Skip '>'.

        assert_eq!(l.peek().type_, TokenType::Less);
        assert_eq!(l.peek().begin, unsafe {
            code.c_str().add(b"<>hello".len())
        });
        assert_eq!(l.peek().end, unsafe { code.c_str().add(b"<>hello<".len()) });
        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }

    // '>=' might be interpreted as greater_equal by a buggy lexer, for example.
    for text_begin in ["=", ">", ">>", ">=", ">>="] {
        let code = PaddedString::from_slice(format!("<>{text_begin}hello").as_bytes());
        scoped_trace!(code);
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        assert_eq!(l.peek().type_, TokenType::Greater);
        assert_eq!(l.peek().begin, unsafe { code.c_str().add(b"<".len()) });
        assert_eq!(l.peek().end, unsafe { code.c_str().add(b"<>".len()) });
        l.skip_in_jsx_children();

        assert_eq!(l.peek().type_, TokenType::EndOfFile);
        if text_begin == "=" {
            qljs_assert_no_diags!(errors.clone_errors(), code.view());
        } else if text_begin == ">" || text_begin == ">=" {
            qljs_assert_diags!(
                errors.clone_errors(),
                code.view(),
                DiagUnexpectedGreaterInJSXText {
                    greater: b"<>"..b">",
                },
            );
        } else if text_begin == ">>" || text_begin == ">>=" {
            qljs_assert_diags!(
                errors.clone_errors(),
                code.view(),
                DiagUnexpectedGreaterInJSXText {
                    greater: b"<>"..b">",
                },
                DiagUnexpectedGreaterInJSXText {
                    greater: b"<>>"..b">",
                },
            );
        } else {
            unreachable!();
        }
    }
}

#[test]
fn jsx_illegal_text_children() {
    {
        let code = PaddedString::from_slice(b"<>hello>world</>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        l.skip_in_jsx_children(); // Skip '>'.
        assert_eq!(l.peek().type_, TokenType::Less);
        qljs_assert_diags!(
            errors.clone_errors(),
            code.view(),
            DiagUnexpectedGreaterInJSXText {
                greater: b"<>hello"..b">",
            },
        );
    }

    {
        let code = PaddedString::from_slice(b"<>hello}world</>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        l.skip_in_jsx(); // Ignore '<'.

        l.skip_in_jsx_children(); // Skip '>'.
        assert_eq!(l.peek().type_, TokenType::Less);
        qljs_assert_diags!(
            errors.clone_errors(),
            code.view(),
            DiagUnexpectedRightCurlyInJSXText {
                right_curly: b"<>hello"..b"}",
            },
        );
    }
}

#[test]
fn jsx_expression_children() {
    let mut f = Fixture::new();

    f.lex_jsx_tokens = true;

    {
        let code = PaddedString::from_slice(b"<>hello {name}!</>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);

        // <>hello
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);

        // {name}
        l.skip_in_jsx_children();
        assert_eq!(l.peek().type_, TokenType::LeftCurly);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip();
        assert_eq!(l.peek().type_, TokenType::RightCurly);

        // !</>
        l.skip_in_jsx_children();
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }
}

#[test]
fn jsx_nested_children() {
    let mut f = Fixture::new();

    f.lex_jsx_tokens = true;

    {
        let code = PaddedString::from_slice(b"<>hello <span>world</span>!</>");
        let errors = DiagCollector::new();
        let allocator = LexerAllocator::new();
        let mut l = Lexer::new(code.view(), &errors, &allocator);
        // <>hello
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);

        // <span>world</span>
        l.skip_in_jsx_children();
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);
        l.skip_in_jsx_children();
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Identifier);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);

        // !</>
        l.skip_in_jsx_children();
        assert_eq!(l.peek().type_, TokenType::Less);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Slash);
        l.skip_in_jsx();
        assert_eq!(l.peek().type_, TokenType::Greater);

        qljs_assert_no_diags!(errors.clone_errors(), code.view());
    }
}

struct Fixture {
    lex_jsx_tokens: bool,
}

impl Fixture {
    fn new() -> Fixture {
        Fixture {
            lex_jsx_tokens: false,
        }
    }

    fn check_single_token(&mut self, input: &[u8], expected_identifier_name: &[u8]) {
        self.check_single_token_with_errors(
            input,
            expected_identifier_name,
            |_code: PaddedStringView, errors: &Vec<AnyDiag>| {
                assert_matches!(errors, e if e.is_empty());
            },
        );
    }

    fn check_single_token_with_errors(
        &mut self,
        input: &[u8],
        expected_identifier_name: &[u8],
        check_errors: fn(PaddedStringView, &Vec<AnyDiag>),
    ) {
        let code = PaddedString::from_slice(input);
        let allocator = LexerAllocator::new();
        let errors = DiagCollector::new();
        self.lex_to_eof(
            code.view(),
            &allocator,
            &errors,
            |lexed_tokens: &Vec<Token>| {
                assert_matches!(lexed_tokens.as_slice(),
                [t] if t.type_ == TokenType::Identifier || t.type_ == TokenType::PrivateIdentifier);
                assert_eq!(
                    lexed_tokens[0].identifier_name().normalized_name(),
                    expected_identifier_name,
                );
                check_errors(code.view(), &errors.clone_errors());
            },
        );
    }

    fn check_tokens(&mut self, input: &[u8], expected_token_types: &[TokenType]) {
        self.check_tokens_with_errors(
            input,
            expected_token_types,
            |_code: PaddedStringView, errors: &Vec<AnyDiag>| {
                assert_matches!(errors, e if e.is_empty());
            },
        );
    }

    fn check_tokens_with_errors(
        &mut self,
        input: &[u8],
        expected_token_types: &[TokenType],
        check_errors: fn(PaddedStringView, &Vec<AnyDiag>),
    ) {
        let input = PaddedString::from_slice(input);
        let allocator = LexerAllocator::new();
        let errors = DiagCollector::new();
        self.lex_to_eof(
            input.view(),
            &allocator,
            &errors,
            |lexed_tokens: &Vec<Token>| {
                let lexed_token_types: Vec<TokenType> =
                    lexed_tokens.iter().map(|t| t.type_).collect();

                assert_eq!(lexed_token_types, expected_token_types.to_vec());
                check_errors(input.view(), &errors.clone_errors());
            },
        );
    }

    // TODO(port-later): Redesign this interface. It's grown out of hand due to Rust quirks.
    fn lex_to_eof<
        'alloc: 'reporter,
        'code: 'alloc,
        'reporter: 'code,
        Callback: for<'alloc2> FnOnce(&'_ Vec<Token<'alloc2, 'code>>),
    >(
        &mut self,
        input: PaddedStringView<'code>,
        allocator: &'alloc LexerAllocator,
        errors: &'reporter DiagCollector<'code>,
        callback: Callback,
    ) {
        let mut l: Lexer<'alloc, 'code, 'reporter> = Lexer::new(input, errors, allocator);
        let mut tokens: Vec<Token<'_, 'code>> = vec![];
        while l.peek().type_ != TokenType::EndOfFile {
            let t: &Token<'_, 'code> = l.peek();
            // HACK(strager): Rust doesn't know that Token::normalized_identifier and other fields
            // won't be corrupted if we later mutate the Lexer. Work around lifetime issues with
            // some reference transmutation.
            tokens.push(unsafe { std::mem::transmute::<_, &Token>(t) }.clone());
            if self.lex_jsx_tokens {
                l.skip_in_jsx();
            } else {
                l.skip();
            }
        }
        callback(&tokens);
    }

    fn lex_to_eof_types(&mut self, input: &[u8]) -> Vec<TokenType> {
        self.lex_to_eof_types_padded(PaddedString::from_slice(input).view())
    }

    fn lex_to_eof_types_padded(&mut self, input: PaddedStringView<'_>) -> Vec<TokenType> {
        let allocator = LexerAllocator::new();
        let errors = DiagCollector::new();
        let mut lexed_token_types: Vec<TokenType> = vec![];
        self.lex_to_eof(input, &allocator, &errors, |lexed_tokens: &Vec<Token>| {
            for t in lexed_tokens {
                lexed_token_types.push(t.type_);
            }
            assert_eq!(errors.len(), 0);
        });
        lexed_token_types
    }
}
