// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#include <quick-lint-js/container/padded-string.h>
#include <quick-lint-js/fe/lex.h>
#include <quick-lint-js/fe/linter.h>
#include <quick-lint-js/fe/token.h>

namespace quick_lint_js {
void parse_and_lint(padded_string_view code, diag_reporter& reporter,
                    linter_options options) {
  // NOTE(port): This is trimmed down because we aren't porting the parser or
  // the variable analyzer. Just lex the whole document. This won't work if
  // there are regexp literals or template literals, but whatever.
  lexer l(code, &reporter);
  while (l.peek().type != token_type::end_of_file) {
    l.skip();
  }
}
}

// quick-lint-js finds bugs in JavaScript programs.
// Copyright (C) 2020  Matthew "strager" Glazar
//
// This file is part of quick-lint-js.
//
// quick-lint-js is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// quick-lint-js is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with quick-lint-js.  If not, see <https://www.gnu.org/licenses/>.
