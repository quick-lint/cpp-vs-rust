#!/usr/bin/env python3

# Copyright (C) 2020  Matthew "strager" Glazar
# See end of file for extended copyright information.

import json
import os
import pathlib
import subprocess
import typing

CONVERTED_CPP_FILES = [
    "cpp/src/quick-lint-js/assert.h",  # Sorta.
    "cpp/src/quick-lint-js/container/allocator.h",
    "cpp/src/quick-lint-js/container/linked-bump-allocator.h",
    "cpp/src/quick-lint-js/container/linked-vector.h",
    "cpp/src/quick-lint-js/container/monotonic-allocator.h",
    "cpp/src/quick-lint-js/container/optional.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/container/padded-string-debug.cpp",
    "cpp/src/quick-lint-js/container/padded-string.cpp",
    "cpp/src/quick-lint-js/container/padded-string.h",
    "cpp/src/quick-lint-js/container/sorted-search.h",
    "cpp/src/quick-lint-js/container/string-view.h",  # Sorta.
    "cpp/src/quick-lint-js/container/vector.h",
    "cpp/src/quick-lint-js/container/winkable.h",
    "cpp/src/quick-lint-js/fe/buffering-diag-reporter.cpp",
    "cpp/src/quick-lint-js/fe/buffering-diag-reporter.h",
    "cpp/src/quick-lint-js/fe/diag-debug.cpp",
    "cpp/src/quick-lint-js/fe/diag-reporter.cpp",
    "cpp/src/quick-lint-js/fe/diag-reporter.h",
    "cpp/src/quick-lint-js/fe/diagnostic-formatter.cpp",
    "cpp/src/quick-lint-js/fe/diagnostic-formatter.h",
    "cpp/src/quick-lint-js/fe/diagnostic-types.h",
    "cpp/src/quick-lint-js/fe/diagnostic.cpp",
    "cpp/src/quick-lint-js/fe/diagnostic.h",
    "cpp/src/quick-lint-js/fe/identifier.h",
    "cpp/src/quick-lint-js/fe/language-debug.cpp",
    "cpp/src/quick-lint-js/fe/language.h",
    "cpp/src/quick-lint-js/fe/source-code-span.cpp",
    "cpp/src/quick-lint-js/fe/source-code-span.h",
    "cpp/src/quick-lint-js/fe/token.h",
    "cpp/src/quick-lint-js/i18n/locale.cpp",
    "cpp/src/quick-lint-js/i18n/locale.h",
    "cpp/src/quick-lint-js/i18n/translation-table-generated.cpp",
    "cpp/src/quick-lint-js/i18n/translation-table-generated.h",
    "cpp/src/quick-lint-js/i18n/translation-table.h",
    "cpp/src/quick-lint-js/i18n/translation.cpp",
    "cpp/src/quick-lint-js/i18n/translation.h",
    "cpp/src/quick-lint-js/port/attribute.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/bit.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/char8-debug.cpp",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/char8.cpp",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/char8.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/have.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/in-range.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/limits.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/math.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/max-align.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/memory-resource.cpp",
    "cpp/src/quick-lint-js/port/memory-resource.h",
    "cpp/src/quick-lint-js/port/simd-neon-arm.h",
    "cpp/src/quick-lint-js/port/simd.h",
    "cpp/src/quick-lint-js/port/type-traits.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/vector-erase.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/port/warning.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/util/cpp.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/util/narrow-cast.h",
    "cpp/src/quick-lint-js/util/pointer.h",  # Not needed in Rust.
    "cpp/src/quick-lint-js/util/utf-8.cpp",
    "cpp/src/quick-lint-js/util/utf-8.h",
    "cpp/test/diag-collector.cpp",
    "cpp/test/quick-lint-js/array.h",  # Not needed in Rust.
    "cpp/test/quick-lint-js/diag-collector.h",
    "cpp/test/quick-lint-js/gtest.h",  # Not needed in Rust.
    "cpp/test/quick-lint-js/test-translation-table-generated.h",
    "cpp/test/test-assert.cpp",
    "cpp/test/test-buffering-diag-reporter.cpp",
    "cpp/test/test-diagnostic-formatter.cpp",
    "cpp/test/test-diagnostic.cpp",
    "cpp/test/test-linked-bump-allocator.cpp",
    "cpp/test/test-linked-vector.cpp",
    "cpp/test/test-locale.cpp",
    "cpp/test/test-narrow-cast.cpp",  # Not needed.
    "cpp/test/test-padded-string.cpp",
    "cpp/test/test-simd.cpp",
    "cpp/test/test-sorted-search.cpp",
    "cpp/test/test-translation-table-generated.cpp",
    "cpp/test/test-translation.cpp",
    "cpp/test/test-utf-8.cpp",
    "cpp/test/test-vector.cpp",
]


def main() -> None:
    os.chdir(pathlib.Path(__file__).parent / "..")
    cwd = pathlib.Path(".")
    converted_cpp_files = set(cwd / p for p in CONVERTED_CPP_FILES)

    cpp_files = flatten(
        cwd.glob(pattern)
        for pattern in [
            "cpp/src/**/*.cpp",
            "cpp/src/**/*.h",
            "cpp/test/**/*.cpp",
            "cpp/test/**/*.h",
        ]
    )

    rust_files = flatten(
        cwd.glob(pattern)
        for pattern in [
            "rust/libs/**/*.rs",
            "rust/src/**/*.rs",
            "rust/tests/**/*.rs",
        ]
    )

    cpp_total_sloc = sloc(cpp_files)
    cpp_human_sloc = sloc([p for p in cpp_files if not is_generated(p)])
    cpp_total_converted_sloc = sloc([p for p in cpp_files if p in converted_cpp_files])
    cpp_human_converted_sloc = sloc(
        [p for p in cpp_files if p in converted_cpp_files and not is_generated(p)]
    )

    rust_total_sloc = sloc(rust_files)
    rust_human_sloc = sloc([p for p in rust_files if not is_generated(p)])

    print(
        f"""\
Total C++ SLOC:                   {cpp_total_sloc:7}
Total non-generated C++ SLOC:     {cpp_human_sloc:7}
Converted C++ SLOC:               {cpp_total_converted_sloc:7} ({100 * cpp_total_converted_sloc / cpp_total_sloc:.1f}%)
Converted non-generated C++ SLOC: {cpp_human_converted_sloc:7} ({100 * cpp_human_converted_sloc / cpp_human_sloc:.1f}%)

Rust SLOC:                        {rust_total_sloc:7} ({100 * rust_total_sloc / cpp_total_converted_sloc:.1f}% of converted C++)
Non-generated Rust SLOC:          {rust_human_sloc:7} ({100 * rust_human_sloc / cpp_human_converted_sloc:.1f}% of converted C++)\
"""
    )

    print("\nUnconverted files:")
    for file in sorted(cpp_files):
        if file not in converted_cpp_files:
            print(f"  {file}")


def flatten(iterable_of_iterables) -> typing.List:
    return [x for iterable in iterable_of_iterables for x in iterable]


def is_generated(path: pathlib.Path) -> bool:
    return "generated" in str(path)


def is_test(path: pathlib.Path) -> bool:
    print(path)
    return True


def sloc(files: typing.List[pathlib.Path]) -> None:
    output = subprocess.check_output(["cloc", "--json", "--"] + files, encoding="utf-8")
    data = json.loads(output)
    return data["SUM"]["code"]


if __name__ == "__main__":
    main()

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
