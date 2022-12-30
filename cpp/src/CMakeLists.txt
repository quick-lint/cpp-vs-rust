# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

cmake_minimum_required(VERSION 3.10)

add_library(
  quick-lint-js-lib
  quick-lint-js/assert.cpp
  quick-lint-js/assert.h
  quick-lint-js/c-api-diag-reporter.cpp
  quick-lint-js/c-api-diag-reporter.h
  quick-lint-js/c-api.cpp
  quick-lint-js/c-api.h
  quick-lint-js/container/allocator.h
  quick-lint-js/container/linked-bump-allocator.h
  quick-lint-js/container/linked-vector.h
  quick-lint-js/container/optional.h
  quick-lint-js/container/padded-string.cpp
  quick-lint-js/container/padded-string.h
  quick-lint-js/container/vector.h
  quick-lint-js/document.cpp
  quick-lint-js/document.h
  quick-lint-js/fe/buffering-diag-reporter.cpp
  quick-lint-js/fe/buffering-diag-reporter.h
  quick-lint-js/fe/diag-reporter.cpp
  quick-lint-js/fe/diag-reporter.h
  quick-lint-js/fe/diagnostic-formatter.cpp
  quick-lint-js/fe/diagnostic-formatter.h
  quick-lint-js/fe/diagnostic-types.h
  quick-lint-js/fe/diagnostic.cpp
  quick-lint-js/fe/diagnostic.h
  quick-lint-js/fe/identifier.h
  quick-lint-js/fe/language.h
  quick-lint-js/fe/lex-keyword-generated.cpp
  quick-lint-js/fe/lex-unicode-generated.cpp
  quick-lint-js/fe/lex.cpp
  quick-lint-js/fe/lex.h
  quick-lint-js/fe/linter.cpp
  quick-lint-js/fe/linter.h
  quick-lint-js/fe/source-code-span.cpp
  quick-lint-js/fe/source-code-span.h
  quick-lint-js/i18n/locale.cpp
  quick-lint-js/i18n/locale.h
  quick-lint-js/i18n/translation-table-generated.cpp
  quick-lint-js/i18n/translation-table-generated.h
  quick-lint-js/i18n/translation-table.h
  quick-lint-js/i18n/translation.cpp
  quick-lint-js/i18n/translation.h
  quick-lint-js/port/attribute.h
  quick-lint-js/port/bit.h
  quick-lint-js/port/char8.cpp
  quick-lint-js/port/char8.h
  quick-lint-js/port/crash.h
  quick-lint-js/port/have.h
  quick-lint-js/port/in-range.h
  quick-lint-js/port/integer.cpp
  quick-lint-js/port/integer.h
  quick-lint-js/port/limits.h
  quick-lint-js/port/max-align.h
  quick-lint-js/port/memory-resource.cpp
  quick-lint-js/port/memory-resource.h
  quick-lint-js/port/simd.h
  quick-lint-js/port/source-location.h
  quick-lint-js/port/type-traits.h
  quick-lint-js/port/unreachable.h
  quick-lint-js/port/warning.h
  quick-lint-js/util/cpp.h
  quick-lint-js/util/narrow-cast.h
  quick-lint-js/util/pointer.h
  quick-lint-js/util/utf-8.cpp
  quick-lint-js/util/utf-8.h
  quick-lint-js/web-demo-location.cpp
  quick-lint-js/web-demo-location.h
)
target_include_directories(quick-lint-js-lib PUBLIC .)
if (QUICK_LINT_JS_PRECOMPILE_HEADERS)
  target_precompile_headers(
    quick-lint-js-lib
    PUBLIC
    # TODO(strager): Optimize these.
    <cmath>
    <cstring>
    <string>
  )
endif ()

# quick-lint-js-lib-debug should be dead-code stripped from production builds.
# It is a separate library from quick-lint-js-lib to increase the chance that
# the linker drops the code entirely.
#
# Code in quick-lint-js-lib-debug can be used from tests, but should not be used
# in production code.
add_library(
  quick-lint-js-lib-debug
  quick-lint-js/container/padded-string-debug.cpp
  quick-lint-js/fe/diag-debug.cpp
  quick-lint-js/fe/language-debug.cpp
  quick-lint-js/fe/lex-debug.cpp
  quick-lint-js/port/char8-debug.cpp
)
target_link_libraries(quick-lint-js-lib-debug PUBLIC quick-lint-js-lib)

add_library(quick-lint-js-c-api SHARED empty.cpp)
target_link_libraries(quick-lint-js-c-api PUBLIC quick-lint-js-lib)
if (CMAKE_SYSTEM_NAME STREQUAL Linux)
  target_link_libraries(
    quick-lint-js-c-api
    PRIVATE
    -Wl,--undefined=qljs_list_locales
    -Wl,--undefined=qljs_web_demo_create_document
    -Wl,--undefined=qljs_web_demo_destroy_document
    -Wl,--undefined=qljs_web_demo_diagnostic
    -Wl,--undefined=qljs_web_demo_document
    -Wl,--undefined=qljs_web_demo_lint
    -Wl,--undefined=qljs_web_demo_set_config
    -Wl,--undefined=qljs_web_demo_set_language_options
    -Wl,--undefined=qljs_web_demo_set_locale
    -Wl,--undefined=qljs_web_demo_set_text
  )
  target_link_libraries(
    quick-lint-js-c-api
    PRIVATE
    -Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/quick-lint-js-c-api.map
  )
endif ()

# quick-lint-js finds bugs in JavaScript programs.
# Copyright (C) 2020  Matthew "strager" Glazar
#
# This file is part of quick-lint-js.
#
# quick-lint-js is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# quick-lint-js is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with quick-lint-js.  If not, see <https://www.gnu.org/licenses/>.
