use crate::container::linked_bump_allocator::*;
use crate::container::monotonic_allocator::*;
use crate::container::padded_string::*;
use crate::container::vector::*;
use crate::fe::buffering_diag_reporter::*;
use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;
use crate::fe::lex_unicode_generated::*;
use crate::fe::source_code_span::*;
use crate::fe::token::*;
use crate::port::maybe_uninit::*;
use crate::port::simd::*;
use crate::qljs_always_assert;
use crate::qljs_assert;
use crate::qljs_const_assert;
use crate::qljs_slow_assert;
use crate::util::narrow_cast::*;
use crate::util::utf_8::*;

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

macro_rules! qljs_case_newline_start {
    () => {
        b'\n' | b'\r' | LINE_SEPARATOR_PARAGRAPH_SEPARATOR_FIRST_BYTE
    };
}

const LINE_SEPARATOR_PARAGRAPH_SEPARATOR_FIRST_BYTE: u8 = 0xe2;

const LEFT_SINGLE_QUOTE: char = '\u{2018}';
const LEFT_DOUBLE_QUOTE: char = '\u{201c}';
const RIGHT_SINGLE_QUOTE: char = '\u{2019}';
const RIGHT_DOUBLE_QUOTE: char = '\u{201d}';

fn look_up_in_unicode_table(table: &[u8], code_point: u32) -> bool {
    const BITS_PER_BYTE: usize = 8;
    const MAX_CODE_POINT: u32 = '\u{10ffff}' as u32;
    const BITS_PER_CHUNK: usize = UNICODE_TABLE_CHUNK_SIZE;
    const BYTES_PER_CHUNK: usize = BITS_PER_CHUNK / BITS_PER_BYTE;
    type ChunkIndexType = u8;

    qljs_assert!(code_point <= MAX_CODE_POINT);
    let chunk_index_index: usize = (code_point as usize) / BITS_PER_CHUNK;
    if chunk_index_index >= table.len() {
        return false;
    }
    let chunk_index: ChunkIndexType = table[chunk_index_index];

    let bit_in_chunk: usize = (code_point as usize) % BITS_PER_CHUNK;
    let slot: u8 = UNICODE_TABLES_CHUNKS
        [(chunk_index as usize) * BYTES_PER_CHUNK + bit_in_chunk / BITS_PER_BYTE];
    (slot & (1 << (bit_in_chunk % BITS_PER_BYTE))) != 0
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

            // Non-ASCII or control character.
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

            b'"' | b'\'' => {
                self.input = self.parse_string_literal();
                self.last_token.type_ = TokenType::String;
                self.last_token.end = self.input.0;
            }

            b'`' => {
                self.input += 1;
                let body: ParsedTemplateBody =
                    self.parse_template_body(self.input, self.last_token.begin, self.diag_reporter);
                self.last_token.extras.template_escape_sequence_diagnostics =
                    std::mem::ManuallyDrop::new(body.escape_sequence_diagnostics);
                self.last_token.type_ = body.type_;
                self.input = InputPointer(body.end);
                self.last_token.end = self.input.0;
            }

            b'#' => {
                if self.input[1] == b'!' && self.input.0 == self.original_input.c_str() {
                    self.input += 2;
                    self.skip_line_comment_body();
                    return false;
                } else if is_initial_identifier_byte(self.input[1]) {
                    // Private identifier: #alphaNumeric
                    let mut ident: ParsedIdentifier =
                        self.parse_identifier((self.input + 1).0, IdentifierKind::JavaScript);
                    if ident.normalized.as_ptr() == (self.input + 1).0 {
                        // Include the '#'.
                        ident.normalized = unsafe {
                            std::slice::from_raw_parts(self.input.0, ident.normalized.len() + 1)
                        };
                    } else {
                        // parse_identifier called parse_identifier_slow, and it included the
                        // '#' already in normalized_name.
                        qljs_assert!(ident.normalized[0] == b'#');
                    }
                    self.input = InputPointer(ident.after);
                    self.last_token.normalized_identifier = ident.normalized;
                    self.last_token.end = ident.after;
                    self.last_token.type_ = TokenType::PrivateIdentifier;
                } else {
                    report(
                        self.diag_reporter,
                        DiagUnexpectedHashCharacter {
                            where_: unsafe {
                                SourceCodeSpan::new(self.input.0, (self.input + 1).0)
                            },
                        },
                    );
                    self.input += 1;
                    self.skip_whitespace();
                    return false;
                }
            }

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
                let character: DecodeUTF8Result = decode_utf_8(unsafe {
                    PaddedStringView::from_begin_end(
                        self.input.0,
                        self.original_input.null_terminator(),
                    )
                });
                // TODO(port): Clean up this casting.
                if character.code_point == (LEFT_SINGLE_QUOTE as u32)
                    || character.code_point == (RIGHT_SINGLE_QUOTE as u32)
                    || character.code_point == (LEFT_DOUBLE_QUOTE as u32)
                    || character.code_point == (RIGHT_DOUBLE_QUOTE as u32)
                {
                    self.input = InputPointer(self.parse_smart_quote_string_literal(&character));
                    self.last_token.type_ = TokenType::String;
                    self.last_token.end = self.input.0;
                } else {
                    let ident: ParsedIdentifier = self.parse_identifier_slow(
                        self.input.0,
                        self.input.0,
                        IdentifierKind::JavaScript,
                    );
                    self.input = InputPointer(ident.after);
                    self.last_token.normalized_identifier = ident.normalized;
                    self.last_token.end = ident.after;
                    self.last_token.type_ = TokenType::Identifier;
                }
            }
        }

        true
    }

    fn parse_string_literal(&mut self) -> InputPointer {
        let opening_quote: u8 = self.input[0];

        let mut c: InputPointer = self.input + 1;
        loop {
            match c[0] {
                b'\0' => {
                    if self.is_eof(c.0) {
                        report(
                            self.diag_reporter,
                            DiagUnclosedStringLiteral {
                                string_literal: unsafe { SourceCodeSpan::new(self.input.0, c.0) },
                            },
                        );
                        return c;
                    } else {
                        c += 1;
                    }
                }

                b'\n' | b'\r' => {
                    let mut matching_quote: *const u8 = std::ptr::null();
                    let mut current_c: InputPointer = c;
                    if current_c[0] == b'\r' && current_c[1] == b'\n' {
                        current_c += 2;
                    } else {
                        current_c += 1;
                    }
                    loop {
                        if current_c[0] == opening_quote {
                            if !matching_quote.is_null() {
                                break;
                            }
                            matching_quote = current_c.0;
                            current_c += 1;
                        } else if current_c[0] == b'\r'
                            || current_c[0] == b'\n'
                            || (current_c[0] == b'\0' && self.is_eof(current_c.0))
                        {
                            if !matching_quote.is_null() {
                                c = InputPointer(matching_quote) + 1;
                            }
                            break;
                        } else {
                            current_c += 1;
                        }
                    }
                    report(
                        self.diag_reporter,
                        DiagUnclosedStringLiteral {
                            string_literal: unsafe { SourceCodeSpan::new(self.input.0, c.0) },
                        },
                    );
                    return c;
                }

                b'\\' => {
                    let escape_sequence_start: *const u8 = c.0;
                    c += 1;
                    match c[0] {
                        b'\0' => {
                            if self.is_eof(c.0) {
                                report(
                                    self.diag_reporter,
                                    DiagUnclosedStringLiteral {
                                        string_literal: unsafe {
                                            SourceCodeSpan::new(self.input.0, c.0)
                                        },
                                    },
                                );
                                return c;
                            } else {
                                c += 1;
                            }
                        }
                        b'\r' => {
                            c += 1;
                            if c[0] == b'\n' {
                                c += 1;
                            }
                        }
                        b'x' => {
                            c += 1;
                            for i in 0..2 {
                                if !is_hex_digit(c[i]) {
                                    report(
                                        self.diag_reporter,
                                        DiagInvalidHexEscapeSequence {
                                            escape_sequence: unsafe {
                                                SourceCodeSpan::new(escape_sequence_start, c.0)
                                            },
                                        },
                                    );
                                    break;
                                }
                            }
                        }
                        b'u' => {
                            c = InputPointer(
                                self.parse_unicode_escape(
                                    escape_sequence_start,
                                    self.diag_reporter,
                                )
                                .end,
                            );
                        }
                        _ => {
                            c += 1;
                        }
                    }
                }

                b'"' | b'\'' => {
                    if c[0] == opening_quote {
                        c += 1;
                        return c;
                    }
                    c += 1;
                }

                _ => {
                    c += 1;
                }
            }
        }
    }

    fn parse_smart_quote_string_literal(&mut self, opening_quote: &DecodeUTF8Result) -> *const u8 {
        qljs_assert!(opening_quote.ok);
        // TODO(port): Clean up this casting.
        qljs_assert!(
            opening_quote.code_point == (LEFT_SINGLE_QUOTE as u32)
                || opening_quote.code_point == (RIGHT_SINGLE_QUOTE as u32)
                || opening_quote.code_point == (LEFT_DOUBLE_QUOTE as u32)
                || opening_quote.code_point == (RIGHT_DOUBLE_QUOTE as u32)
        );
        let opening_quote_begin: InputPointer = self.input;
        let opening_quote_end: InputPointer =
            opening_quote_begin + narrow_cast::<isize, _>(opening_quote.size);

        // TODO(port): Clean up this casting.
        let is_double_quote: bool = opening_quote.code_point == (LEFT_DOUBLE_QUOTE as u32)
            || opening_quote.code_point == (RIGHT_DOUBLE_QUOTE as u32);
        report(
            self.diag_reporter,
            DiagInvalidQuotesAroundStringLiteral {
                opening_quote: unsafe {
                    SourceCodeSpan::new(opening_quote_begin.0, opening_quote_end.0)
                },
                suggested_quote: if is_double_quote { b'"' } else { b'\'' },
            },
        );

        const DOUBLE_ENDING_QUOTES: [char; 3] = ['"', LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE];
        const SINGLE_ENDING_QUOTES: [char; 3] = ['\'', LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE];
        let ending_quotes: &[char; 3] = if is_double_quote {
            &DOUBLE_ENDING_QUOTES
        } else {
            &SINGLE_ENDING_QUOTES
        };
        let is_ending_quote = |code_point: char| -> bool {
            qljs_const_assert!(DOUBLE_ENDING_QUOTES.len() == SINGLE_ENDING_QUOTES.len());
            ending_quotes.contains(&code_point)
        };

        let mut c: InputPointer = opening_quote_end;
        loop {
            let decoded: DecodeUTF8Result = decode_utf_8(unsafe {
                PaddedStringView::from_begin_end(c.0, self.original_input.null_terminator())
            });
            if decoded.ok {
                // TODO(port): Clean up this casting.
                if is_ending_quote(unsafe { std::char::from_u32_unchecked(decoded.code_point) }) {
                    return (c + narrow_cast::<isize, _>(decoded.size)).0;
                }
                if is_newline_character(decoded.code_point) {
                    report(
                        self.diag_reporter,
                        DiagUnclosedStringLiteral {
                            string_literal: unsafe {
                                SourceCodeSpan::new(opening_quote_begin.0, c.0)
                            },
                        },
                    );
                    return c.0;
                }
            }
            if c[0] == b'\0' && self.is_eof(c.0) {
                report(
                    self.diag_reporter,
                    DiagUnclosedStringLiteral {
                        string_literal: unsafe { SourceCodeSpan::new(opening_quote_begin.0, c.0) },
                    },
                );
                return c.0;
            }
            c += narrow_cast::<isize, _>(decoded.size);
            // Loop.
        }
    }

    pub fn skip_in_template(&mut self, template_begin: *const u8) {
        qljs_assert!(self.peek().type_ == TokenType::RightCurly);
        self.last_token.begin = self.input.0;
        let body: ParsedTemplateBody =
            self.parse_template_body(self.input, template_begin, self.diag_reporter);
        self.last_token.type_ = body.type_;
        self.last_token.extras.template_escape_sequence_diagnostics =
            std::mem::ManuallyDrop::new(body.escape_sequence_diagnostics);
        self.input = InputPointer(body.end);
        self.last_token.end = body.end;
    }

    fn parse_template_body<'this>(
        &mut self,
        input: InputPointer,
        template_begin: *const u8,
        diag_reporter: &dyn DiagReporter,
    ) -> ParsedTemplateBody</* HACK(strager) */ 'code, 'code> {
        let mut escape_sequence_diagnostics: Option<
            & /* HACK(strager) */ 'code mut BufferingDiagReporter,
        > = None;
        let mut c: InputPointer = input;
        loop {
            match c[0] {
                b'\0' => {
                    if self.is_eof(c.0) {
                        report(
                            diag_reporter,
                            DiagUnclosedTemplate {
                                incomplete_template: unsafe {
                                    SourceCodeSpan::new(template_begin, c.0)
                                },
                            },
                        );
                        return ParsedTemplateBody {
                            type_: TokenType::CompleteTemplate,
                            end: c.0,
                            escape_sequence_diagnostics: escape_sequence_diagnostics,
                        };
                    } else {
                        c += 1;
                    }
                }

                b'`' => {
                    c += 1;
                    return ParsedTemplateBody {
                        type_: TokenType::CompleteTemplate,
                        end: c.0,
                        escape_sequence_diagnostics: escape_sequence_diagnostics,
                    };
                }

                b'\\' => {
                    let escape_sequence_start: *const u8 = c.0;
                    c += 1;
                    match c[0] {
                        b'\0' => {
                            if self.is_eof(c.0) {
                                report(
                                    diag_reporter,
                                    DiagUnclosedTemplate {
                                        incomplete_template: unsafe {
                                            SourceCodeSpan::new(template_begin, c.0)
                                        },
                                    },
                                );
                                return ParsedTemplateBody {
                                    type_: TokenType::CompleteTemplate,
                                    end: c.0,
                                    escape_sequence_diagnostics: escape_sequence_diagnostics,
                                };
                            } else {
                                c += 1;
                            }
                        }
                        b'u' => {
                            if escape_sequence_diagnostics.is_none() {
                                escape_sequence_diagnostics = Some(unsafe {
                                    &mut *self.allocator.new_object(BufferingDiagReporter::new(
                                        self.get_allocator(),
                                    ))
                                });
                            }
                            let inner_reporter: &mut BufferingDiagReporter =
                                unsafe { escape_sequence_diagnostics.as_mut().unwrap_unchecked() };
                            c = InputPointer(
                                self.parse_unicode_escape(escape_sequence_start, inner_reporter)
                                    .end,
                            );
                        }
                        _ => {
                            c += 1;
                        }
                    }
                }

                b'$' => {
                    if c[1] == b'{' {
                        c += 2;
                        return ParsedTemplateBody {
                            type_: TokenType::IncompleteTemplate,
                            end: c.0,
                            escape_sequence_diagnostics: escape_sequence_diagnostics,
                        };
                    }
                    c += 1;
                }

                _ => {
                    c += 1;
                }
            }
        }
    }

    // Reparse a '/' or '/=' token as a regular expression literal.
    //
    // Precondition: self.peek().type_ == TokenType::Slash or
    //               TokenType::SlashEqual.
    // Postcondition: self.peek().type_ == TokenType::Regexp.
    pub fn reparse_as_regexp(&mut self) {
        qljs_assert!(
            self.last_token.type_ == TokenType::Slash
                || self.last_token.type_ == TokenType::SlashEqual
        );

        self.input = InputPointer(self.last_token.begin);
        qljs_assert!(self.input[0] == b'/');
        self.last_token.type_ = TokenType::Regexp;

        let mut c: InputPointer = self.input + 1;
        'next: loop {
            match c[0] {
                b'\0' => {
                    if self.is_eof(c.0) {
                        report(
                            self.diag_reporter,
                            DiagUnclosedRegexpLiteral {
                                regexp_literal: unsafe {
                                    SourceCodeSpan::new(self.last_token.begin, c.0)
                                },
                            },
                        );
                        break 'next;
                    } else {
                        c += 1;
                        continue 'next;
                    }
                }

                b'\\' => {
                    c += 1;
                    match c[0] {
                        b'\0' => {
                            if self.is_eof(c.0) {
                                report(
                                    self.diag_reporter,
                                    DiagUnclosedRegexpLiteral {
                                        regexp_literal: unsafe {
                                            SourceCodeSpan::new(self.last_token.begin, c.0)
                                        },
                                    },
                                );
                                break 'next;
                            } else {
                                c += 1;
                                continue 'next;
                            }
                        }

                        _ => {
                            c += 1;
                            continue 'next;
                        }
                    }
                }

                b'[' => {
                    c += 1;
                    loop {
                        match c[0] {
                            b']' | b'\0' => {
                                continue 'next;
                            }

                            b'\\' => {
                                if c[1] == b']' || c[1] == b'\\' {
                                    c += 2;
                                } else {
                                    c += 1;
                                }
                            }

                            qljs_case_newline_start!() => {
                                if newline_character_size(c) != 0 {
                                    continue 'next;
                                }
                                // NOTE(port): This used to be fallthrough.
                                c += 1;
                            }

                            _ => {
                                c += 1;
                            }
                        }
                    }
                }

                b'/' => {
                    c += 1;
                    // TODO(strager): Is the check for '\\' correct?
                    if is_identifier_byte(c[0]) || c[0] == b'\\' {
                        let ident: ParsedIdentifier =
                            self.parse_identifier(c.0, IdentifierKind::JavaScript);
                        c = InputPointer(ident.after);
                        match ident.escape_sequences {
                            Some(escape_sequences) => {
                                for escape_sequence in escape_sequences.as_slice() {
                                    report(
                                        self.diag_reporter,
                                        DiagRegexpLiteralFlagsCannotContainUnicodeEscapes {
                                            escape_sequence: *escape_sequence,
                                        },
                                    );
                                }
                            }
                            None => {}
                        }
                    }
                    break 'next;
                }

                qljs_case_newline_start!() => {
                    if newline_character_size(c) != 0 {
                        report(
                            self.diag_reporter,
                            DiagUnclosedRegexpLiteral {
                                regexp_literal: unsafe {
                                    SourceCodeSpan::new(self.last_token.begin, c.0)
                                },
                            },
                        );
                        break 'next;
                    }
                    // NOTE(port): This used to be fallthrough.
                    c += 1;
                    continue 'next;
                }

                _ => {
                    c += 1;
                    continue 'next;
                }
            }
        }

        self.input = c;
        self.last_token.end = self.input.0;
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
                self.check_garbage_in_number_literal(input.0, |span: SourceCodeSpan| {
                    DiagUnexpectedCharactersInBinaryNumber { characters: span }
                }),
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
        let mut input: InputPointer = self.input;

        loop {
            input = InputPointer(self.parse_octal_digits(input.0));
            if input[0] == b'_' {
                let underscores_start: *const u8 = input.0;
                while input[0] == b'_' {
                    input += 1;
                }
                report(
                    self.diag_reporter,
                    DiagLegacyOctalLiteralMayNotContainUnderscores {
                        underscores: unsafe { SourceCodeSpan::new(underscores_start, input.0) },
                    },
                );
                continue;
            }

            break;
        }

        if is_digit(input[0]) {
            self.input = input;
            self.parse_number();
            return;
        }

        let garbage_begin: *const u8 = input.0;
        let has_decimal_point: bool = input[0] == b'.' && is_digit(input[1]);
        if has_decimal_point {
            input += 1;
            report(
                self.diag_reporter,
                DiagOctalLiteralMayNotHaveDecimal {
                    characters: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                },
            );
            input = InputPointer(self.parse_octal_digits(input.0));
        }
        let has_exponent: bool = input[0] == b'e' || input[0] == b'E';
        if has_exponent {
            input += 1;
            if input[0] == b'-' || input[0] == b'+' {
                input += 1;
            }
            report(
                self.diag_reporter,
                DiagOctalLiteralMayNotHaveExponent {
                    characters: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                },
            );
            input = InputPointer(self.parse_octal_digits(input.0));
        }
        let is_bigint: bool = input[0] == b'n';
        if is_bigint {
            input += 1;
            report(
                self.diag_reporter,
                DiagLegacyOctalLiteralMayNotBeBigInt {
                    characters: unsafe { SourceCodeSpan::new(garbage_begin, input.0) },
                },
            );
            input = InputPointer(self.parse_octal_digits(input.0));
        }

        self.input = InputPointer(
            self.check_garbage_in_number_literal(input.0, |span: SourceCodeSpan| {
                DiagUnexpectedCharactersInOctalNumber { characters: span }
            }),
        );
    }

    // 0o775, 0o111_555
    fn parse_modern_octal_number(&mut self) {
        // TODO(strager): Why does this look different from parse_binary_number and
        // parse_hexadecimal_number? We should probably make them look the same and
        // factor the common structure.

        let mut input: InputPointer = self.input;
        input = InputPointer(self.parse_digits_and_underscores(
            |character: u8| -> bool { is_octal_digit(character) },
            input.0,
        ));
        if input == self.input {
            report(
                self.diag_reporter,
                DiagNoDigitsInOctalNumber {
                    characters: unsafe { SourceCodeSpan::new(self.last_token.begin, input.0) },
                },
            );
            return;
        }
        if input[0] == b'n' {
            input += 1;
        }
        self.input = InputPointer(
            self.check_garbage_in_number_literal(input.0, |span: SourceCodeSpan| {
                DiagUnexpectedCharactersInOctalNumber { characters: span }
            }),
        );
    }

    fn parse_hexadecimal_number(&mut self) {
        let mut input: InputPointer = self.input;

        input = InputPointer(self.parse_hex_digits_and_underscores(input.0));
        let found_digits: bool = input != self.input;
        let is_bigint: bool = input[0] == b'n';
        if is_bigint {
            input += 1;
        }

        if found_digits {
            self.input = InputPointer(
                self.check_garbage_in_number_literal(input.0, |span: SourceCodeSpan| {
                    DiagUnexpectedCharactersInHexNumber { characters: span }
                }),
            );
        } else {
            report(
                self.diag_reporter,
                DiagNoDigitsInHexNumber {
                    characters: unsafe { SourceCodeSpan::new(self.last_token.begin, input.0) },
                },
            );
            self.input = input;
        }
    }

    fn check_garbage_in_number_literal<
        Diag: 'code + HasDiagType,
        MakeError: FnOnce(SourceCodeSpan<'code>) -> Diag,
    >(
        &mut self,
        input: *const u8,
        make_error: MakeError,
    ) -> *const u8 {
        let mut input: InputPointer = InputPointer(input);
        let garbage_begin: *const u8 = input.0;
        loop {
            match input[0] {
                // 0xffffq  // Invalid.
                // 0b0123   // Invalid.
                qljs_case_decimal_digit!() | qljs_case_identifier_start!() => {
                    input += 1;
                }

                // 0b0000.toString()
                // 0b0000.2  // Invalid.
                b'.' => {
                    if is_digit(input[1]) {
                        // 0b0000.2  // Invalid.
                        input += 2;
                    } else {
                        // 0b0000.toString()
                        // 0b0000. 2          // Invalid.
                        break;
                    }
                }

                _ => {
                    break;
                }
            }
        }

        let garbage_end: *const u8 = input.0;
        if garbage_end != garbage_begin {
            report(
                self.diag_reporter,
                make_error(unsafe { SourceCodeSpan::new(garbage_begin, garbage_end) }),
            );
            input = InputPointer(garbage_end);
        }

        input.0
    }

    fn check_integer_precision_loss(&mut self, number_literal: &[u8]) {
        // Any integer which is 15 or fewer digits is guaranteed to be able to be
        // represented accurately without precision loss. This is because Numbers have
        // 53 bits of precision, which is equal to 53 log10(2) ≈ 15.955 decimal digits
        // of precision.
        const GUARANTEED_ACC_LENGTH: usize = 15;
        // There is no integer which can be represented accurately that is greater
        // than 309 digits long. This is because the largest representable Number is
        // equal to 2^1023 × (1 + (1 − 2^−52)) ≈ 1.7976931348623157 × 10^308, which is
        // 309 digits long.
        const MAX_ACC_LENGTH: usize = 309;
        if number_literal.len() <= GUARANTEED_ACC_LENGTH {
            return;
        }
        let mut cleaned_string: Vec<u8> = Vec::new();
        for c in number_literal {
            if *c != b'_' {
                cleaned_string.push(*c);
            }
        }
        if cleaned_string.len() <= GUARANTEED_ACC_LENGTH {
            return;
        }
        if cleaned_string.len() > MAX_ACC_LENGTH {
            report(
                self.diag_reporter,
                DiagIntegerLiteralWillLosePrecision {
                    characters: SourceCodeSpan::from_slice(number_literal),
                    rounded_val: b"inf",
                },
            );
            return;
        }
        let cleaned_string: &str =
            unsafe { std::str::from_utf8_unchecked(cleaned_string.as_slice()) };
        let num: Result<f64, std::num::ParseFloatError> = cleaned_string.parse::<f64>();
        let num: f64 = match num {
            Ok(num) => num,
            Err(_) => {
                // TODO(port)
                todo!();
            }
        };
        // TODO(port): Avoid this heap allocation to make this similar to the C++ code. (Really, we
        // should redesign this code anyway...)
        let result_string: String = format!("{num:.0}");
        qljs_always_assert!(result_string.len() <= MAX_ACC_LENGTH);
        if cleaned_string != result_string {
            let result_string_bytes: &[u8] = result_string.as_bytes();
            let rounded_val: &mut [std::mem::MaybeUninit<u8>] = self
                .allocator
                .allocate_uninitialized_array::<u8>(result_string_bytes.len());
            write_slice(rounded_val, result_string_bytes);
            report(
                self.diag_reporter,
                DiagIntegerLiteralWillLosePrecision {
                    characters: SourceCodeSpan::from_slice(number_literal),
                    rounded_val: unsafe { slice_assume_init_ref(rounded_val) },
                },
            );
        }
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
        let mut input = InputPointer(input);
        while is_octal_digit(input[0]) {
            input += 1;
        }
        input.0
    }

    fn parse_decimal_digits_and_underscores(&mut self, input: *const u8) -> *const u8 {
        self.parse_digits_and_underscores(|character: u8| -> bool { is_digit(character) }, input)
    }

    fn parse_hex_digits_and_underscores(&mut self, input: *const u8) -> *const u8 {
        self.parse_digits_and_underscores(
            |character: u8| -> bool { is_hex_digit(character) },
            input,
        )
    }

    fn parse_unicode_escape(
        &mut self,
        input: *const u8,
        reporter: &dyn DiagReporter,
    ) -> ParsedUnicodeEscape {
        let mut input = InputPointer(input);
        let escape_sequence_begin: *const u8 = input.0;
        let get_escape_span =
            |input: InputPointer| unsafe { SourceCodeSpan::new(escape_sequence_begin, input.0) };

        let code_point_hex_begin: *const u8;
        let code_point_hex_end: *const u8;
        if input[2] == b'{' {
            code_point_hex_begin = (input + 3).0;
            input += 3; // Skip "\u{".
            let mut found_non_hex_digit: bool = false;
            while input[0] != b'}' {
                if !is_identifier_byte(input[0]) {
                    // TODO: Add an enum to DiagUnclosedIdentifierEscapeSequence to
                    // indicate whether the token is a template literal, a string literal
                    // or an identifier.
                    report(
                        reporter,
                        DiagUnclosedIdentifierEscapeSequence {
                            escape_sequence: get_escape_span(input),
                        },
                    );
                    return ParsedUnicodeEscape {
                        end: input.0,
                        code_point: None,
                    };
                }
                if !is_hex_digit(input[0]) {
                    found_non_hex_digit = true;
                }
                input += 1;
            }
            code_point_hex_end = input.0;
            input += 1; // Skip "}".
            if found_non_hex_digit || code_point_hex_begin == code_point_hex_end {
                report(
                    reporter,
                    DiagExpectedHexDigitsInUnicodeEscape {
                        escape_sequence: get_escape_span(input),
                    },
                );
                return ParsedUnicodeEscape {
                    end: input.0,
                    code_point: None,
                };
            }
        } else {
            input += 2; // Skip "\u".
            code_point_hex_begin = input.0;
            for i in 0..4 {
                if input[0] == b'\0' && self.is_eof(input.0) {
                    // TODO: Add an enum to DiagExpectedHexDigitsInUnicodeEscape to
                    // indicate whether the token is a template literal, a string literal
                    // or an identifier.
                    report(
                        reporter,
                        DiagExpectedHexDigitsInUnicodeEscape {
                            escape_sequence: get_escape_span(input),
                        },
                    );
                    return ParsedUnicodeEscape {
                        end: input.0,
                        code_point: None,
                    };
                }
                if !is_hex_digit(input[0]) {
                    report(
                        reporter,
                        DiagExpectedHexDigitsInUnicodeEscape {
                            escape_sequence: unsafe {
                                SourceCodeSpan::new(escape_sequence_begin, (input + 1).0)
                            },
                        },
                    );
                    return ParsedUnicodeEscape {
                        end: input.0,
                        code_point: None,
                    };
                }
                input += 1;
            }
            code_point_hex_end = input.0;
        }
        let code_point_hex: &[u8] = unsafe {
            std::slice::from_raw_parts(
                code_point_hex_begin,
                code_point_hex_end.offset_from(code_point_hex_begin) as usize,
            )
        };
        let code_point_hex: &str = unsafe { std::str::from_utf8_unchecked(code_point_hex) };

        let code_point: u32 = u32::from_str_radix(code_point_hex, 16).unwrap_or(0x110000u32);
        if code_point >= 0x110000 {
            report(
                reporter,
                DiagEscapedCodePointInUnicodeOutOfRange {
                    escape_sequence: get_escape_span(input),
                },
            );
        }
        ParsedUnicodeEscape {
            end: input.0,
            code_point: Some(code_point),
        }
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
        kind: IdentifierKind,
    ) -> ParsedIdentifier</* HACK(strager) */ 'code, 'code> {
        let mut input = InputPointer(input);
        let is_private_identifier: bool = identifier_begin != self.original_input.c_str()
            && unsafe { *identifier_begin.offset(-1) } == b'#';
        let private_identifier_begin: *const u8 = if is_private_identifier {
            unsafe { identifier_begin.offset(-1) }
        } else {
            identifier_begin
        };

        let mut normalized: BumpVector<u8, MonotonicAllocator> =
            BumpVector::new("parse_identifier_slow normalized", self.get_allocator());
        normalized
            .append(unsafe { SourceCodeSpan::new(private_identifier_begin, input.0) }.as_slice());

        let escape_sequences: &mut EscapeSequenceList = unsafe {
            &mut *self.allocator.new_object(EscapeSequenceList::new(
                "parse_identifier_slow escape_sequences",
                &self.get_allocator(),
            ))
        };

        fn parse_unicode_escape<Alloc: BumpAllocatorLike>(
            this: &mut Lexer,
            input: &mut InputPointer,
            identifier_begin: *const u8,
            normalized: &mut BumpVector<u8, Alloc>,
            kind: IdentifierKind,
            escape_sequences: &mut EscapeSequenceList,
        ) {
            let escape_begin: InputPointer = *input;
            let escape: ParsedUnicodeEscape =
                this.parse_unicode_escape(escape_begin.0, this.diag_reporter);
            let escape_span: SourceCodeSpan =
                unsafe { SourceCodeSpan::new(escape_begin.0, escape.end) };

            match escape.code_point {
                Some(code_point) => {
                    let is_identifier_initial: bool = escape_begin.0 == identifier_begin;
                    if code_point >= 0x110000 {
                        // parse_unicode_escape reported
                        // DiagEscapedCodePointInIdentifierOutOfRange already.
                        normalized.append(escape_span.as_slice());
                    } else if !is_identifier_initial
                        && kind == IdentifierKind::JSX
                        && code_point == ('-' as u32)
                    {
                        report(
                            this.diag_reporter,
                            DiagEscapedHyphenNotAllowedInJSXTag {
                                escape_sequence: escape_span,
                            },
                        );
                        normalized.append(escape_span.as_slice());
                    } else if !(if is_identifier_initial {
                        is_initial_identifier_character(code_point)
                    } else {
                        is_identifier_character(code_point, IdentifierKind::JavaScript)
                    }) {
                        report(
                            this.diag_reporter,
                            DiagEscapedCharacterDisallowedInIdentifiers {
                                escape_sequence: escape_span,
                            },
                        );
                        normalized.append(escape_span.as_slice());
                    } else {
                        let normalized_len_before: usize = normalized.size();
                        normalized.append_count(4, b'\0');
                        let encoded: &mut [u8] =
                            &mut normalized.as_mut_slice()[normalized_len_before..];
                        // TODO(port): Change encode_utf_8's interface so this is less awkward.
                        let encoded_remainder_len: usize = encode_utf_8(code_point, encoded).len();
                        normalized.resize(normalized.size() - encoded_remainder_len);
                        escape_sequences.push_back(escape_span);
                    }
                }

                None => {
                    normalized.append(escape_span.as_slice());
                }
            }

            qljs_assert!(input.0 != escape.end);
            *input = InputPointer(escape.end);
        }

        loop {
            let mut decode_result: DecodeUTF8Result = decode_utf_8(unsafe {
                PaddedStringView::from_begin_end(input.0, self.original_input.null_terminator())
            });
            if decode_result.size == 0 {
                qljs_assert!(self.is_eof(input.0));
                break;
            }
            if !decode_result.ok {
                let errors_begin: InputPointer = input;
                input += narrow_cast::<isize, _>(decode_result.size);
                loop {
                    decode_result = decode_utf_8(unsafe {
                        PaddedStringView::from_begin_end(
                            input.0,
                            self.original_input.null_terminator(),
                        )
                    });
                    if decode_result.ok || decode_result.size == 0 {
                        break;
                    }
                    input += narrow_cast::<isize, _>(decode_result.size);
                }
                let sequence_span: SourceCodeSpan =
                    unsafe { SourceCodeSpan::new(errors_begin.0, input.0) };
                report(
                    self.diag_reporter,
                    DiagInvalidUTF8Sequence {
                        sequence: sequence_span,
                    },
                );
                normalized.append(sequence_span.as_slice());
                continue;
            }

            if input[0] == b'\\' {
                if input[1] == b'u' {
                    parse_unicode_escape(
                        self,
                        &mut input,
                        identifier_begin,
                        &mut normalized,
                        kind,
                        escape_sequences,
                    );
                } else {
                    let backslash_begin: InputPointer = input;
                    input += 1;
                    let backslash_end: InputPointer = input;
                    let backslash_span: SourceCodeSpan =
                        unsafe { SourceCodeSpan::new(backslash_begin.0, backslash_end.0) };
                    report(
                        self.diag_reporter,
                        DiagUnexpectedBackslashInIdentifier {
                            backslash: backslash_span,
                        },
                    );
                    normalized.append(backslash_span.as_slice());
                }
            } else {
                qljs_assert!(decode_result.size >= 1);
                let character_begin: InputPointer = input;
                let character_end: InputPointer =
                    input + narrow_cast::<isize, _>(decode_result.size);
                let code_point: u32 = decode_result.code_point;

                let is_identifier_initial: bool = character_begin.0 == identifier_begin;
                let is_legal_character: bool = if is_identifier_initial {
                    is_initial_identifier_character(code_point)
                } else {
                    is_identifier_character(code_point, kind)
                };
                let character_span: SourceCodeSpan =
                    unsafe { SourceCodeSpan::new(character_begin.0, character_end.0) };
                if !is_legal_character {
                    if is_ascii_code_point(code_point)
                        || is_non_ascii_whitespace_character(code_point)
                    {
                        break;
                    } else {
                        report(
                            self.diag_reporter,
                            DiagCharacterDisallowedInIdentifiers {
                                character: character_span,
                            },
                        );
                        // Allow non-ASCII characters in the identifier. Otherwise, we'd try
                        // parsing the invalid character as an identifier character again,
                        // causing an infinite loop.
                    }
                }

                normalized.append(character_span.as_slice());
                input = character_end;
            }
        }

        let normalized_slice: &[u8] = unsafe { &*normalized.release() };

        ParsedIdentifier {
            after: input.0,
            normalized: normalized_slice,
            escape_sequences: Some(escape_sequences),
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

    // HACK(port): Hack lifetimes to prevent Rust getting confused by code like the following:
    //
    //   let thing = alloc_something(&self.allocator);  // shared borrow of self
    //   self.mut_method();                             // mutable borrow of self (error!)
    fn get_allocator(&self) -> &'code MonotonicAllocator {
        unsafe { std::mem::transmute(&self.allocator) }
    }
}

struct ParsedUnicodeEscape {
    end: *const u8,
    code_point: Option<u32>,
}

struct ParsedTemplateBody<'alloc, 'code> {
    type_: TokenType,
    end: *const u8,
    escape_sequence_diagnostics: Option<&'alloc mut BufferingDiagReporter<'code>>,
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
// identifier, and .normalized points to a heap-allocated string b"wat".
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

fn is_octal_digit(c: u8) -> bool {
    matches!(c, qljs_case_octal_digit!())
}

fn is_digit(c: u8) -> bool {
    matches!(c, qljs_case_decimal_digit!())
}

fn is_hex_digit(c: u8) -> bool {
    matches!(c, qljs_case_decimal_digit!() | b'a'..=b'f' | b'A'..=b'F')
}

fn is_initial_identifier_byte(byte: u8) -> bool {
    matches!(byte, qljs_case_identifier_start!() | 0xc2..=0xcb | 0xcd..=0xed | 0xef..=0xf0)
}

fn is_identifier_byte(byte: u8) -> bool {
    matches!(byte, qljs_case_decimal_digit!() | qljs_case_identifier_start!() | 0xc2..=0xed | 0xef..=0xf0 | 0xf3)
}

fn is_initial_identifier_character(code_point: u32) -> bool {
    look_up_in_unicode_table(&IDENTIFIER_START_CHUNK_INDEXES, code_point)
}

fn is_identifier_character(code_point: u32, kind: IdentifierKind) -> bool {
    if kind == IdentifierKind::JSX && code_point == (b'-' as u32) {
        return true;
    }
    look_up_in_unicode_table(&IDENTIFIER_PART_CHUNK_INDEXES, code_point)
}

fn is_non_ascii_whitespace_character(code_point: u32) -> bool {
    qljs_assert!(code_point >= 0x80);
    const NON_ASCII_WHITESPACE_CODE_POINTS: &[u16] = &[
        0x00a0, // 0xc2 0xa0      No-Break Space (NBSP)
        0x1680, // 0xe1 0x9a 0x80 Ogham Space Mark
        0x2000, // 0xe2 0x80 0x80 En Quad
        0x2001, // 0xe2 0x80 0x81 Em Quad
        0x2002, // 0xe2 0x80 0x82 En Space
        0x2003, // 0xe2 0x80 0x83 Em Space
        0x2004, // 0xe2 0x80 0x84 Three-Per-Em Space
        0x2005, // 0xe2 0x80 0x85 Four-Per-Em Space
        0x2006, // 0xe2 0x80 0x86 Six-Per-Em Space
        0x2007, // 0xe2 0x80 0x87 Figure Space
        0x2008, // 0xe2 0x80 0x88 Punctuation Space
        0x2009, // 0xe2 0x80 0x89 Thin Space
        0x200a, // 0xe2 0x80 0x8a Hair Space
        0x2028, // 0xe2 0x80 0xa8 Line Separator
        0x2029, // 0xe2 0x80 0xa9 Paragraph Separator
        0x202f, // 0xe2 0x80 0xaf Narrow No-Break Space (NNBSP)
        0x205f, // 0xe2 0x81 0x9f Medium Mathematical Space (MMSP)
        0x3000, // 0xe3 0x80 0x80 Ideographic Space
        0xfeff, // 0xef 0xbb 0xbf Zero Width No-Break Space (BOM, ZWNBSP)
    ];
    if code_point >= 0x10000 {
        false
    } else {
        NON_ASCII_WHITESPACE_CODE_POINTS
            .binary_search(&narrow_cast::<u16, _>(code_point))
            .is_ok()
    }
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
