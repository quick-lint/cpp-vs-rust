#!/usr/bin/env python3

import abc
import argparse
import collections
import contextlib
import dataclasses
import math
import os
import pathlib
import re
import shutil
import socket
import sqlite3
import subprocess
import sys
import time
import typing
import unittest

HOSTNAME = socket.gethostname()

ROOT = pathlib.Path(__file__).parent / ".."

BENCH_BUILD_DB = ROOT / "bench-build.db"

CPP_ROOT = ROOT / "cpp"
CPP_BUILD_DIR = CPP_ROOT / "build"

MOLD_LINKER_EXE: typing.Optional[str] = shutil.which("mold")

CARGO_CLIF_EXE: typing.Optional[pathlib.Path] = pathlib.Path(
    "/home/strager/tmp/Projects/rustc_codegen_cranelift/dist/cargo-clif"
)
if not CARGO_CLIF_EXE.exists():
    CARGO_CLIF_EXE = None


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dump-all-runs", action="store_true")
    parser.add_argument("--dump-runs", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--iterations", type=int, default=3)
    parser.add_argument("--warmup-iterations", type=int, default=2)
    parser.add_argument("filter", default="", nargs="?")
    args = parser.parse_args()

    if args.self_test:
        unittest.main(verbosity=2, exit=True, argv=sys.argv[:1])

    if args.dump_runs or args.dump_all_runs:
        db = DB(BENCH_BUILD_DB)
        if args.dump_all_runs:
            runs = db.load_all_runs()
        else:
            runs = db.load_latest_runs()
        db.dump_runs(runs)
        return

    if args.list:
        profiler = Lister()
    else:
        db = DB(BENCH_BUILD_DB)
        profiler = Profiler(
            warmup_iterations=args.warmup_iterations,
            iterations=args.iterations,
            db=db,
        )
    profiler = Filterer(profiler, filter=args.filter)

    cpp_configs = find_cpp_configs()

    for cpp_config in cpp_configs:
        profiler.profile(CPPFullBenchmark(cpp_config))
        profiler.profile(CPPHalfBenchmark(cpp_config))
        profiler.profile(CPPTestOnlyBenchmark(cpp_config))
        profiler.profile(
            CPPIncrementalBenchmark(
                cpp_config,
                files_to_mutate=[
                    CPP_ROOT / "src/quick-lint-js/fe/lex.cpp",
                ],
            )
        )
        profiler.profile(
            CPPIncrementalBenchmark(
                cpp_config,
                files_to_mutate=[
                    CPP_ROOT / "src/quick-lint-js/fe/diagnostic-types.h",
                ],
            )
        )
        profiler.profile(
            CPPIncrementalBenchmark(
                cpp_config,
                files_to_mutate=[
                    CPP_ROOT / "test/test-utf-8.cpp",
                ],
            )
        )

    for rust_root in ROOT.glob("rust*"):
        for rust_config in find_rust_configs(root=rust_root):
            profiler.profile(RustFullBenchmark(rust_config))
            profiler.profile(RustHalfBenchmark(rust_config))
            profiler.profile(RustTestOnlyBenchmark(rust_config))
            profiler.profile(
                RustIncrementalBenchmark(
                    rust_config,
                    files_to_mutate=[
                        find_unique_file(rust_root, "lex.rs"),
                    ],
                )
            )
            profiler.profile(
                RustIncrementalBenchmark(
                    rust_config,
                    files_to_mutate=[
                        find_unique_file(rust_root, "diagnostic_types.rs"),
                    ],
                )
            )
            profiler.profile(
                RustIncrementalBenchmark(
                    rust_config,
                    files_to_mutate=[
                        find_unique_file(rust_root, "test_utf_8.rs"),
                    ],
                )
            )

    profiler.dump_results()


class CPPConfig(typing.NamedTuple):
    label: str
    cxx_compiler: pathlib.Path
    cxx_flags: str
    link_flags: str
    pch: bool


def find_cpp_configs() -> typing.List[CPPConfig]:
    cpp_configs = []

    def try_add_cxx_config(config: CPPConfig) -> None:
        if cxx_compiler_builds(
            cxx_compiler=config.cxx_compiler,
            flags=f"{config.cxx_flags} {config.link_flags}",
        ):
            cpp_configs.append(config)
        if config.pch:
            new_config = config._replace(
                label=f"{config.label} -fpch-instantiate-templates",
                cxx_flags=f"{config.cxx_flags} -fpch-instantiate-templates",
            )
            if cxx_compiler_builds(
                cxx_compiler=new_config.cxx_compiler,
                flags=f"{new_config.cxx_flags} {new_config.link_flags}",
            ):
                cpp_configs.append(new_config)

    def try_add_cxx_configs(
        label: str, cxx_compiler: pathlib.Path, cxx_flags: str
    ) -> None:
        for g in ("", "-g0"):
            for pch in (False, True):
                label_suffix = (" PCH" if pch else "") + (" " if g else "") + g
                try_add_cxx_config(
                    CPPConfig(
                        label=f"{label}{label_suffix}",
                        cxx_compiler=cxx_compiler,
                        cxx_flags=f"{cxx_flags} {g}",
                        link_flags="",
                        pch=pch,
                    )
                )
                if MOLD_LINKER_EXE is not None:
                    try_add_cxx_config(
                        CPPConfig(
                            label=f"{label}{label_suffix} Mold",
                            cxx_compiler=cxx_compiler,
                            cxx_flags=f"{cxx_flags} {g}",
                            link_flags=f"-Wl,-fuse-ld={MOLD_LINKER_EXE}",
                            pch=pch,
                        )
                    )

    def try_add_clang_configs(label: str, cxx_compiler: pathlib.Path) -> None:
        try_add_cxx_configs(
            label=f"{label} libstdc++",
            cxx_compiler=cxx_compiler,
            cxx_flags="-stdlib=libstdc++",
        )
        try_add_cxx_configs(
            label=f"{label} libc++",
            cxx_compiler=cxx_compiler,
            cxx_flags="-stdlib=libc++",
        )

    try_add_clang_configs(label="Clang 12", cxx_compiler=pathlib.Path("clang++-12"))
    try_add_clang_configs(
        label="Clang Custom",
        cxx_compiler=pathlib.Path("/home/strager/Toolchains/clang-stage2/bin/clang++"),
    )
    try_add_clang_configs(
        label="Clang Custom PGO",
        cxx_compiler=pathlib.Path(
            "/home/strager/Toolchains/clang-stage4-qljs/bin/clang++"
        ),
    )
    try_add_clang_configs(
        label="Clang Custom PGO BOLT",
        cxx_compiler=pathlib.Path(
            "/home/strager/Toolchains/clang-stage4-qljs-bolt/bin/clang++"
        ),
    )
    try_add_clang_configs(
        label="Clang",
        cxx_compiler=pathlib.Path("clang++"),
    )
    try_add_cxx_configs(
        label="GCC 12",
        cxx_compiler=pathlib.Path("g++-12"),
        cxx_flags="",
    )
    return cpp_configs


class Benchmark:
    project: str
    toolchain_label: str
    name: str

    @property
    def full_name(self) -> str:
        return f"{self.project}, {self.toolchain_label}, {self.name}"

    def before_all_untimed(self) -> None:
        pass

    def before_each_untimed(self) -> None:
        pass

    def run_timed(self) -> None:
        raise NotImplementedError()

    def after_each_untimed(self) -> None:
        pass

    def after_all_untimed(self) -> None:
        pass


class CPPBenchmarkBase(Benchmark):
    project = "C++"

    _cpp_config: CPPConfig

    def __init__(self, cpp_config: CPPConfig) -> None:
        self._cpp_config = cpp_config

    @property
    def toolchain_label(self) -> str:
        return self._cpp_config.label


class CPPFullBenchmark(CPPBenchmarkBase):
    name = "full build and test"

    def before_each_untimed(self) -> None:
        cpp_clean()

    def run_timed(self) -> None:
        cpp_configure(self._cpp_config)
        cpp_build(targets=["quick-lint-js-test"])
        cpp_test()


class CPPHalfBenchmark(CPPBenchmarkBase):
    name = "build and test only my code"

    def before_each_untimed(self) -> None:
        cpp_clean()
        cpp_configure(self._cpp_config)
        cpp_build(targets=["gmock", "gmock_main", "gtest"])

    def run_timed(self) -> None:
        cpp_build(targets=["quick-lint-js-test"])
        cpp_test()


class CPPIncrementalBenchmark(CPPBenchmarkBase):
    _files_to_mutate: typing.Tuple[pathlib.Path]

    def __init__(
        self, cpp_config: CPPConfig, files_to_mutate: typing.List[pathlib.Path]
    ) -> None:
        super().__init__(cpp_config)
        self._files_to_mutate = tuple(files_to_mutate)

    @property
    def name(self) -> str:
        names = ", ".join(sorted(f.name for f in self._files_to_mutate))
        return f"incremental build and test ({names})"

    def before_all_untimed(self) -> None:
        cpp_clean()
        cpp_configure(self._cpp_config)
        cpp_build(targets=["quick-lint-js-test"])

    def before_each_untimed(self) -> None:
        for f in self._files_to_mutate:
            mutate_file(f)

    def run_timed(self) -> None:
        cpp_build(targets=["quick-lint-js-test"])
        cpp_test()

    def after_all_untimed(self) -> None:
        for f in self._files_to_mutate:
            unmutate_file(f)


class CPPTestOnlyBenchmark(CPPBenchmarkBase):
    name = "test only"

    def before_all_untimed(self) -> None:
        cpp_clean()
        cpp_configure(self._cpp_config)
        cpp_build(targets=["quick-lint-js-test"])

    def run_timed(self) -> None:
        cpp_test()


def cpp_clean() -> None:
    delete_dir(CPP_BUILD_DIR)


def cpp_configure(cpp_config: CPPConfig) -> None:
    subprocess.check_call(
        [
            "cmake",
            "-S",
            ".",
            "-B",
            CPP_BUILD_DIR.relative_to(CPP_ROOT),
            "-G",
            "Ninja",
            f"-DCMAKE_CXX_COMPILER={cpp_config.cxx_compiler}",
            f"-DCMAKE_CXX_FLAGS={cpp_config.cxx_flags}",
            f"-DCMAKE_EXE_LINKER_FLAGS={cpp_config.link_flags}",
            f"-DCMAKE_SHARED_LINKER_FLAGS={cpp_config.link_flags}",
            f"-DQUICK_LINT_JS_PRECOMPILE_HEADERS={'YES' if cpp_config.pch else 'NO'}",
        ],
        cwd=CPP_ROOT,
    )


def cpp_build(targets: typing.List[str] = []) -> None:
    subprocess.check_call(["ninja", "-C", CPP_BUILD_DIR, "--"] + targets)


def cpp_test() -> None:
    subprocess.check_call([CPP_BUILD_DIR / "test" / "quick-lint-js-test"])


class RustConfig(typing.NamedTuple):
    root: pathlib.Path
    label: str
    cargo: pathlib.Path
    cargo_profile: typing.Optional[str]
    rustflags: str
    nextest: bool

    @property
    def rustc(self) -> pathlib.Path:
        is_clif = self.cargo.name == "cargo-clif"
        return self.cargo.parent / ("rustc-clif" if is_clif else "rustc")


def find_rust_configs(root: pathlib.Path) -> typing.List[RustConfig]:
    rust_configs = []

    def add_rust_configs_for_toolchain(
        label: str,
        cargo: pathlib.Path,
        cargo_profile: typing.Optional[str],
        rustflags: str,
    ) -> None:
        rust_configs.append(
            RustConfig(
                label=f"{label}",
                root=root,
                cargo=cargo,
                cargo_profile=cargo_profile,
                rustflags=rustflags,
                nextest=False,
            )
        )
        rust_configs.append(
            RustConfig(
                label=f"{label} cargo-nextest",
                root=root,
                cargo=cargo,
                cargo_profile=cargo_profile,
                rustflags=rustflags,
                nextest=True,
            )
        )

    def add_rust_configs(
        extra_label: str,
        cargo_profile: typing.Optional[str],
        rustflags: str,
    ) -> None:
        add_rust_configs_for_toolchain(
            label=f"Rust Stable {extra_label}".rstrip(),
            cargo=rustup_which("cargo", toolchain="stable"),
            cargo_profile=cargo_profile,
            rustflags=rustflags,
        )
        add_rust_configs_for_toolchain(
            label=f"Rust Nightly {extra_label}".rstrip(),
            cargo=rustup_which("cargo", toolchain="nightly"),
            cargo_profile=cargo_profile,
            rustflags=rustflags,
        )
        add_rust_configs_for_toolchain(
            label=f"Rust Custom {extra_label}".rstrip(),
            cargo=pathlib.Path("/home/strager/Toolchains/rustc-stage2/bin/cargo"),
            cargo_profile=cargo_profile,
            rustflags=rustflags,
        )
        add_rust_configs_for_toolchain(
            label=f"Rust Custom PGO {extra_label}".rstrip(),
            cargo=pathlib.Path("/home/strager/Toolchains/rustc-stage4-pgo/bin/cargo"),
            cargo_profile=cargo_profile,
            rustflags=rustflags,
        )
        add_rust_configs_for_toolchain(
            label=f"Rust Custom PGO BOLT {extra_label}".rstrip(),
            cargo=pathlib.Path(
                "/home/strager/Toolchains/rustc-stage4-pgo-bolt/bin/cargo"
            ),
            cargo_profile=cargo_profile,
            rustflags=rustflags,
        )
        if CARGO_CLIF_EXE is not None:
            add_rust_configs_for_toolchain(
                label=f"Rust Cranelift {extra_label}".rstrip(),
                cargo=CARGO_CLIF_EXE,
                cargo_profile=cargo_profile,
                rustflags=rustflags,
            )

    for cargo_profile in (
        None,
        "quick-build-incremental",
        "quick-build-nonincremental",
    ):
        add_rust_configs(
            extra_label=f"{cargo_profile or ''}",
            cargo_profile=cargo_profile,
            rustflags="",
        )
        if MOLD_LINKER_EXE is not None:
            add_rust_configs(
                extra_label=f"Mold {cargo_profile or ''}",
                cargo_profile=cargo_profile,
                rustflags=f"-Clinker=clang -Clink-arg=-fuse-ld={MOLD_LINKER_EXE}",
            )
    return reversed(rust_configs)


class RustBenchmarkBase(Benchmark):
    _rust_config: RustConfig

    def __init__(self, rust_config: RustConfig) -> None:
        self._rust_config = rust_config

    @property
    def project(self) -> str:
        return self._rust_config.root.name

    @property
    def toolchain_label(self) -> str:
        return self._rust_config.label

    def before_all_untimed(self) -> None:
        rust_download_dependencies(self._rust_config)


class RustFullBenchmark(RustBenchmarkBase):
    name = "full build and test"

    def before_each_untimed(self) -> None:
        rust_clean(root=self._rust_config.root)

    def run_timed(self) -> None:
        rust_build_and_test(self._rust_config)


class RustHalfBenchmark(RustBenchmarkBase):
    name = "build and test only my code"

    def before_each_untimed(self) -> None:
        rust_clean(root=self._rust_config.root)
        rust_build_packages(
            self._rust_config, packages=["lazy_static", "libc", "memoffset"]
        )

    def run_timed(self) -> None:
        rust_build_and_test(self._rust_config)


class RustIncrementalBenchmark(RustBenchmarkBase):
    _files_to_mutate: typing.Tuple[pathlib.Path]

    def __init__(
        self, rust_config: RustConfig, files_to_mutate: typing.List[pathlib.Path]
    ) -> None:
        super().__init__(rust_config)
        self._files_to_mutate = tuple(files_to_mutate)

    @property
    def name(self) -> str:
        names = ", ".join(sorted(f.name for f in self._files_to_mutate))
        return f"incremental build and test ({names})"

    def before_all_untimed(self) -> None:
        super().before_all_untimed()
        rust_clean(root=self._rust_config.root)
        rust_build_and_test(self._rust_config)

    def before_each_untimed(self) -> None:
        for f in self._files_to_mutate:
            mutate_file(f)

    def run_timed(self) -> None:
        rust_build_and_test(self._rust_config)

    def after_all_untimed(self) -> None:
        for f in self._files_to_mutate:
            unmutate_file(f)


class RustTestOnlyBenchmark(RustBenchmarkBase):
    name = "test only"

    def before_all_untimed(self) -> None:
        super().before_all_untimed()
        rust_clean(root=self._rust_config.root)
        rust_build_and_test(self._rust_config)

    def run_timed(self) -> None:
        rust_build_and_test(self._rust_config)


def rust_clean(root: pathlib.Path) -> None:
    delete_dir(root / "target")


def rust_download_dependencies(rust_config: RustConfig) -> None:
    subprocess.check_call([rust_config.cargo, "fetch"], cwd=rust_config.root)


def rust_build_packages(rust_config: RustConfig, packages: typing.List[str]) -> None:
    assert packages
    command = [rust_config.cargo, "build"]
    if rust_config.cargo_profile is not None:
        command.append(f"--profile={rust_config.cargo_profile}")
    for package in packages:
        command.extend(("--package", package))
    subprocess.check_call(
        command,
        cwd=rust_config.root,
        env=dict(
            os.environ,
            RUSTC=str(rust_config.rustc),
        ),
    )


def rust_build_and_test(rust_config: RustConfig) -> None:
    if rust_config.nextest:
        command = [rust_config.cargo, "nextest", "run"]
        if rust_config.cargo_profile is not None:
            command.append(f"--cargo-profile={rust_config.cargo_profile}")
    else:
        command = [rust_config.cargo, "test"]
        if rust_config.cargo_profile is not None:
            command.append(f"--profile={rust_config.cargo_profile}")
    subprocess.check_call(
        command,
        cwd=rust_config.root,
        env=dict(
            os.environ,
            RUSTC=str(rust_config.rustc),
        ),
    )


MillisecondDuration = int
NanosecondDuration = int


class DB:
    RunID = int

    class Run(typing.NamedTuple):
        id: "DB.RunID"
        hostname: str
        project: str
        toolchain_label: str
        benchmark_name: str
        samples: typing.Tuple[NanosecondDuration, ...]

    def __init__(self, path: typing.Optional[pathlib.Path]) -> None:
        self._connection = sqlite3.connect(":memory:" if path is None else path)

        cursor = self._connection.cursor()
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS run (
                id INTEGER PRIMARY KEY,
                hostname TEXT,
                project TEXT,
                toolchain_label TEXT,
                benchmark_name TEXT,
                created_at NUMERIC
            )
        """
        )
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS sample (
                run_id INTEGER,
                duration_ns NUMERIC
            )
        """
        )
        self._connection.commit()

    def create_run(
        self, hostname: str, project: str, toolchain_label: str, benchmark_name: str
    ) -> "DB.RunID":
        cursor = self._connection.cursor()
        cursor.execute(
            """
            INSERT INTO run (hostname, project, toolchain_label, benchmark_name, created_at)
            VALUES (?, ?, ?, ?, strftime('%s'))
        """,
            (hostname, project, toolchain_label, benchmark_name),
        )
        self._connection.commit()
        return cursor.lastrowid

    def add_sample_to_run(
        self, run_id: "DB.RunID", duration_ns: NanosecondDuration
    ) -> None:
        cursor = self._connection.cursor()
        cursor.execute(
            """
            INSERT INTO sample (run_id, duration_ns)
            VALUES (?, ?)
        """,
            (run_id, duration_ns),
        )
        self._connection.commit()

    def load_all_runs(self) -> typing.List["DB.Run"]:
        return self._load_runs_with_filter(
            samples_where_clause="",
            samples_parameters=(),
            runs_where_clause="",
            runs_parameters=(),
        )

    def load_latest_runs(self) -> typing.List["DB.Run"]:
        return self._load_runs_with_filter(
            samples_where_clause="",
            samples_parameters=(),
            runs_id_selector="MAX(id)",
            runs_where_clause="GROUP BY hostname, project, toolchain_label, benchmark_name",
            runs_parameters=(),
        )

    def load_runs_by_ids(
        self, run_ids: typing.Sequence["DB.RunID"]
    ) -> typing.List["DB.Run"]:
        return self._load_runs_with_filter(
            samples_where_clause=f"WHERE run_id IN ({', '.join('?' for _ in run_ids)})",
            samples_parameters=tuple(run_ids),
            runs_where_clause=f"WHERE id IN ({', '.join('?' for _ in run_ids)})",
            runs_parameters=tuple(run_ids),
        )

    def _load_runs_with_filter(
        self,
        samples_where_clause: str,
        samples_parameters,
        runs_where_clause: str,
        runs_parameters,
        runs_id_selector: str = "id",
    ) -> typing.List["DB.Run"]:
        cursor = self._connection.cursor()

        raw_samples = cursor.execute(
            f"""
                SELECT run_id, duration_ns
                FROM sample
                {samples_where_clause}
            """,
            samples_parameters,
        ).fetchall()
        run_samples = collections.defaultdict(list)
        for (run_id, duration_ns) in raw_samples:
            run_samples[run_id].append(duration_ns)

        raw_runs = cursor.execute(
            f"""
                SELECT {runs_id_selector} AS run_id, hostname, project, toolchain_label, benchmark_name
                FROM run
                {runs_where_clause}
            """,
            runs_parameters,
        ).fetchall()
        runs = [
            DB.Run(
                id=run_id,
                hostname=hostname,
                project=project,
                toolchain_label=toolchain_label,
                benchmark_name=benchmark_name,
                samples=tuple(run_samples[run_id]),
            )
            for (
                run_id,
                hostname,
                project,
                toolchain_label,
                benchmark_name,
            ) in raw_runs
        ]

        return runs

    def dump_runs(self, runs: typing.List["DB.Run"]) -> None:
        column_names = (
            "hostname",
            "project",
            "toolchain",
            "benchmark",
            "min(ms)",
            "avg(ms)",
            "max(ms)",
        )
        rows = [
            (
                run.hostname,
                run.project,
                run.toolchain_label,
                run.benchmark_name,
                ns_to_ms(min(run.samples)) if run.samples else "---",
                ns_to_ms(avg(run.samples)) if run.samples else "---",
                ns_to_ms(max(run.samples)) if run.samples else "---",
            )
            for run in runs
        ]

        column_widths = [
            max([len(str(row[i])) for row in rows] + [len(column_names[i])])
            for i in range(len(column_names))
        ]

        def print_row(row: typing.Tuple[typing.Any, ...]) -> typing.Tuple[str, ...]:
            print(" | ".join(format_cell(row[i], i) for i in range(len(row))))

        def format_cell(cell, column_index: int) -> str:
            width = column_widths[column_index]
            if isinstance(cell, (int, float)):
                return str(cell).rjust(width)
            else:
                return str(cell).ljust(width)

        print_row(column_names)
        for row in rows:
            print_row(row)


