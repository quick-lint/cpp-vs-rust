# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

cmake_minimum_required(VERSION 3.10)
if (POLICY CMP0091)
  cmake_policy(SET CMP0091 NEW)
endif ()

project(quick-lint-js)

if (${CMAKE_VERSION} VERSION_GREATER_EQUAL 3.17.3)
  option(QUICK_LINT_JS_PRECOMPILE_HEADERS "Use CMake's target_precompile_headers" ON)
else ()
  set(QUICK_LINT_JS_PRECOMPILE_HEADERS FALSE)
endif ()

set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_STANDARD 20)

if (MSVC)
  add_compile_options(/EHc-s- /GR-)
  add_definitions(-D_HAS_EXCEPTIONS=0)
else ()
  add_compile_options(-fno-exceptions -fno-rtti)
endif ()

include(vendor/googletest.cmake)

add_subdirectory(src)
add_subdirectory(test)

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
