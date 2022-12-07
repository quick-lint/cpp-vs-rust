use cpp_vs_rust::assert_matches;
use cpp_vs_rust::container::padded_string::*;
use cpp_vs_rust::fe::diagnostic_types::*;
use cpp_vs_rust::fe::lex::*;
use cpp_vs_rust::fe::token::*;
use cpp_vs_rust::test::characters::*;
use cpp_vs_rust::test::diag_collector::*;

macro_rules! scoped_trace {
    ($expr:expr $(,)?) => {
        // TODO(port): SCOPED_TRACE from Google Test.
    };
}

// TODO(port): lex_block_comments
// TODO(port): lex_unopened_block_comment
// TODO(port): lex_regexp_literal_starting_with_star_slash
// TODO(port): lex_regexp_literal_starting_with_star_star_slash

#[test]
fn lex_line_comments() {
    let mut f = Fixture::new();

    assert_matches!(f.lex_to_eof_types("// hello"), ts if ts.is_empty());
    for line_terminator in LINE_TERMINATORS {
        f.check_single_token(&format!("// hello{line_terminator}world"), "world");
    }
    assert_matches!(f.lex_to_eof_types("// hello\n// world"), ts if ts.is_empty());
    f.check_tokens(
        "hello//*/\n \n \nworld",
        &[TokenType::Identifier, TokenType::Identifier],
    );

    /*
     * Also test for a unicode sign that starts with 0xe280, because the
     * skip_line_comment() will also look for U+2028 and U+2029
     *  > U+2028 Line Separator      (0xe280a8)
     *  > U+2029 Paragraph Separator (0xe280a9)
     *  > U+2030 Per Mille Sign      (0xe280b0)
     */
    assert_matches!(f.lex_to_eof_types("// 123‰"), ts if ts.is_empty());
}

#[test]
fn lex_line_comments_with_control_characters() {
    let mut f = Fixture::new();
    for control_character in CONTROL_CHARACTERS_EXCEPT_LINE_TERMINATORS {
        // TODO(port): Change this back to "42.0".
        let input: String = format!("// hello {control_character} world\nident");
        scoped_trace!(input);
        f.check_tokens(&input, &[TokenType::Identifier]);
    }
}

// TODO(port): lex_html_open_comments
// TODO(port): lex_html_close_comments
// TODO(port): lex_numbers
// TODO(port): lex_binary_numbers
// TODO(port): fail_lex_integer_loses_precision
// TODO(port): fail_lex_binary_number_no_digits
// TODO(port): fail_lex_binary_number
// TODO(port): lex_modern_octal_numbers
// TODO(port): fail_lex_modern_octal_number_no_digits
// TODO(port): fail_lex_modern_octal_numbers
// TODO(port): lex_legacy_octal_numbers_strict
// TODO(port): lex_legacy_octal_numbers_lax
// TODO(port): fail_lex_legacy_octal_numbers
// TODO(port): legacy_octal_numbers_cannot_contain_underscores
// TODO(port): lex_hex_numbers
// TODO(port): fail_lex_hex_number_no_digits
// TODO(port): fail_lex_hex_number
// TODO(port): lex_number_with_trailing_garbage
// TODO(port): lex_decimal_number_with_dot_method_call_is_invalid
// TODO(port): lex_invalid_big_int_number
// TODO(port): lex_number_with_double_underscore
// TODO(port): lex_number_with_many_underscores
// TODO(port): lex_number_with_multiple_groups_of_consecutive_underscores
// TODO(port): lex_number_with_trailing_underscore
// TODO(port): lex_number_with_trailing_underscores
// TODO(port): lex_strings
// TODO(port): lex_string_with_ascii_control_characters
// TODO(port): string_with_curly_quotes
// TODO(port): lex_templates
// TODO(port): templates_buffer_unicode_escape_errors
// TODO(port): templates_do_not_buffer_valid_unicode_escapes
// TODO(port): lex_template_literal_with_ascii_control_characters
// TODO(port): lex_regular_expression_literals
// TODO(port): lex_regular_expression_literal_with_digit_flag
// TODO(port): lex_unicode_escape_in_regular_expression_literal_flags
// TODO(port): lex_non_ascii_in_regular_expression_literal_flags
// TODO(port): lex_regular_expression_literals_preserves_leading_newline_flag
// TODO(port): lex_regular_expression_literal_with_ascii_control_characters
// TODO(port): split_less_less_into_two_tokens
// TODO(port): split_less_less_has_no_leading_newline
// TODO(port): split_greater_from_bigger_token
// TODO(port): split_greater_from_bigger_token_has_no_leading_newline