class Lister:
    _test_cases: typing.List[str]

    def __init__(self):
        super().__init__()
        self._test_cases = []

    def profile(self, benchmark: Benchmark) -> None:
        self._test_cases.append(benchmark.full_name)

    def dump_results(self) -> None:
        for test_case in self._test_cases:
            print(test_case)


class Profiler:
    _warmup_iterations: int
    _iterations: int
    _db: DB
    _run_ids: typing.List[DB.RunID]

    def __init__(self, warmup_iterations: int, iterations: int, db: DB) -> None:
        super().__init__()
        self._warmup_iterations = warmup_iterations
        self._iterations = iterations
        self._db = db
        self._run_ids = []

    def profile(self, benchmark: Benchmark) -> None:
        run_id = self._db.create_run(
            hostname=HOSTNAME,
            project=benchmark.project,
            toolchain_label=benchmark.toolchain_label,
            benchmark_name=benchmark.name,
        )
        self._run_ids.append(run_id)

        benchmark.before_all_untimed()

        for _ in range(self._warmup_iterations):
            self._profile_one(benchmark, run_id=None)
        for _ in range(self._iterations):
            self._profile_one(benchmark, run_id=run_id)

        benchmark.after_all_untimed()

    def _profile_one(
        self, benchmark: Benchmark, run_id: typing.Optional[DB.RunID]
    ) -> None:
        benchmark.before_each_untimed()

        before_ns = time.monotonic_ns()
        benchmark.run_timed()
        after_ns = time.monotonic_ns()

        benchmark.after_each_untimed()

        if run_id is not None:
            duration_ns: NanosecondDuration = after_ns - before_ns
            self._db.add_sample_to_run(run_id=run_id, duration_ns=duration_ns)

    def dump_results(self) -> None:
        self._db.dump_runs(self._db.load_runs_by_ids(self._run_ids))


