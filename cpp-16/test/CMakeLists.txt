# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

cmake_minimum_required(VERSION 3.10)

add_executable(
  quick-lint-js-test
  diag-collector.cpp
  diag-matcher.cpp
  parse-support.cpp
  quick-lint-js/array.h
  quick-lint-js/characters.h
  quick-lint-js/diag-collector.h
  quick-lint-js/diag-matcher.h
  quick-lint-js/dirty-set.h
  quick-lint-js/gtest.h
  quick-lint-js/parse-support.h
  test-assert.cpp
  test-buffering-diag-reporter.cpp
  test-c-api.cpp
  test-diagnostic-formatter.cpp
  test-diagnostic.cpp
  test-document.cpp
  test-integer-hexadecimal.cpp
  test-lex.cpp
  test-lex-1.cpp
  test-lex-2.cpp
  test-lex-3.cpp
  test-lex-4.cpp
  test-lex-5.cpp
  test-lex-6.cpp
  test-lex-7.cpp
  test-lex-8.cpp
  test-lex-9.cpp
  test-lex-10.cpp
  test-lex-11.cpp
  test-lex-12.cpp
  test-lex-13.cpp
  test-lex-14.cpp
  test-lex-15.cpp
  test-linked-bump-allocator.cpp
  test-linked-vector.cpp
  test-locale.cpp
  test-narrow-cast.cpp
  test-padded-string.cpp
  test-simd.cpp
  test-sorted-search.cpp
  test-translation-table-generated.cpp
  test-translation.cpp
  test-utf-8.cpp
  test-vector.cpp
  test-web-demo-location.cpp
)
target_include_directories(quick-lint-js-test PRIVATE .)
target_link_libraries(
  quick-lint-js-test
  PRIVATE
  gmock
  gmock_main
  gtest
  quick-lint-js-lib
  quick-lint-js-lib-debug
)
if (QUICK_LINT_JS_PRECOMPILE_HEADERS)
  target_precompile_headers(
    quick-lint-js-test
    PRIVATE
    # TODO(strager): Optimize these.
    <gmock/gmock.h>
    <gtest/gtest.h>
    <iostream>
  )
endif ()

# HACK(strager): LTO builds are very slow. Our tests don't
# *need* LTO optimizations. Disable LTO for our tests.
set_target_properties(
  quick-lint-js-test
  PROPERTIES
  INTERPROCEDURAL_OPTIMIZATION OFF
)

add_test(
  NAME quick-lint-js-test
  COMMAND quick-lint-js-test
)

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