#[test]
fn lex_identifiers() {
    let mut f = Fixture::new();
    f.check_tokens("i", &[TokenType::Identifier]);
    f.check_tokens("_", &[TokenType::Identifier]);
    f.check_tokens("$", &[TokenType::Identifier]);
    f.check_single_token("id", "id");
    f.check_single_token("id ", "id");
    f.check_single_token("this_is_an_identifier", "this_is_an_identifier");
    f.check_single_token("MixedCaseIsAllowed", "MixedCaseIsAllowed");
    f.check_single_token("ident$with$dollars", "ident$with$dollars");
    f.check_single_token("digits0123456789", "digits0123456789");
}

// TODO(port): ascii_identifier_with_escape_sequence
// TODO(port): non_ascii_identifier
// TODO(port): non_ascii_identifier_with_escape_sequence
// TODO(port): identifier_with_escape_sequences_source_code_span_is_in_place
// TODO(port): lex_identifier_with_malformed_escape_sequence
// TODO(port): lex_identifier_with_out_of_range_escaped_character
// TODO(port): lex_identifier_with_out_of_range_utf_8_sequence
// TODO(port): lex_identifier_with_malformed_utf_8_sequence
// TODO(port): lex_identifier_with_disallowed_character_escape_sequence
// TODO(port): lex_identifier_with_disallowed_non_ascii_character
// TODO(port): lex_identifier_with_disallowed_escaped_initial_character
// TODO(port): lex_identifier_with_disallowed_non_ascii_initial_character
// TODO(port): lex_identifier_with_disallowed_initial_character_as_subsequent_character
// TODO(port): lex_identifiers_which_look_like_keywords
// TODO(port): private_identifier
// TODO(port): private_identifier_with_disallowed_non_ascii_initial_character
// TODO(port): private_identifier_with_disallowed_escaped_initial_character
// TODO(port): lex_reserved_keywords
// TODO(port): lex_contextual_keywords
// TODO(port): lex_typescript_contextual_keywords
// TODO(port): lex_reserved_keywords_except_await_and_yield_sometimes_cannot_contain_escape_sequences
// TODO(port): lex_contextual_keywords_and_await_and_yield_can_contain_escape_sequences

#[test]
fn lex_single_character_symbols() {
    let mut f = Fixture::new();
    f.check_tokens("+", &[TokenType::Plus]);
    f.check_tokens("-", &[TokenType::Minus]);
    f.check_tokens("*", &[TokenType::Star]);
    f.check_tokens("/", &[TokenType::Slash]);
    f.check_tokens("<", &[TokenType::Less]);
    f.check_tokens(">", &[TokenType::Greater]);
    f.check_tokens("=", &[TokenType::Equal]);
    f.check_tokens("&", &[TokenType::Ampersand]);
    f.check_tokens("^", &[TokenType::Circumflex]);
    f.check_tokens("!", &[TokenType::Bang]);
    f.check_tokens(".", &[TokenType::Dot]);
    f.check_tokens(",", &[TokenType::Comma]);
    f.check_tokens("~", &[TokenType::Tilde]);
    f.check_tokens("%", &[TokenType::Percent]);
    f.check_tokens("(", &[TokenType::LeftParen]);
    f.check_tokens(")", &[TokenType::RightParen]);
    f.check_tokens("[", &[TokenType::LeftSquare]);
    f.check_tokens("]", &[TokenType::RightSquare]);
    f.check_tokens("{", &[TokenType::LeftCurly]);
    f.check_tokens("}", &[TokenType::RightCurly]);
    f.check_tokens(":", &[TokenType::Colon]);
    f.check_tokens(";", &[TokenType::Semicolon]);
    f.check_tokens("?", &[TokenType::Question]);
    f.check_tokens("|", &[TokenType::Pipe]);
}