class Filterer:
    _profiler: typing.Union[Profiler, Lister]
    _filter: str

    def __init__(self, profiler: typing.Union[Profiler, Lister], filter: str):
        self._profiler = profiler
        self._filter = filter

    def profile(self, benchmark: Benchmark) -> None:
        if re.search(self._filter, benchmark.full_name):
            self._profiler.profile(benchmark)

    def timed(self):
        return self._profiler.timed()

    def dump_results(self) -> None:
        self._profiler.dump_results()


def rustup_which(command: str, *, toolchain: str) -> pathlib.Path:
    return pathlib.Path(
        subprocess.check_output(
            ["rustup", "which", "--toolchain", toolchain, "--", command],
            encoding="utf-8",
        ).rstrip()
    )


def format_ns(ns: NanosecondDuration) -> str:
    return f"{ns_to_ms(ns)}ms"


def ns_to_ms(ns: NanosecondDuration) -> MillisecondDuration:
    return int(math.ceil(ns / 1e6))


def avg(xs):
    return sum(xs) / len(xs)


def delete_dir(dir: pathlib.Path) -> None:
    try:
        shutil.rmtree(dir)
    except FileNotFoundError:
        pass


cache_bust = 1


def mutate_file(path: pathlib.Path) -> None:
    global cache_bust
    cache_bust += 1

    # Add a line at the top. This will force debug info to change.
    old_text = path.read_text()
    new_text = f"// CACHE-BUST:{cache_bust}\n{old_text}"
    path.write_text(new_text)


