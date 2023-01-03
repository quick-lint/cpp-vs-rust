// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#include <cerrno>
#include <cinttypes>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <quick-lint-js/port/char8.h>
#include <quick-lint-js/port/have.h>
#include <quick-lint-js/port/integer.h>
#include <quick-lint-js/port/warning.h>
#include <quick-lint-js/util/narrow-cast.h>
#include <string>
#include <type_traits>

QLJS_WARNING_IGNORE_GCC("-Wuseless-cast")

#if QLJS_HAVE_CHARCONV_HEADER
#include <charconv>
#endif

namespace quick_lint_js {
#if QLJS_HAVE_CHARCONV_HEADER
from_chars_result from_chars_hex(const char *begin, const char *end,
                                 char32_t &value) {
  using underlying_type = std::uint_least32_t;
  static_assert(sizeof(char32_t) == sizeof(underlying_type));
  underlying_type parsed_value;
  std::from_chars_result result =
      std::from_chars(begin, end, parsed_value, /*base=*/16);
  if (result.ec == std::errc{}) {
    value = static_cast<char32_t>(parsed_value);
  }
  return from_chars_result{.ptr = result.ptr, .ec = result.ec};
}

from_chars_result from_chars_hex(const char *begin, const char *end,
                                 std::uint8_t &value) {
  std::from_chars_result result =
      std::from_chars(begin, end, value, /*base=*/16);
  return from_chars_result{.ptr = result.ptr, .ec = result.ec};
}
#else
namespace {
bool is_decimal_digit(char c) noexcept { return '0' <= c && c <= '9'; }

bool is_hexadecimal_digit(char c) noexcept {
  return ('0' <= c && c <= '9') || ('a' <= c && c <= 'f') ||
         ('A' <= c && c <= 'F');
}
}

from_chars_result from_chars_hex(const char *begin, const char *end,
                                 char32_t &value) {
  std::string buffer(begin, end);
  if (!(buffer.size() >= 1 && is_hexadecimal_digit(buffer[0]))) {
    return from_chars_result{.ptr = begin, .ec = std::errc::invalid_argument};
  }
  if (buffer.size() >= 1 && (buffer[1] == 'x' || buffer[1] == 'X')) {
    // Prevent strtol from parsing '0x' prefixes.
    buffer[1] = '\0';
  }
  char *endptr;
  errno = 0;
  long long_value = std::strtol(buffer.c_str(), &endptr, /*base=*/16);
  const char *ptr = (endptr - buffer.c_str()) + begin;
  if (errno == ERANGE || !in_range<char32_t>(long_value)) {
    return from_chars_result{.ptr = ptr, .ec = std::errc::result_out_of_range};
  }
  value = static_cast<char32_t>(long_value);
  return from_chars_result{.ptr = ptr, .ec = std::errc{0}};
}

from_chars_result from_chars_hex(const char *begin, const char *end,
                                 std::uint8_t &value) {
  char32_t long_value;
  from_chars_result result = from_chars_hex(begin, end, long_value);
  if (result.ec != std::errc()) {
    return result;
  }
  if (!in_range<std::uint8_t>(long_value)) {
    return from_chars_result{.ptr = result.ptr,
                             .ec = std::errc::result_out_of_range};
  }
  value = static_cast<std::uint8_t>(long_value);
  return result;
}
#endif

from_char8s_result from_char8s_hex(const char8 *begin, const char8 *end,
                                   char32_t &value) {
  from_chars_result result =
      from_chars_hex(reinterpret_cast<const char *>(begin),
                     reinterpret_cast<const char *>(end), value);
  return from_char8s_result{
      .ptr = reinterpret_cast<const char8 *>(result.ptr),
      .ec = result.ec,
  };
}

from_char8s_result from_char8s_hex(const char8 *begin, const char8 *end,
                                   unsigned char &value) {
  from_chars_result result =
      from_chars_hex(reinterpret_cast<const char *>(begin),
                     reinterpret_cast<const char *>(end), value);
  return from_char8s_result{
      .ptr = reinterpret_cast<const char8 *>(result.ptr),
      .ec = result.ec,
  };
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
