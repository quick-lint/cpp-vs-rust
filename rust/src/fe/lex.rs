use crate::container::monotonic_allocator::*;
use crate::container::padded_string::*;
use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;
use crate::fe::source_code_span::*;
use crate::fe::token::*;
use crate::port::simd::*;
use crate::qljs_assert;
use crate::qljs_slow_assert;
use crate::util::narrow_cast::*;

macro_rules! qljs_case_identifier_start {
    () => {
      b'\\' | b'$' | b'_' |
      b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'G' |
      b'H' | b'I' | b'J' | b'K' | b'L' | b'M' | b'N' |
      b'O' | b'P' | b'Q' | b'R' | b'S' | b'T' | b'U' |
      b'V' | b'W' | b'X' | b'Y' | b'Z' |
      b'a' | b'b' | b'c' | b'd' | b'e' | b'f' | b'g' |
      b'h' | b'i' | b'j' | b'k' | b'l' | b'm' | b'n' |
      b'o' | b'p' | b'q' | b'r' | b's' | b't' | b'u' |
      b'v' | b'w' | b'x' | b'y' | b'z'
    }
}

macro_rules! qljs_case_octal_digit {
    () => {
        b'0'..=b'7'
    };
}

macro_rules! qljs_case_decimal_digit {
    () => {
        qljs_case_octal_digit!() | b'8'..=b'9'
    };
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum IdentifierKind {
    JavaScript,
    JSX, // Allows '-'.
}

pub struct Lexer<'code, 'reporter> {
    last_token: Token</* HACK(strager) */ 'code, 'code>,
    last_last_token_end: *const u8,
    input: InputPointer,
    diag_reporter: &'reporter dyn DiagReporter,
    original_input: PaddedStringView<'code>,

    allocator: MonotonicAllocator,
    transaction_allocator: MonotonicAllocator,
}

impl<'code, 'reporter> Lexer<'code, 'reporter> {
    pub fn new(
        input: PaddedStringView<'code>,
        diag_reporter: &'reporter dyn DiagReporter,
    ) -> Lexer<'code, 'reporter> {
        let mut lexer = Lexer {
            last_token: Token {
                type_: TokenType::EndOfFile,
                begin: std::ptr::null(),
                end: input.c_str(),
                has_leading_newline: false,
                normalized_identifier: &[],
                extras: TokenExtras { no_data: () },
            },
            last_last_token_end: std::ptr::null(),
            input: InputPointer(input.c_str()),
            diag_reporter: diag_reporter,
            original_input: input,
            allocator: MonotonicAllocator::new("Lexer::allocator"),
            transaction_allocator: MonotonicAllocator::new("Lexer::transaction_allocator"),
        };
        lexer.parse_bom_before_shebang();
        lexer.parse_current_token();
        lexer
    }

    // Return information about the current token.
    pub fn peek<'this>(&'this self) -> &'this Token<'this, 'code> {
        &self.last_token
    }

    // Advance to the next token. Use self.peek() after to observe the next
    // token.
    //
    // This function ignores leading and trailing whitespace and comments.
    //
    // Precondition: self.peek().type_ != TokenType::EndOfFile.
    pub fn skip(&mut self) {
        self.parse_current_token();
    }

    pub fn skip_in_jsx(&mut self) {
        todo!();
    }

    // Returns true if a valid regexp literal is found
    // Precondition: *regexp_begin == '/'
    pub fn test_for_regexp(&self, regexp_begin: *const u8) -> bool {
        todo!();
    }

    fn parse_bom_before_shebang(&mut self) {
        // TODO(port)
    }

    // Skips leading whitespace and comments. Initializes self.last_token and
    // self.last_last_token_end.
    // TODO(port): noinline
    fn parse_current_token(&mut self) {
        self.last_last_token_end = self.last_token.end;
        self.last_token.has_leading_newline = false;
        self.skip_whitespace();

        while !self.try_parse_current_token() {
            // Loop.
        }
    }

    // Does not skip whitespace.
    //
    // Returns false if a comment was found. Returns true if a token or EOF was
    // found.
    //
    // Does not update self.last_last_token_end. Assumes
    // self.last_token.has_leading_newline was previously initialized. Updates
    // self.last_token.begin and other members of self.last_token.
    fn try_parse_current_token(&mut self) -> bool {
        self.last_token.begin = self.input.0;
        match self.input[0] {
            qljs_case_decimal_digit!() => {
                self.last_token.type_ = TokenType::Number;
                if self.input[0] == b'0' {
                    match self.input[1] {
                        b'b' | b'B' => {
                            self.input += 2;
                            self.parse_binary_number();
                        }
                        b'o' | b'O' => {
                            self.input += 2;
                            self.parse_modern_octal_number();
                        }
                        qljs_case_decimal_digit!() => {
                            self.input += 1;
                            self.parse_legacy_octal_number();
                        }
                        b'x' | b'X' => {
                            self.input += 2;
                            self.parse_hexadecimal_number();
                        }
                        _ => {
                            self.parse_number();
                        }
                    }
                } else {
                    self.parse_number();
                }
                self.last_token.end = self.input.0;
            }

            qljs_case_identifier_start!() => {
                let ident: ParsedIdentifier =
                    self.parse_identifier(self.input.0, IdentifierKind::JavaScript);
                self.input = InputPointer(ident.after);
                self.last_token.normalized_identifier = ident.normalized;
                self.last_token.end = ident.after;
                self.last_token.type_ = identifier_token_type(ident.normalized);
                /* TODO(port)
                if ident.escape_sequences && !ident.escape_sequences->empty() {
                    match self.last_token.type_ {
                        TokenType::Identifier => {
                            self.last_token.type_ = TokenType::Identifier;
                        }

                        qljs_case_contextual_keyword!()
                        | TokenType::KWAwait
                        | TokenType::KWYield => {
                            // Escape sequences in identifiers prevent it from becoming a
                            // contextual keyword.
                            self.last_token.type_ = TokenType::Identifier;
                        }

                        qljs_case_strict_only_reserved_keyword!() => {
                            // TODO(#73): Treat 'protected', 'implements', etc. in strict mode as
                            // reserved words.
                            self.last_token.type_ = TokenType::Identifier;
                        }

                        qljs_case_reserved_keyword_except_await_and_yield!() => {
                            // Escape sequences in identifiers prevent it from becoming a reserved
                            // keyword.
                            self.last_token.type_ = TokenType::ReservedKeywordWithEscapeSequence;
                            self.last_token.identifier_escape_sequences = ident.escape_sequences;
                        }

                        _ => {
                            unreachable!();
                        }
                    }
                }
                */
            }

            // TODO(port): default:
            b'(' | b')' | b',' | b':' | b';' | b'[' | b']' | b'{' | b'}' | b'~' => {
                self.last_token.type_ = unsafe { std::mem::transmute(self.input[0]) };
                self.input += 1;
                self.last_token.end = self.input.0;
            }

            b'?' => {
                if self.input[1] == b'?' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::QuestionQuestionEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::QuestionQuestion;
                        self.input += 2;
                    }
                } else if self.input[1] == b'.' {
                    if is_digit(self.input[2]) {
                        // '?.3' is '?' followed by '.3'.
                        self.last_token.type_ = TokenType::Question;
                        self.input += 1;
                    } else {
                        self.last_token.type_ = TokenType::QuestionDot;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Question;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'.' => {
                if self.input[1] == b'.' && self.input[2] == b'.' {
                    self.last_token.type_ = TokenType::DotDotDot;
                    self.input += 3;
                } else if is_digit(self.input[1]) {
                    self.last_token.type_ = TokenType::Number;
                    self.parse_number();
                } else {
                    self.last_token.type_ = TokenType::Dot;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'=' => {
                if self.input[1] == b'=' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::EqualEqualEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::EqualEqual;
                        self.input += 2;
                    }
                } else if self.input[1] == b'>' {
                    self.last_token.type_ = TokenType::EqualGreater;
                    self.input += 2;
                } else {
                    self.last_token.type_ = TokenType::Equal;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'!' => {
                if self.input[1] == b'=' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::BangEqualEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::BangEqual;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Bang;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'<' => {
                if self.input[1] == b'!' && self.input[2] == b'-' && self.input[3] == b'-' {
                    self.input += 4;
                    self.skip_line_comment_body();
                    return false;
                } else if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::LessEqual;
                    self.input += 2;
                } else if self.input[1] == b'<' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::LessLessEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::LessLess;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Less;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'>' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::GreaterEqual;
                    self.input += 2;
                } else if self.input[1] == b'>' {
                    if self.input[2] == b'>' {
                        if self.input[3] == b'=' {
                            self.last_token.type_ = TokenType::GreaterGreaterGreaterEqual;
                            self.input += 4;
                        } else {
                            self.last_token.type_ = TokenType::GreaterGreaterGreater;
                            self.input += 3;
                        }
                    } else if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::GreaterGreaterEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::GreaterGreater;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Greater;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'+' => {
                if self.input[1] == b'+' {
                    self.last_token.type_ = TokenType::PlusPlus;
                    self.input += 2;
                } else if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::PlusEqual;
                    self.input += 2;
                } else {
                    self.last_token.type_ = TokenType::Plus;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'-' => {
                if self.input[1] == b'-' {
                    if self.input[2] == b'>' && self.is_first_token_on_line() {
                        self.input += 3;
                        self.skip_line_comment_body();
                        return false;
                    } else {
                        self.last_token.type_ = TokenType::MinusMinus;
                        self.input += 2;
                    }
                } else if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::MinusEqual;
                    self.input += 2;
                } else {
                    self.last_token.type_ = TokenType::Minus;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'*' => {
                if self.input[1] == b'*' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::StarStarEqual;
                        self.input += 3;
                    } else if self.input[2] == b'/' {
                        let parsed_ok: bool = self.test_for_regexp(&self.input[2]);
                        if !parsed_ok {
                            // We saw '**/'. Emit a '*' token now. Later, we will interpret the
                            // following '*/' as a comment.
                            self.last_token.type_ = TokenType::Star;
                            self.input += 1;
                        } else {
                            self.last_token.type_ = TokenType::StarStar;
                            self.input += 2;
                        }
                    } else {
                        self.last_token.type_ = TokenType::StarStar;
                        self.input += 2;
                    }
                } else if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::StarEqual;
                    self.input += 2;
                } else if self.input[1] == b'/' {
                    let starpos: *const u8 = &self.input[0];
                    let parsed_ok: bool = self.test_for_regexp(&self.input[1]);

                    if !parsed_ok {
                        report(
                            self.diag_reporter,
                            DiagUnopenedBlockComment {
                                comment_close: unsafe {
                                    SourceCodeSpan::new(starpos, &self.input[2])
                                },
                            },
                        );
                        self.input += 2;
                        self.skip_whitespace();
                        return false;
                    } else {
                        self.last_token.type_ = TokenType::Star;
                        self.input += 1;
                    }
                } else {
                    self.last_token.type_ = TokenType::Star;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'/' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::SlashEqual;
                    self.input += 2;
                } else if self.input[1] == b'*' {
                    self.skip_block_comment();
                    return false;
                } else if self.input[1] == b'/' {
                    self.input += 2;
                    self.skip_line_comment_body();
                    return false;
                } else {
                    self.last_token.type_ = TokenType::Slash;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'^' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::CircumflexEqual;
                    self.input += 2;
                } else {
                    self.last_token.type_ = TokenType::Circumflex;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'%' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::PercentEqual;
                    self.input += 2;
                } else {
                    self.last_token.type_ = TokenType::Percent;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'&' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::AmpersandEqual;
                    self.input += 2;
                } else if self.input[1] == b'&' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::AmpersandAmpersandEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::AmpersandAmpersand;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Ampersand;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            b'|' => {
                if self.input[1] == b'=' {
                    self.last_token.type_ = TokenType::PipeEqual;
                    self.input += 2;
                } else if self.input[1] == b'|' {
                    if self.input[2] == b'=' {
                        self.last_token.type_ = TokenType::PipePipeEqual;
                        self.input += 3;
                    } else {
                        self.last_token.type_ = TokenType::PipePipe;
                        self.input += 2;
                    }
                } else {
                    self.last_token.type_ = TokenType::Pipe;
                    self.input += 1;
                }
                self.last_token.end = self.input.0;
            }

            // TODO(port): case '"': case '\'':
            // TODO(port): case '`':
            // TODO(port): case '#':
            b'\0' => {
                if self.is_eof(self.input.0) {
                    self.last_token.type_ = TokenType::EndOfFile;
                    self.last_token.end = self.input.0;
                } else {
                    // TODO(port): fallthrough
                }
            }
            // TODO(port): case '\x01' ... case '\x7f':
            // TODO(port): case '@':
            _ => {
                todo!();
            }
        }

        true
    }

    fn parse_binary_number(&mut self) {
        let mut input: InputPointer = self.input;

        input = InputPointer(self.parse_digits_and_underscores(
            |character: u8| -> bool { is_binary_digit(character) },
            input.0,
        ));
        let found_digits: bool = input.0 != self.input.0;
        if input[0] == b'n' {
            input += 1;
        }

        if found_digits {
            self.input = InputPointer(
                self.check_garbage_in_number_literal::<DiagUnexpectedCharactersInBinaryNumber>(
                    input.0,
                ),
            );
        } else {
            report(
                self.diag_reporter,
                DiagNoDigitsInBinaryNumber {
                    characters: unsafe { SourceCodeSpan::new(self.last_token.begin, input.0) },
                },
            );
            self.input = input;
        }
    }

    // 0775, 09999, 08.24
    fn parse_legacy_octal_number(&mut self) {
        todo!();
    }

    // 0o775, 0o111_555
    fn parse_modern_octal_number(&mut self) {
        todo!();
    }

    fn parse_hexadecimal_number(&mut self) {
        todo!();
    }

    fn check_garbage_in_number_literal<Error>(&mut self, input: *const u8) -> *const u8 {
        // TODO(port)
        input
    }

    fn check_integer_precision_loss(&mut self, number_literal: &[u8]) {
        // TODO(port)
    }

    fn parse_number(&mut self) {
        qljs_slow_assert!(is_digit(self.input[0]) || self.input[0] == b'.');
        let mut input: InputPointer = self.input;
        let number_begin = InputPointer(input.0);

        let consume_garbage = |this: &mut Self, input: &mut InputPointer| {
            let garbage_begin: *const u8 = input.0;
            let garbage_end: *const u8 = this
                .parse_identifier(garbage_begin, IdentifierKind::JavaScript)
                .after;
            report(
                this.diag_reporter,
                DiagUnexpectedCharactersInNumber {
                    characters: unsafe { SourceCodeSpan::new(garbage_begin, garbage_end) },
                },
            );
            *input = InputPointer(garbage_end);
        };

        input = InputPointer(self.parse_decimal_digits_and_underscores(input.0));
        let has_decimal_point: bool = input[0] == b'.';
        if has_decimal_point {
            input += 1;
            input = InputPointer(self.parse_decimal_digits_and_underscores(input.0));
        }
        let has_exponent: bool = input[0] == b'e' || input[0] == b'E';
        if has_exponent {
            let e: InputPointer = input;
            input += 1;
            if input[0] == b'-' || input[0] == b'+' {
                input += 1;
            }
            if is_digit(input[0]) {
                input = InputPointer(self.parse_decimal_digits_and_underscores(input.0));
            } else {
                input = e;
                consume_garbage(self, &mut input);
            }
        }
        let is_bigint: bool = input[0] == b'n';
        if is_bigint {
            input += 1;
            if has_decimal_point {
                report(
                    self.diag_reporter,
                    DiagBigIntLiteralContainsDecimalPoint {
                        where_: unsafe { SourceCodeSpan::new(number_begin.0, input.0) },
                    },
                );
            }
            if has_exponent {
                report(
                    self.diag_reporter,
                    DiagBigIntLiteralContainsExponent {
                        where_: unsafe { SourceCodeSpan::new(number_begin.0, input.0) },
                    },
                );
            }
            qljs_slow_assert!(!(number_begin[0] == b'0' && is_digit(number_begin[1])));
        }
        if !has_decimal_point && !has_exponent && !is_bigint {
            self.check_integer_precision_loss(unsafe {
                slice_from_begin_end(number_begin.0, input.0)
            });
        }

        if matches!(input[0], qljs_case_identifier_start!()) {
            consume_garbage(self, &mut input);
        }
        self.input = input;
    }

    fn parse_digits_and_underscores<Func: FnMut(u8) -> bool>(
        &mut self,
        mut is_valid_digit: Func,
        input: *const u8,
    ) -> *const u8 {
        let mut input = InputPointer(input);
        let mut has_trailing_underscore: bool = false;
        let mut garbage_begin: *const u8 = std::ptr::null();
        while is_valid_digit(input[0]) {
            has_trailing_underscore = false;
            input += 1;
            if input[0] == b'_' {
                garbage_begin = input.0;
                has_trailing_underscore = true;
                input += 1;
                if input[0] == b'_' {
                    has_trailing_underscore = false;

                    while input[0] == b'_' {
                        input += 1;
                    }

                    if is_valid_digit(input[0]) {
                        report(
                            self.diag_reporter,
                            DiagNumberLiteralContainsConsecutiveUnderscores {
                                underscores: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                            },
                        );
                    } else {
                        report(
                            self.diag_reporter,
                            DiagNumberLiteralContainsTrailingUnderscores {
                                underscores: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                            },
                        );
                    }
                }
            }
        }
        if !garbage_begin.is_null() && has_trailing_underscore == true {
            report(
                self.diag_reporter,
                DiagNumberLiteralContainsTrailingUnderscores {
                    underscores: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                },
            );
        }
        input.0
    }

    fn parse_octal_digits(&mut self, input: *const u8) -> *const u8 {
        todo!();
    }

    fn parse_decimal_digits_and_underscores(&mut self, input: *const u8) -> *const u8 {
        self.parse_digits_and_underscores(|character: u8| -> bool { is_digit(character) }, input)
    }

    fn parse_hex_digits_and_underscores(&mut self, input: *const u8) -> *const u8 {
        todo!();
    }

    fn parse_identifier(
        &mut self,
        input: *const u8,
        kind: IdentifierKind,
    ) -> ParsedIdentifier</* HACK(strager) */ 'code, 'code> {
        let begin: *const u8 = input;
        let end: *const u8 = self.parse_identifier_fast_only(input);
        let end_c: u8 = unsafe { *end };
        if end_c == b'\\'
            || (kind == IdentifierKind::JSX && end_c == b'-')
            || !is_ascii_code_unit(end_c)
        {
            self.parse_identifier_slow(end, /*identifier_begin=*/ begin, kind)
        } else {
            ParsedIdentifier {
                after: end,
                normalized: unsafe { slice_from_begin_end(begin, end) },
                escape_sequences: None,
            }
        }
    }

    fn parse_identifier_fast_only(&mut self, input: *const u8) -> *const u8 {
        let mut input = InputPointer(input);
        // TODO(strager): Is the check for '\\' correct?
        qljs_slow_assert!(is_identifier_byte(input[0]) || input[0] == b'\\');

        #[cfg(target_feature = "neon")]
        type CharVector = CharVector16NEON;
        // TODO(port): char_vector_16_wasm_simd128
        #[cfg(target_feature = "sse2")]
        type CharVector = CharVector16SSE2;
        // TODO(port): char_vector_1

        fn count_identifier_characters(chars: CharVector) -> u32 {
            // TODO(port): Test code gen carefully.
            #[cfg(target_feature = "sse4.2")]
            {
                // TODO(port)
                let ranges: __m128i = _mm_setr_epi8(
                    '$', '$', //
                    '_', '_', //
                    '0', '9', //
                    'a', 'z', //
                    'A', 'Z', //
                    // For unused table entries, duplicate a previous entry.
                    // (If we zero-filled, we would match null bytes!)
                    '$', '$', //
                    '$', '$', //
                    '$', '$',
                );
                _mm_cmpistri(
                    ranges,
                    chars.m128i(),
                    _SIDD_CMP_RANGES
                        | _SIDD_LEAST_SIGNIFICANT
                        | _SIDD_NEGATIVE_POLARITY
                        | _SIDD_UBYTE_OPS,
                )
            }
            #[cfg(not(target_feature = "sse4.2"))]
            {
                #[cfg(target_feature = "neon")]
                type BoolVector = BoolVector16NEON;
                // TODO(port): bool_vector_16_wasm_simd128
                #[cfg(target_feature = "sse2")]
                type BoolVector = BoolVector16SSE2;
                // TODO(port): bool_vector_1

                const UPPER_TO_LOWER_MASK: u8 = b'a' - b'A';
                /* TODO(port):
                const_assert!((b'A' | UPPER_TO_LOWER_MASK) == b'a');
                */

                let lower_cased_characters: CharVector =
                    chars | CharVector::repeated(UPPER_TO_LOWER_MASK);
                let is_alpha: BoolVector = (lower_cased_characters
                    .lane_gt(CharVector::repeated(b'a' - 1)))
                    & (lower_cased_characters.lane_lt(CharVector::repeated(b'z' + 1)));
                let is_digit: BoolVector = (chars.lane_gt(CharVector::repeated(b'0' - 1)))
                    & (chars.lane_lt(CharVector::repeated(b'9' + 1)));
                let is_identifier: BoolVector = is_alpha
                    | is_digit
                    | (chars.lane_eq(CharVector::repeated(b'$')))
                    | (chars.lane_eq(CharVector::repeated(b'_')));
                is_identifier.find_first_false()
            }
        }

        let mut is_all_identifier_characters: bool = true;
        while is_all_identifier_characters {
            let chars: CharVector = unsafe { CharVector::load_raw(input.0) };
            let identifier_character_count: usize = count_identifier_characters(chars) as usize;

            for i in 0..identifier_character_count {
                qljs_slow_assert!(is_ascii_code_unit(input[i]));
                qljs_slow_assert!(is_identifier_character(
                    input[i] as u32,
                    IdentifierKind::JavaScript
                ));
            }
            input += identifier_character_count as isize;

            is_all_identifier_characters = identifier_character_count == chars.len();
        }

        input.0
    }

    fn parse_identifier_slow(
        &mut self,
        input: *const u8,
        identifier_begin: *const u8,
        _kind: IdentifierKind,
    ) -> ParsedIdentifier</* HACK(strager) */ 'code, 'code> {
        // TODO(port)
        ParsedIdentifier {
            after: input,
            normalized: unsafe { slice_from_begin_end(identifier_begin, input) },
            escape_sequences: None,
        }
    }

    #[allow(unreachable_code)]
    fn skip_whitespace(&mut self) {
        let mut input: InputPointer = self.input;

        loop {
            let c0: u8 = input[0];
            let c1: u8 = input[1];
            let c2: u8 = input[2];
            if c0 == b' ' || c0 == b'\t' || c0 == 0x0c || c0 == 0x0b {
                input += 1;
                continue;
            } else if c0 == b'\n' || c0 == b'\r' {
                self.last_token.has_leading_newline = true;
                input += 1;
                continue;
            } else if c0 >= 0xc2 {
                // TODO(port): [[unlikely]]
                match c0 {
                    0xe1 => {
                        if c1 == 0x9a && c2 == 0x80 {
                            // U+1680 Ogham Space Mark
                            input += 3;
                            continue;
                        } else {
                            break;
                        }
                    }

                    0xe2 => {
                        if c1 == 0x80 {
                            match c2 {
                                0x80  // U+2000 En Quad
                                | 0x81  // U+2001 Em Quad
                                | 0x82  // U+2002 En Space
                                | 0x83  // U+2003 Em Space
                                | 0x84  // U+2004 Three-Per-Em Space
                                | 0x85  // U+2005 Four-Per-Em Space
                                | 0x86  // U+2006 Six-Per-Em Space
                                | 0x87  // U+2007 Figure Space
                                | 0x88  // U+2008 Punctuation Space
                                | 0x89  // U+2009 Thin Space
                                | 0x8a  // U+200A Hair Space
                                | 0xaf => { // U+202F Narrow No-Break Space (NNBSP)
                                    input += 3;
                                    continue;
                                }

                                0xa8  // U+2028 Line Separator
                                | 0xa9 => { // U+2029 Paragraph Separator
                                    qljs_assert!(newline_character_size(input) == 3);
                                    self.last_token.has_leading_newline = true;
                                    input += 3;
                                    continue;
                                }

                                _ => {
                                    break;
                                }
                            }
                        } else if c1 == 0x81 {
                            if c2 == 0x9f {
                                // U+205F Medium Mathematical Space (MMSP)
                                input += 3;
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    0xe3 => {
                        if c1 == 0x80 && c2 == 0x80 {
                            // U+3000 Ideographic Space
                            input += 3;
                            continue;
                        } else {
                            break;
                        }
                    }

                    0xef => {
                        if c1 == 0xbb && c2 == 0xbf {
                            // U+FEFF Zero Width No-Break Space (BOM, ZWNBSP)
                            input += 3;
                            continue;
                        } else {
                            break;
                        }
                    }

                    0xc2 => {
                        if c1 == 0xa0 {
                            // U+00A0 No-Break Space (NBSP)
                            input += 2;
                            continue;
                        } else {
                            break;
                        }
                    }

                    _ => {
                        break;
                    }
                }
            } else {
                break;
            }
            unreachable!();
        }

        self.input = input;
    }

    fn skip_block_comment(&mut self) {
        todo!();
    }

    fn skip_line_comment_body(&mut self) {
        #[cfg(target_feature = "neon")]
        type BoolVector = BoolVector16NEON;
        #[cfg(target_feature = "neon")]
        type CharVector = CharVector16NEON;
        // TODO(port): char_vector_16_wasm_simd128, bool_vector_16_wasm_simd128
        #[cfg(target_feature = "sse2")]
        type BoolVector = BoolVector16SSE2;
        #[cfg(target_feature = "sse2")]
        type CharVector = CharVector16SSE2;
        // TODO(port): bool_vector_1, char_vector_1

        let new_line: CharVector = CharVector::repeated(b'\n');
        let carriage_return: CharVector = CharVector::repeated(b'\r');
        let unicode_first_byte: CharVector = CharVector::repeated(0xe2); // U+2028 U+2029
        let zero: CharVector = CharVector::repeated(0);

        loop {
            let chars: CharVector = unsafe { CharVector::load_raw(self.input.0) };

            let matches: BoolVector = chars.lane_eq(new_line)
                | chars.lane_eq(carriage_return)
                | chars.lane_eq(unicode_first_byte)
                | chars.lane_eq(zero);

            let mask: u32 = matches.mask();
            if mask == 0 {
                // nothing found, go to the next chunk
                self.input += matches.len() as isize;
            } else {
                // found an interesting char
                self.input += mask.trailing_zeros() as isize;

                let found_comment_end: bool = {
                    let n: usize = newline_character_size(self.input);

                    if n == 1 {
                        self.input += 1;
                        self.skip_whitespace();
                        true
                    }
                    // U+2028 Line Separator
                    // U+2029 Paragraph Separator
                    else if n == 3 {
                        self.input += 3;
                        self.skip_whitespace();
                        true
                    } else if self.input[0] == b'\0' && self.is_eof(self.input.0) {
                        true
                    } else {
                        self.input += 1;
                        false
                    }
                };
                if found_comment_end {
                    break;
                }
            }
        }

        self.last_token.has_leading_newline = true;
    }

    fn is_eof(&self, input: *const u8) -> bool {
        qljs_assert!(unsafe { *input } == b'\0');
        input == self.original_input.null_terminator()
    }

    fn is_first_token_on_line(&self) -> bool {
        self.last_token.has_leading_newline
            || self.last_last_token_end == self.original_input.c_str()
    }
}

// The result of parsing an identifier.
//
// Typically, .normalized is default-constructed. However, if an identifier
// contains escape squences, then .normalized points to a heap-allocated
// null-terminated string of the unescaped identifier.
//
// Say we are parsing the identifier starting with 'w' in the following
// example:
//
// Input: log(w\u{61}t)
//                    ^
//                    .end
//
// In this case, .end points to the ')' character which follows the
// identifier, and .normalized points to a heap-allocated string u8"wat".
//
// If any escape sequences were parsed, .escape_sequences points to a list of
// escape squence spans.
//
// Invariant:
//   (escape_sequences == nullptr) == (normalized.data() == nullptr)
//
// TODO(port): Update docs for Rustisms.
struct ParsedIdentifier<'alloc, 'code> {
    after: *const u8, // Where to continue parsing.
    normalized: &'alloc [u8],

    escape_sequences: Option<&'alloc EscapeSequenceList<'alloc, 'code>>,
}

fn is_binary_digit(c: u8) -> bool {
    c == b'0' || c == b'1'
}

fn is_digit(c: u8) -> bool {
    matches!(c, qljs_case_decimal_digit!())
}

fn is_initial_identifier_byte(byte: u8) -> bool {
    matches!(byte, qljs_case_identifier_start!() | 0xc2..=0xcb | 0xcd..=0xed | 0xef..=0xf0)
}

fn is_identifier_byte(byte: u8) -> bool {
    matches!(byte, qljs_case_decimal_digit!() | qljs_case_identifier_start!() | 0xc2..=0xed | 0xef..=0xf0 | 0xf3)
}

fn is_identifier_character(code_point: u32, kind: IdentifierKind) -> bool {
    if kind == IdentifierKind::JSX && code_point == (b'-' as u32) {
        return true;
    }
    /* TODO(port)
    look_up_in_unicode_table(identifier_part_chunk_indexes,
                                    identifier_part_chunk_indexes_size,
                                    code_point)
    */
    // HACK(port): Temporary implementation.
    matches!(
        code_point as u8,
        qljs_case_identifier_start!() | b'0'..=b'9'
    )
}

fn is_ascii_code_unit(code_unit: u8) -> bool {
    code_unit < 0x80
}

fn is_ascii_code_point(code_point: u32) -> bool {
    code_point < 0x80
}

fn newline_character_size(input: InputPointer) -> usize {
    if input[0] == b'\n' || input[0] == b'\r' {
        1
    // U+2028 Line Separator
    // U+2029 Paragraph Separator
    } else if input[0] == 0xe2 && input[1] == 0x80 && (input[2] == 0xa8 || input[2] == 0xa9) {
        3
    } else {
        0
    }
}

fn is_newline_character(code_point: u32) -> bool {
    code_point == ('\n' as u32) || code_point == ('\r' as u32) ||
         code_point == 0x2028 ||  // Line Separator
         code_point == 0x2029 // Paragraph Separator
}

// TODO(port)
fn identifier_token_type(identifier: &[u8]) -> TokenType {
    TokenType::Identifier
}

// NOTE(port): This is a transitioning struct to make it easier to port code.
#[derive(Clone, Copy, Eq, PartialEq)]
struct InputPointer(*const u8);

impl std::ops::Index<usize> for InputPointer {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        unsafe { &*self.0.add(index) }
    }
}

impl std::ops::Add<isize> for InputPointer {
    type Output = InputPointer;

    fn add(self, rhs: isize) -> InputPointer {
        InputPointer(unsafe { self.0.offset(rhs) })
    }
}

impl std::ops::AddAssign<isize> for InputPointer {
    fn add_assign(&mut self, rhs: isize) {
        *self = *self + rhs;
    }
}

unsafe fn slice_from_begin_end<'out>(begin: *const u8, end: *const u8) -> &'out [u8] {
    std::slice::from_raw_parts(begin, narrow_cast(end.offset_from(begin)))
}
