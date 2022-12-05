use crate::container::monotonic_allocator::*;
use crate::container::padded_string::*;
use crate::fe::diag_reporter::*;
use crate::fe::diagnostic_types::*;
use crate::fe::source_code_span::*;
use crate::fe::token::*;
use crate::qljs_assert;

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
    // Precondition: self.peek().type != TokenType::EndOfFile.
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
            // TODO(port): QLJS_CASE_DECIMAL_DIGIT:
            // TODO(port): QLJS_CASE_IDENTIFIER_START:
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

    fn parse_number(&mut self) {
        todo!();
    }

    fn skip_whitespace(&mut self) {
        // TODO(port)
    }

    fn skip_block_comment(&mut self) {
        todo!();
    }

    fn skip_line_comment_body(&mut self) {
        todo!();
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

fn is_digit(c: u8) -> bool {
    matches!(c, b'0'..=b'9')
}

// NOTE(port): This is a transitioning struct to make it easier to port code.
#[derive(Clone, Copy, Eq, PartialEq)]
struct InputPointer(*const u8);

impl std::ops::Index<usize> for InputPointer {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        unsafe { &*self.0.offset(index as isize) }
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
