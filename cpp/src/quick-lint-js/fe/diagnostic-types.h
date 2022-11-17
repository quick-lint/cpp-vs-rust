// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#ifndef QUICK_LINT_JS_FE_DIAGNOSTIC_TYPES_H
#define QUICK_LINT_JS_FE_DIAGNOSTIC_TYPES_H

#include <iosfwd>
#include <quick-lint-js/fe/identifier.h>
#include <quick-lint-js/fe/language.h>
#include <quick-lint-js/fe/source-code-span.h>
#include <quick-lint-js/fe/token.h>
#include <quick-lint-js/i18n/translation.h>
#include <quick-lint-js/port/char8.h>

// QLJS_DIAG_TYPE should have the following signature:
//
// #define QLJS_DIAG_TYPE(error_name, error_code, severity, struct_body,
// format) ...
//
// * error_name: identifier
// * error_code: string literal
// * severity: diagnostic_severity value
// * struct_body: class member list, wrapped in { }
// * format: member function calls
//
// A class named *error_name* is created in the quick_lint_js namespace.
// *struct_body* is the body of the class.
//
// *format* should look like the following:
//
//    MESSAGE(QLJS_TRANSLATABLE("format string"), source_location)
//
// Within *format*:
//
// * MESSAGE's first argument must be QLJS_TRANSLATABLE(...)
// * MESSAGE's second argument must be a member variable of the *error_name*
//   class (i.e. listed in *struct_body*)
// * MESSAGE's second argument must have type *identifier* or *source_code_span*
//
// When removing a diagnostic from this list, add an entry to
// QLJS_X_RESERVED_DIAG_TYPES.
#define QLJS_X_DIAG_TYPES                                                       \
  QLJS_DIAG_TYPE(                                                               \
      diag_big_int_literal_contains_decimal_point, "E0005",                     \
      diagnostic_severity::error, { source_code_span where; },                  \
      MESSAGE(QLJS_TRANSLATABLE("BigInt literal contains decimal point"),       \
              where))                                                           \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_big_int_literal_contains_exponent, "E0006",                          \
      diagnostic_severity::error, { source_code_span where; },                  \
      MESSAGE(QLJS_TRANSLATABLE("BigInt literal contains exponent"), where))    \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_character_disallowed_in_identifiers, "E0011",                        \
      diagnostic_severity::error, { source_code_span character; },              \
      MESSAGE(QLJS_TRANSLATABLE("character is not allowed in identifiers"),     \
              character))                                                       \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_escaped_character_disallowed_in_identifiers, "E0012",                \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "escaped character is not allowed in identifiers"),           \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_escaped_code_point_in_identifier_out_of_range, "E0013",              \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE("code point out of range"), escape_sequence))   \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_escaped_code_point_in_unicode_out_of_range, "E0207",                 \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE("code point in Unicode escape sequence must "   \
                                "not be greater than U+10FFFF"),                \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_escaped_hyphen_not_allowed_in_jsx_tag, "E0019",                      \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(                                                                  \
          QLJS_TRANSLATABLE(                                                    \
              "escaping '-' is not allowed in tag names; write '-' instead"),   \
          escape_sequence))                                                     \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_expected_hex_digits_in_unicode_escape, "E0016",                      \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "expected hexadecimal digits in Unicode escape sequence"),    \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_invalid_hex_escape_sequence, "E0060", diagnostic_severity::error,    \
      { source_code_span escape_sequence; },                                    \
      MESSAGE(QLJS_TRANSLATABLE("invalid hex escape sequence: {0}"),            \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_invalid_quotes_around_string_literal, "E0197",                       \
      diagnostic_severity::error,                                               \
      {                                                                         \
        source_code_span opening_quote;                                         \
        char8 suggested_quote;                                                  \
      },                                                                        \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "'{0}' is not allowed for strings; use {1} instead"),         \
              opening_quote, suggested_quote))                                  \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_invalid_utf_8_sequence, "E0022", diagnostic_severity::error,         \
      { source_code_span sequence; },                                           \
      MESSAGE(QLJS_TRANSLATABLE("invalid UTF-8 sequence"), sequence))           \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_keywords_cannot_contain_escape_sequences, "E0023",                   \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE("keywords cannot contain escape sequences"),    \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_legacy_octal_literal_may_not_be_big_int, "E0032",                    \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("legacy octal literal may not be BigInt"),      \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_legacy_octal_literal_may_not_contain_underscores, "E0152",           \
      diagnostic_severity::error, { source_code_span underscores; },            \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "legacy octal literals may not contain underscores"),         \
              underscores))                                                     \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_number_literal_contains_consecutive_underscores, "E0028",            \
      diagnostic_severity::error, { source_code_span underscores; },            \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "number literal contains consecutive underscores"),           \
              underscores))                                                     \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_number_literal_contains_trailing_underscores, "E0029",               \
      diagnostic_severity::error, { source_code_span underscores; },            \
      MESSAGE(                                                                  \
          QLJS_TRANSLATABLE("number literal contains trailing underscore(s)"),  \
          underscores))                                                         \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_octal_literal_may_not_have_exponent, "E0030",                        \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("octal literal may not have exponent"),         \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_octal_literal_may_not_have_decimal, "E0031",                         \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("octal literal may not have decimal"),          \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_integer_literal_will_lose_precision, "E0212",                        \
      diagnostic_severity::warning,                                             \
      {                                                                         \
        source_code_span characters;                                            \
        string8_view rounded_val;                                               \
      },                                                                        \
      MESSAGE(QLJS_TRANSLATABLE("integer cannot be represented and will be "    \
                                "rounded to '{1}'"),                            \
              characters, rounded_val))                                         \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_regexp_literal_flags_cannot_contain_unicode_escapes, "E0035",        \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "RegExp literal flags cannot contain Unicode escapes"),       \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_block_comment, "E0037", diagnostic_severity::error,         \
      { source_code_span comment_open; },                                       \
      MESSAGE(QLJS_TRANSLATABLE("unclosed block comment"), comment_open))       \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_identifier_escape_sequence, "E0038",                        \
      diagnostic_severity::error, { source_code_span escape_sequence; },        \
      MESSAGE(QLJS_TRANSLATABLE("unclosed identifier escape sequence"),         \
              escape_sequence))                                                 \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_regexp_literal, "E0039", diagnostic_severity::error,        \
      { source_code_span regexp_literal; },                                     \
      MESSAGE(QLJS_TRANSLATABLE("unclosed regexp literal"), regexp_literal))    \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_string_literal, "E0040", diagnostic_severity::error,        \
      { source_code_span string_literal; },                                     \
      MESSAGE(QLJS_TRANSLATABLE("unclosed string literal"), string_literal))    \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_jsx_string_literal, "E0181", diagnostic_severity::error,    \
      { source_code_span string_literal_begin; },                               \
      MESSAGE(QLJS_TRANSLATABLE("unclosed string literal"),                     \
              string_literal_begin))                                            \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unclosed_template, "E0041", diagnostic_severity::error,              \
      { source_code_span incomplete_template; },                                \
      MESSAGE(QLJS_TRANSLATABLE("unclosed template"), incomplete_template))     \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_at_character, "E0042", diagnostic_severity::error,        \
      { source_code_span character; },                                          \
      MESSAGE(QLJS_TRANSLATABLE("unexpected '@'"), character))                  \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_backslash_in_identifier, "E0043",                         \
      diagnostic_severity::error, { source_code_span backslash; },              \
      MESSAGE(QLJS_TRANSLATABLE("unexpected '\\' in identifier"), backslash))   \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_characters_in_number, "E0044",                            \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("unexpected characters in number literal"),     \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_control_character, "E0045", diagnostic_severity::error,   \
      { source_code_span character; },                                          \
      MESSAGE(QLJS_TRANSLATABLE("unexpected control character"), character))    \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_characters_in_binary_number, "E0046",                     \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("unexpected characters in binary literal"),     \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_characters_in_octal_number, "E0047",                      \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("unexpected characters in octal literal"),      \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_characters_in_hex_number, "E0048",                        \
      diagnostic_severity::error, { source_code_span characters; },             \
      MESSAGE(QLJS_TRANSLATABLE("unexpected characters in hex literal"),        \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_greater_in_jsx_text, "E0182",                             \
      diagnostic_severity::error, { source_code_span greater; },                \
      MESSAGE(QLJS_TRANSLATABLE("'>' is not allowed directly in JSX text; "     \
                                "write {{'>'} or &gt; instead"),                \
              greater))                                                         \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_right_curly_in_jsx_text, "E0183",                         \
      diagnostic_severity::error, { source_code_span right_curly; },            \
      MESSAGE(QLJS_TRANSLATABLE("'}' is not allowed directly in JSX text; "     \
                                "write {{'}'} instead"),                        \
              right_curly))                                                     \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unopened_block_comment, "E0210", diagnostic_severity::error,         \
      { source_code_span comment_close; },                                      \
      MESSAGE(QLJS_TRANSLATABLE("unopened block comment"), comment_close))      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_no_digits_in_binary_number, "E0049", diagnostic_severity::error,     \
      { source_code_span characters; },                                         \
      MESSAGE(QLJS_TRANSLATABLE("binary number literal has no digits"),         \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_no_digits_in_hex_number, "E0050", diagnostic_severity::error,        \
      { source_code_span characters; },                                         \
      MESSAGE(QLJS_TRANSLATABLE("hex number literal has no digits"),            \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_no_digits_in_octal_number, "E0051", diagnostic_severity::error,      \
      { source_code_span characters; },                                         \
      MESSAGE(QLJS_TRANSLATABLE("octal number literal has no digits"),          \
              characters))                                                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_hash_character, "E0052", diagnostic_severity::error,      \
      { source_code_span where; },                                              \
      MESSAGE(QLJS_TRANSLATABLE("unexpected '#'"), where))                      \
                                                                                \
  QLJS_DIAG_TYPE(                                                               \
      diag_unexpected_bom_before_shebang, "E0095", diagnostic_severity::error,  \
      { source_code_span bom; },                                                \
      MESSAGE(QLJS_TRANSLATABLE(                                                \
                  "unicode byte order mark (BOM) cannot appear before #! "      \
                  "at beginning of script"),                                    \
              bom))                                                             \
  /* END */

namespace quick_lint_js {
#define QLJS_DIAG_TYPE(name, code, severity, struct_body, format_call) \
  struct name struct_body;
QLJS_X_DIAG_TYPES
#undef QLJS_DIAG_TYPE

enum class diag_type {
#define QLJS_DIAG_TYPE(name, code, severity, struct_body, format_call) name,
  QLJS_X_DIAG_TYPES
#undef QLJS_DIAG_TYPE
};

std::ostream& operator<<(std::ostream&, diag_type);

template <class Error>
struct diag_type_from_type_detail;

#define QLJS_DIAG_TYPE(name, code, severity, struct_body, format_call) \
  template <>                                                          \
  struct diag_type_from_type_detail<name> {                            \
    static constexpr diag_type type = diag_type::name;                 \
  };
QLJS_X_DIAG_TYPES
#undef QLJS_DIAG_TYPE

template <class Error>
inline constexpr diag_type diag_type_from_type =
    diag_type_from_type_detail<Error>::type;

inline constexpr int diag_type_count = 0
#define QLJS_DIAG_TYPE(name, code, severity, struct_body, format_call) +1
    QLJS_X_DIAG_TYPES
#undef QLJS_DIAG_TYPE
    ;
}

#endif

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
