// Copyright (C) 2020  Matthew "strager" Glazar
// See end of file for extended copyright information.

// See ADR001-Feature-testing-with-have-h.md for usage of and rationale for this
// file.

#ifndef QUICK_LINT_JS_PORT_HAVE_H
#define QUICK_LINT_JS_PORT_HAVE_H

#if defined(QLJS_HAVE_VERSION_HEADER) && QLJS_HAVE_VERSION_HEADER
#elif defined(__has_include)
#if __has_include(<version>)
#define QLJS_HAVE_VERSION_HEADER 1
#endif
#endif
#if !defined(QLJS_HAVE_VERSION_HEADER)
#define QLJS_HAVE_VERSION_HEADER 0
#endif

#if QLJS_HAVE_VERSION_HEADER
#include <version>
#endif

#if defined(QLJS_HAVE_UNISTD_H) && QLJS_HAVE_UNISTD_H
#elif defined(__has_include)
#if __has_include(<unistd.h>) && !defined(__EMSCRIPTEN__) && !defined(__MINGW32__)
#define QLJS_HAVE_UNISTD_H 1
#endif
#elif defined(__unix__) && !defined(__EMSCRIPTEN__)
#define QLJS_HAVE_UNISTD_H 1
#endif
#if !defined(QLJS_HAVE_UNISTD_H)
#define QLJS_HAVE_UNISTD_H 0
#endif

#if defined(QLJS_HAVE_SANITIZER_ASAN_INTERFACE_H) && \
    QLJS_HAVE_SANITIZER_ASAN_INTERFACE_H
#elif defined(__has_include)
#if __has_include(<sanitizer/asan_interface.h>)
#define QLJS_HAVE_SANITIZER_ASAN_INTERFACE_H 1
#endif
#endif
#if !defined(QLJS_HAVE_SANITIZER_ASAN_INTERFACE_H)
#define QLJS_HAVE_SANITIZER_ASAN_INTERFACE_H 0
#endif

#if QLJS_HAVE_UNISTD_H
// Define _POSIX_VERSION.
#include <unistd.h>
#endif

#if !defined(QLJS_HAVE_CHARCONV_HEADER) && defined(__has_include)
// std::to_chars on libc++ version 7.0.0 is buggy on macOS x86_64.
#if __has_include(<charconv>) && \
    !(defined(_LIBCPP_VERSION) && _LIBCPP_VERSION <= 7000)
#define QLJS_HAVE_CHARCONV_HEADER 1
#endif
#endif
#if !defined(QLJS_HAVE_CHARCONV_HEADER)
#define QLJS_HAVE_CHARCONV_HEADER 0
#endif

#if !defined(QLJS_HAVE_ARM_NEON)
#if defined(__ARM_NEON)
#define QLJS_HAVE_ARM_NEON 1
#else
#define QLJS_HAVE_ARM_NEON 0
#endif
#endif

#if !defined(QLJS_HAVE_ARM_NEON_A64)
#if QLJS_HAVE_ARM_NEON && defined(__aarch64__)
#define QLJS_HAVE_ARM_NEON_A64 1
#else
#define QLJS_HAVE_ARM_NEON_A64 0
#endif
#endif

#if !defined(QLJS_HAVE_WEB_ASSEMBLY_SIMD128)
#if defined(__wasm_simd128__)
#define QLJS_HAVE_WEB_ASSEMBLY_SIMD128 1
#else
#define QLJS_HAVE_WEB_ASSEMBLY_SIMD128 0
#endif
#endif

#if !defined(QLJS_HAVE_X86_SSE2)
#if defined(_M_AMD64) || defined(_M_X64) || \
    (defined(_M_IX86_FP) && _M_IX86_FP == 2) || defined(__SSE2__)
#define QLJS_HAVE_X86_SSE2 1
#else
#define QLJS_HAVE_X86_SSE2 0
#endif
#endif

// TODO(strager): Check for SSE4.2 support in MSVC.
#if !defined(QLJS_HAVE_X86_SSE4_2)
#if defined(__SSE4_2__)
#define QLJS_HAVE_X86_SSE4_2 1
#else
#define QLJS_HAVE_X86_SSE4_2 0
#endif
#endif

#if !defined(QLJS_HAVE_DEBUGBREAK)
#if defined(_WIN32) && defined(__has_include)
#if __has_include(<intrin.h>)
#define QLJS_HAVE_DEBUGBREAK 1
#endif
#endif
#endif
#if !defined(QLJS_HAVE_DEBUGBREAK)
#define QLJS_HAVE_DEBUGBREAK 0
#endif

#if !defined(QLJS_HAVE_BUILTIN_TRAP)
#if defined(__GNUC__) || defined(__clang__)
#define QLJS_HAVE_BUILTIN_TRAP 1
#else
#define QLJS_HAVE_BUILTIN_TRAP 0
#endif
#endif

#if !defined(QLJS_HAVE_BUILTIN_FILE_FUNCTION_LINE)
#if defined(__has_builtin)
#if __has_builtin(__builtin_FILE) && __has_builtin(__builtin_FUNCTION) && \
    __has_builtin(__builtin_LINE)
#define QLJS_HAVE_BUILTIN_FILE_FUNCTION_LINE 1
#else
#define QLJS_HAVE_BUILTIN_FILE_FUNCTION_LINE 0
#endif
#else
#define QLJS_HAVE_BUILTIN_FILE_FUNCTION_LINE 0
#endif
#endif

#if !defined(QLJS_HAVE_SIZED_ALIGNED_NEW)
// TODO(strager): Set this to 1 if operator new is supported with both a size
// and an alignment. Our Debian build compiles and links but doesn't run with
// this set, so be conservative and disable it for now.
#define QLJS_HAVE_SIZED_ALIGNED_NEW 0
#endif

#if !defined(QLJS_HAVE_SIZED_ALIGNED_DELETE)
// TODO(strager): Set this to 1 if operator delete is supported with both a size
// and an alignment. Our Debian build compiles and links but doesn't run with
// this set, so be conservative and disable it for now.
#define QLJS_HAVE_SIZED_ALIGNED_DELETE 0
#endif

#if !defined(QLJS_HAVE_FILE_NAME_MACRO)
#if defined(__clang__) && (defined(NDEBUG) && NDEBUG)
#define QLJS_HAVE_FILE_NAME_MACRO 1
#else
#define QLJS_HAVE_FILE_NAME_MACRO 0
#endif
#endif

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
