// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

#include <gtest/gtest.h>
#include <quick-lint-js/document.h>
#include <quick-lint-js/port/char8.h>
#include <quick-lint-js/port/warning.h>
#include <quick-lint-js/web-demo-location.h>

QLJS_WARNING_IGNORE_GCC("-Wsuggest-override")

namespace quick_lint_js {
namespace {
template <typename Locator>
class test_document : public testing::Test {};

using document_locator_types = ::testing::Types<web_demo_locator>;
TYPED_TEST_SUITE(test_document, document_locator_types,
                 ::testing::internal::DefaultNameGenerator);

TYPED_TEST(test_document, set_text) {
  using Locator = TypeParam;
  document<Locator> doc;
  doc.set_text(u8"content goes here"_sv);
  EXPECT_EQ(doc.string(), u8"content goes here"_sv);
}

TYPED_TEST(test_document, set_text_multiple_times) {
  using Locator = TypeParam;
  document<Locator> doc;
  doc.set_text(u8"content goes here"_sv);
  doc.set_text(u8"newer content goes here"_sv);
  EXPECT_EQ(doc.string(), u8"newer content goes here"_sv);
  doc.set_text(u8"finally"_sv);
  EXPECT_EQ(doc.string(), u8"finally"_sv);
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
