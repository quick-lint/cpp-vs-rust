# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

cmake_minimum_required(VERSION 3.10)

add_library(
  quick-lint-js-lib
  quick-lint-js/assert.cpp
  quick-lint-js/assert.h
  quick-lint-js/container/allocator.h
  quick-lint-js/container/async-byte-queue.cpp
  quick-lint-js/container/async-byte-queue.h
  quick-lint-js/container/byte-buffer.cpp
  quick-lint-js/container/byte-buffer.h
  quick-lint-js/container/concat.h
  quick-lint-js/container/hash-map.h
  quick-lint-js/container/hash-set.h
  quick-lint-js/container/hash.h
  quick-lint-js/container/heap-function.h
  quick-lint-js/container/linked-bump-allocator.h
  quick-lint-js/container/linked-vector.h
  quick-lint-js/container/optional.h
  quick-lint-js/container/padded-string.cpp
  quick-lint-js/container/padded-string.h
  quick-lint-js/container/result.h
  quick-lint-js/container/vector.h
  quick-lint-js/debug/debug-probe.cpp
  quick-lint-js/debug/debug-probe.h
  quick-lint-js/debug/debug-server-fs.cpp
  quick-lint-js/debug/debug-server-fs.h
  quick-lint-js/debug/debug-server.cpp
  quick-lint-js/debug/debug-server.h
  quick-lint-js/debug/mongoose.cpp
  quick-lint-js/debug/mongoose.h
  quick-lint-js/fe/buffering-diag-reporter.cpp
  quick-lint-js/fe/buffering-diag-reporter.h
  quick-lint-js/fe/diag-code-list.cpp
  quick-lint-js/fe/diag-code-list.h
  quick-lint-js/fe/diag-reporter.cpp
  quick-lint-js/fe/diag-reporter.h
  quick-lint-js/fe/diagnostic-formatter.cpp
  quick-lint-js/fe/diagnostic-formatter.h
  quick-lint-js/fe/diagnostic-types.h
  quick-lint-js/fe/diagnostic.cpp
  quick-lint-js/fe/diagnostic.h
  quick-lint-js/fe/global-declared-variable-set.cpp
  quick-lint-js/fe/global-declared-variable-set.h
  quick-lint-js/fe/global-variables-browser-generated.cpp
  quick-lint-js/fe/global-variables.cpp
  quick-lint-js/fe/global-variables.h
  quick-lint-js/fe/identifier.h
  quick-lint-js/fe/jsx.cpp
  quick-lint-js/fe/jsx.h
  quick-lint-js/fe/language.h
  quick-lint-js/fe/lex-keyword-generated.cpp
  quick-lint-js/fe/lex-unicode-generated.cpp
  quick-lint-js/fe/lex.cpp
  quick-lint-js/fe/lex.h
  quick-lint-js/fe/reported-diag-statistics.h
  quick-lint-js/fe/source-code-span.cpp
  quick-lint-js/fe/source-code-span.h
  quick-lint-js/feature.h
  quick-lint-js/i18n/locale.cpp
  quick-lint-js/i18n/locale.h
  quick-lint-js/i18n/translation-table-generated.cpp
  quick-lint-js/i18n/translation-table-generated.h
  quick-lint-js/i18n/translation-table.h
  quick-lint-js/i18n/translation.cpp
  quick-lint-js/i18n/translation.h
  quick-lint-js/io/event-loop.h
  quick-lint-js/io/file-canonical.cpp
  quick-lint-js/io/file-canonical.h
  quick-lint-js/io/file-handle.cpp
  quick-lint-js/io/file-handle.h
  quick-lint-js/io/file-path.cpp
  quick-lint-js/io/file-path.h
  quick-lint-js/io/file.cpp
  quick-lint-js/io/file.h
  quick-lint-js/io/output-stream.cpp
  quick-lint-js/io/output-stream.h
  quick-lint-js/io/pipe-writer.cpp
  quick-lint-js/io/pipe-writer.h
  quick-lint-js/io/pipe.cpp
  quick-lint-js/io/pipe.h
  quick-lint-js/io/temporary-directory.cpp
  quick-lint-js/io/temporary-directory.h
  quick-lint-js/json.cpp
  quick-lint-js/json.h
  quick-lint-js/logging/log.h
  quick-lint-js/logging/logger.cpp
  quick-lint-js/logging/logger.h
  quick-lint-js/logging/trace-flusher-directory-backend.cpp
  quick-lint-js/logging/trace-flusher.cpp
  quick-lint-js/logging/trace-flusher.h
  quick-lint-js/logging/trace-metadata.cpp
  quick-lint-js/logging/trace-metadata.h
  quick-lint-js/logging/trace-stream-reader.cpp
  quick-lint-js/logging/trace-writer.cpp
  quick-lint-js/logging/trace-writer.h
  quick-lint-js/port/attribute.h
  quick-lint-js/port/bit.h
  quick-lint-js/port/char8.cpp
  quick-lint-js/port/char8.h
  quick-lint-js/port/consteval.h
  quick-lint-js/port/crash.h
  quick-lint-js/port/function-ref.h
  quick-lint-js/port/have.h
  quick-lint-js/port/in-range.h
  quick-lint-js/port/integer.cpp
  quick-lint-js/port/integer.h
  quick-lint-js/port/limits.h
  quick-lint-js/port/max-align.h
  quick-lint-js/port/memory-resource.cpp
  quick-lint-js/port/memory-resource.h
  quick-lint-js/port/process.cpp
  quick-lint-js/port/process.h
  quick-lint-js/port/simd.h
  quick-lint-js/port/source-location.h
  quick-lint-js/port/thread.cpp
  quick-lint-js/port/thread.h
  quick-lint-js/port/type-traits.h
  quick-lint-js/port/unreachable.h
  quick-lint-js/port/warning.h
  quick-lint-js/port/windows-error.cpp
  quick-lint-js/port/windows-error.h
  quick-lint-js/port/windows.h
  quick-lint-js/util/binary-reader.h
  quick-lint-js/util/binary-writer.h
  quick-lint-js/util/cpp.h
  quick-lint-js/util/instance-tracker.h
  quick-lint-js/util/math-overflow.h
  quick-lint-js/util/narrow-cast.h
  quick-lint-js/util/pointer.h
  quick-lint-js/util/try-catch-stack.h
  quick-lint-js/util/type-traits.h
  quick-lint-js/util/utf-16.cpp
  quick-lint-js/util/utf-16.h
  quick-lint-js/util/utf-8.cpp
  quick-lint-js/util/utf-8.h
  quick-lint-js/version.h
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
  quick-lint-js/container/vector-profiler-debug.cpp
  quick-lint-js/fe/diag-debug.cpp
  quick-lint-js/fe/language-debug.cpp
  quick-lint-js/fe/lex-debug.cpp
  quick-lint-js/port/char8-debug.cpp
)
target_link_libraries(quick-lint-js-lib-debug PUBLIC quick-lint-js-lib)

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
