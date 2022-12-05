use crate::container::monotonic_allocator::*;
use crate::container::padded_string::*;
use crate::fe::diag_reporter::*;
use crate::fe::token::*;
use crate::qljs_assert;

pub struct Lexer<'code, 'reporter> {
    last_token: Token</* HACK(strager) */ 'code, 'code>,
    last_last_token_end: *const u8,
    input: *const u8,
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
            input: input.c_str(),
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
        self.last_token.begin = self.input;
        match unsafe { *self.input } {
            // TODO(port): QLJS_CASE_DECIMAL_DIGIT:
            // TODO(port): QLJS_CASE_IDENTIFIER_START:
            // TODO(port): default:
            b'(' | b')' | b',' | b':' | b';' | b'[' | b']' | b'{' | b'}' | b'~' => {
                self.last_token.type_ = unsafe { std::mem::transmute(*self.input) };
                self.input = unsafe { self.input.offset(1) };
                self.last_token.end = self.input;
            }

            // TODO(port): case '?':
            // TODO(port): case '.':
            // TODO(port): case '=':
            // TODO(port): case '!':
            // TODO(port): case '<':
            // TODO(port): case '>':
            // TODO(port): case '+':
            // TODO(port): case '-':
            // TODO(port): case '*':
            // TODO(port): case '/':
            // TODO(port): case '^':
            // TODO(port): case '%':
            // TODO(port): case '&':
            // TODO(port): case '|':
            // TODO(port): case '"': case '\'':
            // TODO(port): case '`':
            // TODO(port): case '#':
            b'\0' => {
                if self.is_eof(self.input) {
                    self.last_token.type_ = TokenType::EndOfFile;
                    self.last_token.end = self.input;
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

    fn skip_whitespace(&mut self) {
        // TODO(port)
    }

    fn is_eof(&self, input: *const u8) -> bool {
        qljs_assert!(unsafe { *input } == b'\0');
        input == self.original_input.null_terminator()
    }
}