#[test]
fn lex_multi_character_symbols() {
    let mut f = Fixture::new();
    f.check_tokens("<=", &[TokenType::LessEqual]);
    f.check_tokens(">=", &[TokenType::GreaterEqual]);
    f.check_tokens("==", &[TokenType::EqualEqual]);
    f.check_tokens("===", &[TokenType::EqualEqualEqual]);
    f.check_tokens("!=", &[TokenType::BangEqual]);
    f.check_tokens("!==", &[TokenType::BangEqualEqual]);
    f.check_tokens("**", &[TokenType::StarStar]);
    f.check_tokens("++", &[TokenType::PlusPlus]);
    f.check_tokens("--", &[TokenType::MinusMinus]);
    f.check_tokens("<<", &[TokenType::LessLess]);
    f.check_tokens(">>", &[TokenType::GreaterGreater]);
    f.check_tokens(">>>", &[TokenType::GreaterGreaterGreater]);
    f.check_tokens("&&", &[TokenType::AmpersandAmpersand]);
    f.check_tokens("||", &[TokenType::PipePipe]);
    f.check_tokens("+=", &[TokenType::PlusEqual]);
    f.check_tokens("-=", &[TokenType::MinusEqual]);
    f.check_tokens("*=", &[TokenType::StarEqual]);
    f.check_tokens("/=", &[TokenType::SlashEqual]);
    f.check_tokens("%=", &[TokenType::PercentEqual]);
    f.check_tokens("**=", &[TokenType::StarStarEqual]);
    f.check_tokens("&&=", &[TokenType::AmpersandAmpersandEqual]);
    f.check_tokens("&=", &[TokenType::AmpersandEqual]);
    f.check_tokens("?.", &[TokenType::QuestionDot]);
    f.check_tokens("??", &[TokenType::QuestionQuestion]);
    f.check_tokens("??=", &[TokenType::QuestionQuestionEqual]);
    f.check_tokens("^=", &[TokenType::CircumflexEqual]);
    f.check_tokens("|=", &[TokenType::PipeEqual]);
    f.check_tokens("||=", &[TokenType::PipePipeEqual]);
    f.check_tokens("<<=", &[TokenType::LessLessEqual]);
    f.check_tokens(">>=", &[TokenType::GreaterGreaterEqual]);
    f.check_tokens(">>>=", &[TokenType::GreaterGreaterGreaterEqual]);
    f.check_tokens("=>", &[TokenType::EqualGreater]);
    f.check_tokens("...", &[TokenType::DotDotDot]);
}

#[test]
fn lex_adjacent_symbols() {
    let mut f = Fixture::new();
    f.check_tokens("{}", &[TokenType::LeftCurly, TokenType::RightCurly]);
    f.check_tokens("[]", &[TokenType::LeftSquare, TokenType::RightSquare]);
    f.check_tokens("/!", &[TokenType::Slash, TokenType::Bang]);
    f.check_tokens("*==", &[TokenType::StarEqual, TokenType::Equal]);
    f.check_tokens("^>>", &[TokenType::Circumflex, TokenType::GreaterGreater]);
}

#[test]
fn lex_symbols_separated_by_whitespace() {
    let mut f = Fixture::new();
    f.check_tokens("{ }", &[TokenType::LeftCurly, TokenType::RightCurly]);
    f.check_tokens("< =", &[TokenType::Less, TokenType::Equal]);
    f.check_tokens("? .", &[TokenType::Question, TokenType::Dot]);
    f.check_tokens(". . .", &[TokenType::Dot, TokenType::Dot, TokenType::Dot]);
}

// TODO(port): question_followed_by_number_is_not_question_dot
// TODO(port): question_dot_followed_by_non_digit_is_question_dot

#[test]
#[allow(unused_mut, unused_variables)] // TODO(port): Delete.
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
            f.check_tokens(&input, &[TokenType::Identifier, TokenType::Identifier]);
        }

        {
            let input: String = format!("{whitespace}10{whitespace}'hi'{whitespace}");
            scoped_trace!(input);
            // TODO(port): f.check_tokens(&input, &[TokenType::Number, TokenType::String]);
        }

        {
            let input: String = format!("async{whitespace}function{whitespace}");
            scoped_trace!(input);
            // f.check_tokens(&input, &[TokenType::KWAsync, TokenType::KWFunction]);
        }
    }
}

