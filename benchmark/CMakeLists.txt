# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

cmake_minimum_required(VERSION 3.10)
if (POLICY CMP0091)
  cmake_policy(SET CMP0091 NEW)
endif ()

project(cpp-vs-rust-benchmark)

set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_STANDARD 20)

if (MSVC)
  add_compile_options(/EHc-s- /GR-)
  add_definitions(-D_HAS_EXCEPTIONS=0)
else ()
  add_compile_options(-fno-exceptions -fno-rtti)
endif ()

include(../cpp/vendor/benchmark.cmake)

add_executable(
  cpp-vs-rust-benchmark
  cpp-vs-rust-benchmark.cpp
)
target_link_libraries(
  cpp-vs-rust-benchmark
  PRIVATE
  benchmark_main
  dl
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