def unmutate_file(path: pathlib.Path) -> None:
    # Add a line at the top. This will force debug info to change.
    old_text = path.read_text()
    lines = old_text.splitlines()
    lines = [l for l in lines if not re.match(r"^// CACHE-BUST:", l)]
    new_text = "\n".join(lines) + "\n"
    path.write_text(new_text)


def find_unique_file(root: pathlib.Path, name: str) -> pathlib.Path:
    paths = list(root.glob(f"**/{name}"))
    if not paths:
        raise Exception(f"failed to find file {name} in {root}")
    if len(paths) != 1:
        raise Exception(f"found too many files named {name} in {root}")
    return paths[0]


def cxx_compiler_builds(cxx_compiler: pathlib.Path, flags: str) -> bool:
    try:
        result = subprocess.run(
            [cxx_compiler, "-x", "c++", "-", "-o", "/dev/null"]
            + [flag for flag in flags.split(" ") if flag],
            input=b"#include <version>\nint main(){}",
            stdout=subprocess.DEVNULL,
            stderr=subprocess.STDOUT,
        )
        return result.returncode == 0
    except FileNotFoundError:
        # Compiler does not exist.
        return False


class TestDB(unittest.TestCase):
    def test_load_run_with_no_samples(self) -> None:
        db = DB(path=None)
        run_id = db.create_run("myhostname", "myproject", "mytoolchain", "mybenchmark")
        runs = db.load_runs_by_ids([run_id])
        self.assertEqual(len(runs), 1)
        self.assertEqual(runs[0].hostname, "myhostname")
        self.assertEqual(runs[0].project, "myproject")
        self.assertEqual(runs[0].toolchain_label, "mytoolchain")
        self.assertEqual(runs[0].benchmark_name, "mybenchmark")
        self.assertEqual(runs[0].samples, ())

    def test_load_run_with_some_samples(self) -> None:
        db = DB(path=None)
        run_id = db.create_run("myhostname", "myproject", "mytoolchain", "mybenchmark")
        db.add_sample_to_run(run_id=run_id, duration_ns=100)
        db.add_sample_to_run(run_id=run_id, duration_ns=200)
        db.add_sample_to_run(run_id=run_id, duration_ns=300)
        runs = db.load_runs_by_ids([run_id])
        self.assertEqual(len(runs), 1)
        self.assertEqual(runs[0].samples, (100, 200, 300))

    def test_load_latest_runs_with_no_obsoleted_runs(self) -> None:
        db = DB(path=None)
        # fmt: off
        first_run_id                     = db.create_run("myhostname",  "myproject",  "mytoolchain",  "mybenchmark" )
        different_hostname_run_id        = db.create_run("myhostname2", "myproject",  "mytoolchain",  "mybenchmark" )
        different_project_run_id        = db.create_run("myhostname",  "myproject2", "mytoolchain",  "mybenchmark" )
        different_toolchain_label_run_id = db.create_run("myhostname",  "myproject",  "mytoolchain2", "mybenchmark" )
        different_benchmark_name_run_id  = db.create_run("myhostname",  "myproject",  "mytoolchain",  "mybenchmark2")
        # fmt: on
        runs = db.load_latest_runs()
        self.assertEqual(
            sorted(run.id for run in runs),
            sorted(
                (
                    first_run_id,
                    different_hostname_run_id,
                    different_project_run_id,
                    different_toolchain_label_run_id,
                    different_benchmark_name_run_id,
                )
            ),
        )

    def test_load_latest_runs_with_obsoleted_run(self) -> None:
        db = DB(path=None)
        _run_1_id = db.create_run(
            "myhostname", "myproject", "mytoolchain", "mybenchmark"
        )
        run_2_id = db.create_run(
            "myhostname", "myproject", "mytoolchain", "mybenchmark"
        )
        runs = db.load_latest_runs()
        self.assertEqual(len(runs), 1)
        self.assertEqual(runs[0].id, run_2_id)

    def test_load_all_runs_includes_obsoleted_runs(self) -> None:
        db = DB(path=None)
        run_1_id = db.create_run(
            "myhostname", "myproject", "mytoolchain", "mybenchmark"
        )
        run_2_id = db.create_run(
            "myhostname", "myproject", "mytoolchain", "mybenchmark"
        )
        runs = db.load_all_runs()
        self.assertEqual(sorted(run.id for run in runs), sorted((run_1_id, run_2_id)))


if __name__ == "__main__":
    main()
