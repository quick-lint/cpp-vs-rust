// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#include <cstddef>
#include <gtest/gtest.h>
#include <quick-lint-js/fe/diagnostic-types.h>
#include <quick-lint-js/fe/diagnostic.h>
#include <quick-lint-js/port/char8.h>
#include <string_view>

using namespace std::literals::string_view_literals;

namespace quick_lint_js {
namespace {
template <class Error>
inline const diagnostic_info& diagnostic_info_for_error =
    get_diagnostic_info(diag_type_from_type<Error>);

TEST(test_diagnostic, diagnostic_info) {
  translator source_code_translator;
  source_code_translator.use_messages_from_source_code();

  {
    const diagnostic_info& info = diagnostic_info_for_error<
        diag_big_int_literal_contains_decimal_point>;
    EXPECT_EQ(info.code, 5);
    EXPECT_EQ(info.severity, diagnostic_severity::error);
    EXPECT_EQ(source_code_translator.translate(info.message_formats[0]),
              u8"BigInt literal contains decimal point"_sv);
    EXPECT_EQ(
        info.message_args[0][0].offset(),
        offsetof(diag_big_int_literal_contains_decimal_point, where));
    EXPECT_EQ(info.message_args[0][0].type,
              diagnostic_arg_type::source_code_span);
    EXPECT_FALSE(info.message_formats[1].valid());
  }

  {
    const diagnostic_info& info = diagnostic_info_for_error<
        diag_invalid_quotes_around_string_literal>;
    EXPECT_EQ(info.code, 197);
    EXPECT_EQ(info.severity, diagnostic_severity::error);
    EXPECT_EQ(source_code_translator.translate(info.message_formats[0]),
              u8"'{0}' is not allowed for strings; use {1} instead"_sv);
    EXPECT_EQ(info.message_args[0][0].offset(),
              offsetof(diag_invalid_quotes_around_string_literal, opening_quote));
    EXPECT_EQ(info.message_args[0][0].type,
              diagnostic_arg_type::source_code_span);
    EXPECT_EQ(info.message_args[0][1].offset(),
              offsetof(diag_invalid_quotes_around_string_literal, suggested_quote));
    EXPECT_EQ(info.message_args[0][1].type, diagnostic_arg_type::char8);
    EXPECT_FALSE(info.message_formats[1].valid());
  }

  {
    const diagnostic_info& info =
        diagnostic_info_for_error<diag_multiple_message_test>;
    EXPECT_EQ(info.code, 6969);
    EXPECT_EQ(info.severity, diagnostic_severity::error);
    EXPECT_EQ(source_code_translator.translate(info.message_formats[0]),
              u8"test for multiple messages"_sv);
    EXPECT_EQ(
        info.message_args[0][0].offset(),
        offsetof(diag_multiple_message_test, a));
    EXPECT_EQ(info.message_args[0][0].type,
              diagnostic_arg_type::source_code_span);
    EXPECT_EQ(
        source_code_translator.translate(info.message_formats[1]),
        u8"second message here"_sv);
    EXPECT_EQ(
        info.message_args[1][0].offset(),
        offsetof(diag_multiple_message_test, b));
    EXPECT_EQ(info.message_args[1][0].type,
              diagnostic_arg_type::source_code_span);
  }
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
