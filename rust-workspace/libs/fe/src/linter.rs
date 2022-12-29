use crate::diag_reporter::*;
use crate::lex::*;
use crate::token::*;
use cpp_vs_rust_util::padded_string::*;

// TODO(#465): Accept parser options from quick-lint-js.config or CLI options.
pub struct LinterOptions {
    // If true, parse and lint JSX language extensions:
    // https://facebook.github.io/jsx/
    pub jsx: bool,

    // If true, parse and lint TypeScript instead of JavaScript.
    pub typescript: bool,

    // If true, print a human-readable representation of parser visits to stderr.
    pub print_parser_visits: bool,
}

impl Default for LinterOptions {
    fn default() -> LinterOptions {
        LinterOptions {
            jsx: true,
            typescript: true,
            print_parser_visits: true,
        }
    }
}

pub fn parse_and_lint(
    code: PaddedStringView<'_>,
    reporter: &'_ dyn DiagReporter,
    _linter_options: LinterOptions,
) {
    // NOTE(port): This is trimmed down because we aren't porting the parser or
    // the variable analyzer. Just lex the whole document. This won't work if
    // there are regexp literals or template literals, but whatever.
    let allocator = LexerAllocator::new();
    let mut l: Lexer = Lexer::new(code, reporter, &allocator);
    while l.peek().type_ != TokenType::EndOfFile {
        l.skip();
    }
}