// TODO(port): lex_shebang
// TODO(port): lex_not_shebang
// TODO(port): lex_unexpected_bom_before_shebang
// TODO(port): lex_invalid_common_characters_are_disallowed
// TODO(port): ascii_control_characters_are_disallowed
// TODO(port): ascii_control_characters_sorta_treated_like_whitespace
// TODO(port): lex_token_notes_leading_newline
// TODO(port): lex_token_notes_leading_newline_after_single_line_comment
// TODO(port): lex_token_notes_leading_newline_after_comment_with_newline
// TODO(port): lex_token_notes_leading_newline_after_comment
// TODO(port): inserting_semicolon_at_newline_remembers_next_token
// TODO(port): insert_semicolon_at_beginning_of_input
// TODO(port): inserting_semicolon_at_right_curly_remembers_next_token
// TODO(port): transaction_buffers_errors_until_commit
// TODO(port): nested_transaction_buffers_errors_until_outer_commit
// TODO(port): rolled_back_inner_transaction_discards_errors
// TODO(port): rolled_back_outer_transaction_discards_errors
// TODO(port): errors_after_transaction_commit_are_reported_unbuffered
// TODO(port): errors_after_transaction_rollback_are_reported_unbuffered
// TODO(port): rolling_back_transaction
// TODO(port): insert_semicolon_after_rolling_back_transaction
// TODO(port): unfinished_transaction_does_not_leak_memory
// TODO(port): is_initial_identifier_byte_agrees_with_is_initial_identifier_character
// TODO(port): is_identifier_byte_agrees_with_is_identifier_character
// TODO(port): jsx_identifier
// TODO(port): invalid_jsx_identifier
// TODO(port): jsx_string
// TODO(port): jsx_string_ignores_comments
// TODO(port): unterminated_jsx_string
// TODO(port): jsx_tag
// TODO(port): jsx_text_children
// TODO(port): jsx_illegal_text_children
// TODO(port): jsx_expression_children
// TODO(port): jsx_nested_children

struct Fixture {
    lex_jsx_tokens: bool,
}

impl Fixture {
    fn new() -> Fixture {
        Fixture {
            lex_jsx_tokens: false,
        }
    }

    fn check_single_token(&mut self, input: &str, expected_identifier_name: &str) {
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
        input: &str,
        expected_identifier_name: &str,
        check_errors: fn(PaddedStringView, &Vec<AnyDiag>),
    ) {
        let code = PaddedString::from_str(input);
        let errors = DiagCollector::new();
        self.lex_to_eof(code.view(), &errors, |lexed_tokens: &Vec<Token>| {
            assert_matches!(lexed_tokens, ts if ts.len() == 1);
            assert_matches!(lexed_tokens[0],
                ref t if t.type_ == TokenType::Identifier || t.type_ == TokenType::PrivateIdentifier);
            assert_eq!(lexed_tokens[0].identifier_name().normalized_name(), expected_identifier_name.as_bytes());
            check_errors(code.view(), &errors.clone_errors());
        });
    }

    fn check_tokens(&mut self, input: &str, expected_token_types: &[TokenType]) {
        let input = PaddedString::from_str(input);
        self.check_tokens_with_errors(
            input.view(),
            expected_token_types,
            |_code: PaddedStringView, errors: &Vec<AnyDiag>| {
                assert_matches!(errors, e if e.is_empty());
            },
        );
    }

    fn check_tokens_with_errors(
        &mut self,
        input: PaddedStringView,
        expected_token_types: &[TokenType],
        check_errors: fn(PaddedStringView, &Vec<AnyDiag>),
    ) {
        let errors = DiagCollector::new();
        self.lex_to_eof(input, &errors, |lexed_tokens: &Vec<Token>| {
            let lexed_token_types: Vec<TokenType> = lexed_tokens.iter().map(|t| t.type_).collect();

            assert_eq!(lexed_token_types, expected_token_types.to_vec());
            check_errors(input, &errors.clone_errors());
        });
    }

    fn lex_to_eof<
        'code,
        'reporter: 'code,
        Callback: for<'lexer> FnOnce(&'lexer Vec<Token<'lexer, 'code>>),
    >(
        &mut self,
        input: PaddedStringView<'code>,
        errors: &'reporter DiagCollector<'code>,
        callback: Callback,
    ) {
        let mut l: Lexer<'code, 'reporter> = Lexer::new(input, errors);
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

    fn lex_to_eof_types(&mut self, input: &str) -> Vec<TokenType> {
        self.lex_to_eof_types_padded(PaddedString::from_str(input).view())
    }

    fn lex_to_eof_types_padded(&mut self, input: PaddedStringView<'_>) -> Vec<TokenType> {
        let errors = DiagCollector::new();
        let mut lexed_token_types: Vec<TokenType> = vec![];
        self.lex_to_eof(input, &errors, |lexed_tokens: &Vec<Token>| {
            for t in lexed_tokens {
                lexed_token_types.push(t.type_);
            }
            assert_eq!(errors.len(), 0);
        });
        lexed_token_types
    }
}
