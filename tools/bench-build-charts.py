#!/usr/bin/env python

import math
import collections
import typing
from cpp_vs_rust_db import DB, avg, format_ns
import argparse
import pathlib


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", metavar="PATH", required=True, nargs="+")
    parser.add_argument("--output-dir", metavar="PATH", required=True)
    args = parser.parse_args()

    dbs = [DB(db_path) for db_path in args.db]
    output_dir = pathlib.Path(args.output_dir)

    latest_runs = []
    for db in dbs:
        latest_runs.extend(db.load_latest_runs())

    output_dir.mkdir(exist_ok=True)
    make_chart_rust_linux_linker(all_runs=latest_runs, output_dir=output_dir)
    make_chart_rust_macos_linker(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cranelift_vs_llvm(all_runs=latest_runs, output_dir=output_dir)
    make_chart_optimized_rustc_flags(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cargo_nextest(all_runs=latest_runs, output_dir=output_dir)
    make_chart_rust_layouts(all_runs=latest_runs, output_dir=output_dir)
    make_chart_rust_toolchains(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cpp_toolchains(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cpp_vs_rust(all_runs=latest_runs, output_dir=output_dir)


def make_chart_rust_linux_linker(
    all_runs: typing.List, output_dir: pathlib.Path
) -> None:
    runs = [
        run
        for run in all_runs
        if run.hostname == "strapurp"
        and run.project == "rust"
        and run.toolchain_label in ("Rust Stable Mold", "Rust Stable")
    ]
    group_bars_by_name = collections.defaultdict(list)
    for run in runs:
        if run.benchmark_name == "test only":
            continue
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name="Mold" if "Mold" in run.toolchain_label else "Default",
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
                emphasize="Mold" in run.toolchain_label,
                classes=[] if "Mold" in run.toolchain_label else ["color-default"],
                show_percent_difference=0 if "Mold" in run.toolchain_label else None,
            ),
        )
    chart = BarChart(
        name="Linux: <tspan class='color-1-of-2'>Mold</tspan> barely beats <tspan class='color-default'>default linker</tspan>",
        subtitle="lower is better.",
        groups=[
            BarChartGroup(name=group_name, bars=group_bars)
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(chart=chart, path=output_dir / "rust-linux-linker.svg")


def make_chart_rust_macos_linker(
    all_runs: typing.List, output_dir: pathlib.Path
) -> None:
    bar_order = ["ld64", "lld", "zld"]
    runs = [
        run
        for run in all_runs
        if run.hostname == "strammer.lan"
        and run.project == "rust"
        and run.toolchain_label
        in ("Rust Stable ld64.lld", "Rust Stable zld", "Rust Stable")
    ]
    group_bars_by_name = collections.defaultdict(list)
    for run in runs:
        if run.benchmark_name == "test only":
            continue
        name = (
            "lld"
            if "ld64.lld" in run.toolchain_label
            else "zld"
            if "zld" in run.toolchain_label
            else "ld64"
        )
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name=name,
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
                classes=[f"color-{bar_order.index(name)+1}-of-3"],
            ),
        )
    chart = BarChart(
        name="macOS: linkers perform about the same",
        subtitle="lower is better.",
        groups=[
            BarChartGroup(
                name=group_name,
                bars=sorted(group_bars, key=lambda bar: bar_order.index(bar.name)),
            )
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(chart=chart, path=output_dir / "rust-macos-linker.svg")


def make_chart_cranelift_vs_llvm(
    all_runs: typing.List, output_dir: pathlib.Path
) -> None:
    bar_order = ["LLVM", "Cranelift"]
    runs = [
        run
        for run in all_runs
        if run.hostname == "strapurp"
        and run.project == "rust"
        and run.toolchain_label in ("Rust Nightly", "Rust Cranelift")
    ]
    group_bars_by_name = collections.defaultdict(list)
    for run in runs:
        if run.benchmark_name == "test only":
            continue
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name="Cranelift" if "Cranelift" in run.toolchain_label else "LLVM",
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
                emphasize="Cranelift" in run.toolchain_label,
                classes=[] if "Cranelift" in run.toolchain_label else ["color-default"],
                show_percent_difference=0
                if "Cranelift" in run.toolchain_label
                else None,
            ),
        )
    chart = BarChart(
        name="Rust backend: <tspan class='color-default'>LLVM (default)</tspan> beats <tspan class='color-1-of-2'>Cranelift</tspan>",
        subtitle="lower is better.",
        groups=[
            BarChartGroup(
                name=group_name,
                bars=sorted(group_bars, key=lambda bar: bar_order.index(bar.name)),
            )
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(chart=chart, path=output_dir / "cranelift-vs-llvm.svg")


def make_chart_optimized_rustc_flags(
    all_runs: typing.List, output_dir: pathlib.Path
) -> None:
    runs = [
        run
        for run in all_runs
        if run.hostname == "strapurp"
        and run.project == "rust"
        and run.toolchain_label
        in (
            "Rust Stable",
            "Rust Stable quick-build-incremental",
            "Rust Stable quick-build-nonincremental",
        )
    ]
    group_bars_by_name = collections.defaultdict(list)
    for run in runs:
        if run.benchmark_name in ("test only", "full build and test"):
            continue
        name = {
            "Rust Stable": "debug",
            "Rust Stable quick-build-incremental": "quick, incremental=true",
            "Rust Stable quick-build-nonincremental": "quick, incremental=false",
        }[run.toolchain_label]
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name=name,
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
                classes={
                    "debug": ["color-default"],
                    "quick, incremental=true": ["color-1-of-2"],
                    "quick, incremental=false": [
                        "color-1-of-2",
                        "color-alternate-shade",
                    ],
                }[name],
                show_percent_difference=None if name == "debug" else 0,
            ),
        )
    chart = BarChart(
        name="rustc flags: <tspan class='color-1-of-2'>quick build</tspan> beats <tspan class='color-default'>debug build</tspan>",
        subtitle="lower is better.",
        groups=[
            BarChartGroup(name=group_name, bars=group_bars)
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(
        chart=chart,
        path=output_dir / f"optimized-rustc-flags.svg",
    )


def make_chart_rust_layouts(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    for is_incremental_chart in (True, False):
        projects = {
            "rust": "workspace; test crates",
            "rust-workspace-crateunotest": "workspace; merged test crate",
            "rust-threecrate-cratecargotest": "2 crates; test crates",
            "rust-threecrate-crateunotest": "2 crates; merged test crate",
            "rust-twocrate-cratecargotest": "single crate; test crates",
            "rust-twocrate-unittest": "single crate; tests in lib",
        }
        project_order = [
            "workspace; test crates",
            "workspace; merged test crate",
            "single crate; test crates",
            "single crate; tests in lib",
            "2 crates; test crates",
            "2 crates; merged test crate",
        ]
        runs = [
            run
            for run in all_runs
            if run.hostname == "strapurp"
            and run.project in projects.keys()
            and run.toolchain_label == "Rust Stable quick-build-incremental"
        ]
        group_bars_by_name = collections.defaultdict(list)
        for run in runs:
            if run.benchmark_name in ("test only", "full build and test"):
                continue
            if ("incremental" in run.benchmark_name) != is_incremental_chart:
                continue
            group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
                BarChartBar(
                    name=projects[run.project],
                    value=avg(run.samples),
                    min=min(run.samples),
                    max=max(run.samples),
                    emphasize="workspace" in projects[run.project]
                    and not is_incremental_chart,
                    classes={
                        "workspace; test crates": ["color-1-of-3"],
                        "workspace; merged test crate": [
                            "color-1-of-3",
                            "color-alternate-shade",
                        ],
                        "single crate; test crates": ["color-2-of-3"],
                        "single crate; tests in lib": [
                            "color-2-of-3",
                            "color-alternate-shade",
                        ],
                        "2 crates; test crates": ["color-3-of-3"],
                        "2 crates; merged test crate": [
                            "color-3-of-3",
                            "color-alternate-shade",
                        ],
                    }[projects[run.project]],
                ),
            )
        chart = BarChart(
            name="Rust incremental builds: best layout is unclear"
            if is_incremental_chart
            else "Rust full builds: <tspan class='color-1-of-3'>workspace layout</tspan> is fastest",
            subtitle="lower is better.",
            groups=[
                BarChartGroup(
                    name=group_name,
                    bars=sorted(
                        group_bars, key=lambda bar: project_order.index(bar.name)
                    ),
                )
                for group_name, group_bars in group_bars_by_name.items()
            ],
        )
        write_chart(
            chart=chart,
            path=output_dir
            / f"rust-layouts-{'incremental' if is_incremental_chart else 'full'}.svg",
        )


def make_chart_cargo_nextest(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    for hostname in ("strammer.lan", "strapurp"):
        runs = [
            run
            for run in all_runs
            if run.hostname == hostname
            and run.project == "rust"
            and run.toolchain_label
            in (
                "Rust Nightly quick-build-incremental cargo-nextest",
                "Rust Nightly quick-build-incremental",
            )
        ]
        group_bars_by_name = collections.defaultdict(list)
        for run in runs:
            group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
                BarChartBar(
                    name="cargo-nextest"
                    if "cargo-nextest" in run.toolchain_label
                    else "Default",
                    value=avg(run.samples),
                    min=min(run.samples),
                    max=max(run.samples),
                    emphasize="cargo-nextest" in run.toolchain_label,
                    classes=[]
                    if "cargo-nextest" in run.toolchain_label
                    else ["color-default"],
                    show_percent_difference=0
                    if "cargo-nextest" in run.toolchain_label
                    else None,
                ),
            )
        chart = BarChart(
            name="Linux: <tspan class='color-1-of-2'>cargo-nextest</tspan> slows down build+test"
            if hostname == "strapurp"
            else "macOS: <tspan class='color-1-of-2'>cargo-nextest</tspan> speeds up build+test",
            subtitle="lower is better.",
            groups=[
                BarChartGroup(name=group_name, bars=group_bars)
                for group_name, group_bars in group_bars_by_name.items()
            ],
        )
        write_chart(
            chart=chart,
            path=output_dir
            / f"cargo-nextest-{'linux' if hostname == 'strapurp' else 'macos'}.svg",
        )


def make_chart_rust_toolchains(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    toolchains = {
        "Rust Stable quick-build-incremental": "Stable",
        "Rust Nightly quick-build-incremental": "Nightly",
        "Rust Custom quick-build-incremental": "Custom",
        "Rust Custom PGO quick-build-incremental": "Custom+PGO",
        "Rust Custom PGO BOLT quick-build-incremental": "Custom+PGO+BOLT",
    }
    toolchain_order = ["Stable", "Nightly", "Custom", "Custom+PGO", "Custom+PGO+BOLT"]
    runs = [
        run
        for run in all_runs
        if run.hostname == "strapurp"
        and run.project == "rust"
        and run.toolchain_label in toolchains.keys()
    ]
    group_bars_by_name = collections.defaultdict(list)
    for run in runs:
        if run.benchmark_name not in (
            "full build and test",
            "incremental build and test (lex.rs)",
        ):
            continue
        tc = toolchains[run.toolchain_label]
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name=tc,
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
                emphasize=tc in ("Nightly", "Custom+PGO+BOLT"),
                show_percent_difference=toolchain_order.index("Nightly")
                if tc == "Custom+PGO+BOLT"
                else None,
            ),
        )
    chart = BarChart(
        name="Rust toolchains: Nightly is fastest",
        subtitle="tested on Linux. lower is better.",
        groups=[
            BarChartGroup(
                name=group_name,
                bars=sorted(
                    group_bars, key=lambda bar: toolchain_order.index(bar.name)
                ),
            )
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(
        chart=chart,
        path=output_dir / f"rust-toolchain.svg",
    )


def make_chart_cpp_toolchains(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    for hostname in ("strammer.lan", "strapurp"):
        if hostname == "strapurp":
            toolchains = {
                "Clang Custom PGO BOLT libstdc++ PCH Mold -fpch-instantiate-templates": "Clang (custom) libstdc++",
                "Clang Custom PGO BOLT libc++ PCH Mold -fpch-instantiate-templates": "Clang (custom) libc++",
                "Clang 12 libstdc++ PCH Mold -fpch-instantiate-templates": "Clang (Ubuntu) libstdc++",
                "Clang 12 libc++ PCH Mold -fpch-instantiate-templates": "Clang (Ubuntu) libc++",
                "GCC 12 PCH -g0 Mold": "GCC",
            }
            toolchain_order = [
                "GCC",
                "Clang (Ubuntu) libc++",
                "Clang (Ubuntu) libstdc++",
                "Clang (custom) libc++",
                "Clang (custom) libstdc++",
            ]
        else:
            toolchains = {
                "Clang libc++ PCH -g0 -fpch-instantiate-templates": "Xcode ld64",
                "Clang libc++ PCH -g0 ld64.lld -fpch-instantiate-templates": "Xcode lld",
                "Clang libc++ PCH -g0 zld -fpch-instantiate-templates": "Xcode zld",
                "Clang 15 libc++ PCH -g0 -fpch-instantiate-templates": "Clang 15 ld64",
                "Clang 15 libc++ PCH -g0 ld64.lld -fpch-instantiate-templates": "Clang 15 lld",
                "Clang 15 libc++ PCH -g0 zld -fpch-instantiate-templates": "Clang 15 zld",
            }
            toolchain_order = [
                "Xcode ld64",
                "Xcode lld",
                "Xcode zld",
                "Clang 15 ld64",
                "Clang 15 lld",
                "Clang 15 zld",
            ]
        runs = [
            run
            for run in all_runs
            if run.hostname == hostname
            and run.project == "cpp"
            and run.toolchain_label in toolchains.keys()
        ]
        group_bars_by_name = collections.defaultdict(list)
        for run in runs:
            if run.benchmark_name in ("test only", "full build and test"):
                continue
            group_bars_by_name[
                munge_benchmark_name_portable(run.benchmark_name)
            ].append(
                BarChartBar(
                    name=toolchains[run.toolchain_label],
                    value=avg(run.samples),
                    min=min(run.samples),
                    max=max(run.samples),
                    emphasize="Clang (custom)" in toolchains[run.toolchain_label],
                    classes={
                        "GCC": ["color-1-of-3"],
                        "Clang (Ubuntu) libc++": ["color-2-of-3"],
                        "Clang (Ubuntu) libstdc++": [
                            "color-2-of-3",
                            "color-alternate-shade",
                        ],
                        "Clang (custom) libc++": ["color-3-of-3"],
                        "Clang (custom) libstdc++": [
                            "color-3-of-3",
                            "color-alternate-shade",
                        ],
                        "Xcode ld64": ["color-1-of-2"],
                        "Xcode lld": ["color-1-of-2", "color-alternate-shade"],
                        "Xcode zld": ["color-1-of-2", "color-alternate-shade-2"],
                        "Clang 15 ld64": ["color-2-of-2"],
                        "Clang 15 lld": ["color-2-of-2", "color-alternate-shade"],
                        "Clang 15 zld": ["color-2-of-2", "color-alternate-shade-2"],
                    }[toolchains[run.toolchain_label]],
                ),
            )
        chart = BarChart(
            name="Linux: <tspan class='color-3-of-3'>custom Clang</tspan> is fastest toolchain"
            if hostname == "strapurp"
            else "macOS: <tspan class='color-1-of-2'>Xcode</tspan> is fastest toolchain",
            subtitle=f"lower is better.",
            groups=[
                BarChartGroup(
                    name=group_name,
                    bars=sorted(
                        group_bars, key=lambda bar: toolchain_order.index(bar.name)
                    ),
                )
                for group_name, group_bars in group_bars_by_name.items()
            ],
        )
        write_chart(
            chart=chart,
            path=output_dir
            / f"cpp-toolchains-{'linux' if hostname == 'strapurp' else 'macos'}.svg",
        )


def make_chart_cpp_vs_rust(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    for hostname in ("strammer.lan", "strapurp"):
        if hostname == "strapurp":
            toolchains = {
                "Rust Nightly Mold quick-build-incremental": "Rust Nightly",
                "Clang Custom PGO BOLT libstdc++ PCH Mold -fpch-instantiate-templates": "C++ Clang libstdc++",
                "Clang Custom PGO BOLT libc++ PCH Mold -fpch-instantiate-templates": "C++ Clang libc++",
            }
            toolchain_order = [
                "Rust Nightly",
                "C++ Clang libstdc++",
                "C++ Clang libc++",
            ]
        else:
            toolchains = {
                "Rust Nightly quick-build-incremental cargo-nextest": "Rust Nightly cargo-nextest",
                "Clang libc++ PCH -g0 -fpch-instantiate-templates": "C++ Clang",
            }
            toolchain_order = [
                "Rust Nightly cargo-nextest",
                "C++ Clang",
            ]
        runs = [
            run
            for run in all_runs
            if run.hostname == hostname
            and run.project in ("rust", "cpp")
            and run.toolchain_label in toolchains.keys()
        ]
        group_bars_by_name = collections.defaultdict(list)
        for run in runs:
            if run.benchmark_name in ("test only", "full build and test"):
                continue
            emphasize = (
                toolchains[run.toolchain_label] == "Rust Nightly"
                if hostname == "strapurp"
                else toolchains[run.toolchain_label] == "C++ Clang"
            )
            name = toolchains[run.toolchain_label]
            group_bars_by_name[
                munge_benchmark_name_portable(run.benchmark_name)
            ].append(
                BarChartBar(
                    name=name,
                    value=avg(run.samples),
                    min=min(run.samples),
                    max=max(run.samples),
                    emphasize=emphasize,
                    classes=["color-1-of-2" if "Rust" in name else "color-2-of-2"]
                    + (["color-alternate-shade"] if "libc++" in name else []),
                    show_percent_difference=0 if "C++" in name else None,
                ),
            )
        chart = BarChart(
            name="Linux: <tspan class='color-1-of-2'>Rust</tspan> usually builds faster than <tspan class='color-2-of-2'>C++</tspan>"
            if hostname == "strapurp"
            else "macOS: <tspan class='color-2-of-2'>C++</tspan> usually builds faster than <tspan class='color-1-of-2'>Rust</tspan>",
            subtitle=f"lower is better.",
            groups=[
                BarChartGroup(
                    name=group_name,
                    bars=sorted(
                        group_bars, key=lambda bar: toolchain_order.index(bar.name)
                    ),
                )
                for group_name, group_bars in group_bars_by_name.items()
            ],
        )
        write_chart(
            chart=chart,
            path=output_dir
            / f"cpp-vs-rust-{'linux' if hostname == 'strapurp' else 'macos'}.svg",
        )


class BarChart(typing.NamedTuple):
    name: str
    subtitle: str
    groups: typing.List["BarChartGroup"]
    force_maximum_value: typing.Optional[float] = None

    @property
    def maximum_value(self) -> float:
        if self.force_maximum_value is not None:
            return self.force_maximum_value
        return max(bar.max for group in self.groups for bar in group.bars)


class BarChartBar(typing.NamedTuple):
    name: str
    value: float
    min: float
    max: float
    emphasize: bool = False
    # Index to the bar to compare to.
    show_percent_difference: typing.Optional[int] = None
    classes: typing.List[str] = []


class BarChartGroup(typing.NamedTuple):
    name: str
    bars: typing.List["BarChartBar"]


def write_chart(chart: BarChart, path: pathlib.Path) -> None:
    with open(path, "w") as svg:
        svg_writer = BarChartWriter(
            svg=svg,
            chart=chart,
        )
        svg_writer.svg_header()

        for group_index, group in enumerate(chart.groups):
            svg_writer.write_group_label(
                group=group,
                group_index=group_index,
            )
            for bar_index, bar in enumerate(group.bars):
                svg_writer.write_bar(
                    bar=bar,
                    group_index=group_index,
                    bar_index=bar_index,
                )
        svg_writer.svg_footer()


class BarChartWriter:
    def __init__(self, svg, chart: BarChart) -> None:
        self.svg = svg
        self.chart = chart

        self.x_labels_height = 0
        self.y_labels_width = 60

        self.title_height = 20
        self.subtitle_height = 15
        self.title_gap = 10  # Margin between subtitle and chart.

        self.graph_width = 300
        self.graph_y_padding = 4

        self.bar_height = 10
        self.bar_gap = 2
        self.group_gap = 7
        self.bar_value_labels_gap = 2
        self.bar_value_labels_width = 30
        self.bar_value_labels_extra_width = (
            35
            if any(
                bar.show_percent_difference is not None
                for group in chart.groups
                for bar in group.bars
            )
            else 0
        )
        self.bar_value_labels_min_x_offset = 40
        self.error_bar_thickness = 0.75
        self.error_bar_height = self.bar_height / 2.5

        self.graph_left = self.y_labels_width
        self.graph_right = self.graph_left + self.graph_width
        self.graph_top = self.title_height + self.subtitle_height + self.title_gap
        bars = sum(len(group.bars) for group in chart.groups)
        self.graph_bottom = (
            self.graph_top
            + bars * (self.bar_height + self.bar_gap)
            + (len(chart.groups) - 1) * self.group_gap
            + self.graph_y_padding * 2
        )

        self.title_center_x = (
            (self.graph_left - (self.y_labels_width / 2)) + self.graph_right
        ) / 2

        self.image_width = self.graph_width + self.y_labels_width + 2
        self.image_height = self.graph_bottom + self.x_labels_height + 2

        self.x_scale = (
            self.graph_width
            - (
                self.bar_value_labels_gap
                + self.bar_value_labels_width
                + self.bar_value_labels_extra_width
            )
        ) / chart.maximum_value

    def _bar_y(self, group_index: int, bar_index: int) -> None:
        y = self.graph_top + self.graph_y_padding + self.bar_gap
        for cur_group_index, group in enumerate(self.chart.groups):
            if cur_group_index == group_index:
                break
            y += len(group.bars) * (self.bar_height + self.bar_gap) + self.group_gap
        y += bar_index * (self.bar_height + self.bar_gap)
        return y

    def _bar_width(self, value: float) -> None:
        return value * self.x_scale

    def write_group_label(self, group: BarChartGroup, group_index: int) -> None:
        y = (
            self._bar_y(group_index=group_index, bar_index=0)
            + (
                self._bar_y(group_index=group_index, bar_index=len(group.bars) - 1)
                + self.bar_height
            )
        ) / 2
        x = self.graph_left - 3

        tspans = [
            f'<tspan x="{x}" dy="{1.1 if i else 0}em">{line}</tspan>'
            for (i, line) in enumerate(group.name.split("\n"))
        ]
        self.svg.write(
            f"""
                <text
                    class="group"
                    text-anchor="end"
                    x="{self.graph_left - 3}"
                    y="{y}">{"".join(tspans)}</text>
"""
        )

    def write_bar(self, bar: BarChartBar, group_index: int, bar_index: int) -> None:
        group = self.chart.groups[group_index]

        classes = list(bar.classes)
        if bar.emphasize:
            classes.append("emphasize-bar")

        percent_difference_text = ""
        if bar.show_percent_difference is not None:
            baseline = group.bars[bar.show_percent_difference].value
            percent_difference = (bar.value - baseline) / baseline * 100
            if abs(percent_difference) < 10:
                percent_difference_text = f"({percent_difference:+.1f}%)"
            else:
                percent_difference_text = f"({percent_difference:+.0f}%)"

        y = self._bar_y(group_index=group_index, bar_index=bar_index)

        value_label_x_offset = (
            max(self._bar_width(cur_bar.max) for cur_bar in group.bars)
            + self.bar_value_labels_gap
            + self.bar_value_labels_width
        )
        value_label_x_offset = max(
            value_label_x_offset, self.bar_value_labels_min_x_offset
        )

        error_bar_y_offset = y + self.bar_height / 2

        label_x_offset = 3
        average_width_per_character = 6
        if any(
            len(cur_bar.name) * average_width_per_character > value_label_x_offset
            for cur_bar in group.bars
        ):
            label_x_offset = value_label_x_offset + 5
            if any(
                cur_bar.show_percent_difference is not None for cur_bar in group.bars
            ):
                label_x_offset += self.bar_value_labels_extra_width
            classes.append("bar-label-outside-bar")

        self.svg.write(
            f"""
                <rect
                    class="bar {' '.join(classes)}"
                    width="{self._bar_width(bar.value)}"
                    height="{self.bar_height}"
                    x="{self.graph_left}"
                    y="{y}" />

                <!-- horizontal error bar -->
                <rect
                    class="error-bar {' '.join(classes)}"
                    width="{self._bar_width(bar.max - bar.min) - self.error_bar_thickness}"
                    height="{self.error_bar_thickness}"
                    x="{self.graph_left + self._bar_width(bar.min) + self.error_bar_thickness/2}"
                    y="{error_bar_y_offset - self.error_bar_thickness/2}" />
                <!-- left error bar -->
                <rect
                    class="error-bar {' '.join(classes)}"
                    width="{self.error_bar_thickness}"
                    height="{self.error_bar_height}"
                    x="{self.graph_left + self._bar_width(bar.min) - self.error_bar_thickness/2}"
                    y="{error_bar_y_offset - self.error_bar_height/2}" />
                <!-- right error bar -->
                <rect
                    class="error-bar {' '.join(classes)}"
                    width="{self.error_bar_thickness}"
                    height="{self.error_bar_height}"
                    x="{self.graph_left + self._bar_width(bar.max) - self.error_bar_thickness/2}"
                    y="{error_bar_y_offset - self.error_bar_height/2}" />

                <text
                    class="bar-label {' '.join(classes)}"
                    x="{self.graph_left + label_x_offset}"
                    y="{y + self.bar_height - 2}">{bar.name}</text>
                <text
                    class="bar-value {' '.join(classes)}"
                    text-anchor="end"
                    x="{self.graph_left + value_label_x_offset}"
                    y="{y + self.bar_height - 2}">{format_ns(bar.value)}</text>
            """
        )
        if percent_difference_text:
            self.svg.write(
                f"""
                    <text
                        class="bar-value {' '.join(classes)}"
                        text-anchor="start"
                        x="{self.graph_left + value_label_x_offset}"
                        y="{y + self.bar_height - 2}">&#x00a0;{percent_difference_text}</text>
                """
            )

    def svg_header(self) -> None:
        self.svg.write(
            f"""\
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg
    xmlns:svg="http://www.w3.org/2000/svg"
    xmlns="http://www.w3.org/2000/svg"
    width="{self.image_width}"
    height="{self.image_height}"
    viewBox="0 0 {self.image_width} {self.image_height}"
    version="1.1">

    <style>
        text {{
            font-family: sans-serif;
            fill: #000;
        }}
        @media (prefers-color-scheme: dark) {{
            text {{
                fill: #fff;
            }}
        }}

        text.chart-title {{
            font-size:{self.title_height*0.8}px;
        }}
        text.chart-subtitle {{
            font-size:{self.subtitle_height*0.6}px;
            font-style: italic;
        }}

        rect.bar,
        rect.bar.color-default {{
            fill: #444;
        }}
        rect.bar.emphasize-bar {{
            fill: #c33;
        }}
        .chart-title .color-1-of-2,
        .chart-title .color-1-of-3,
        .bar.color-1-of-2,
        .bar.color-1-of-3,
        .bar-label.bar-label-outside-bar.color-1-of-2,
        .bar-label.bar-label-outside-bar.color-1-of-3 {{
            fill: #933;
        }}
        .bar.color-1-of-2.emphasize-bar,
        .bar.color-1-of-3.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-1-of-2.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-1-of-3.emphasize-bar {{
            fill: #c33;
        }}
        .bar.color-1-of-2.color-alternate-shade,
        .bar.color-1-of-3.color-alternate-shade,
        .bar-label.bar-label-outside-bar.color-1-of-2.color-alternate-shade,
        .bar-label.bar-label-outside-bar.color-1-of-3.color-alternate-shade {{
            fill: #a52;
        }}
        .bar.color-1-of-2.color-alternate-shade-2,
        .bar.color-1-of-3.color-alternate-shade-2,
        .bar-label.bar-label-outside-bar.color-1-of-2.color-alternate-shade-2,
        .bar-label.bar-label-outside-bar.color-1-of-3.color-alternate-shade-2 {{
            fill: #915;
        }}
        .chart-title .color-2-of-2,
        .chart-title .color-2-of-3,
        .bar.color-2-of-2,
        .bar.color-2-of-3,
        .bar-label.bar-label-outside-bar.color-2-of-2,
        .bar-label.bar-label-outside-bar.color-2-of-3 {{
            fill: #339;
        }}
        .bar.color-2-of-2.emphasize-bar,
        .bar.color-2-of-3.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-2-of-2.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-2-of-3.emphasize-bar {{
            fill: #33c;
        }}
        .bar.color-2-of-2.color-alternate-shade,
        .bar.color-2-of-3.color-alternate-shade,
        .bar-label.bar-label-outside-bar.color-2-of-2.color-alternate-shade,
        .bar-label.bar-label-outside-bar.color-2-of-3.color-alternate-shade {{
            fill: #52a;
        }}
        .bar.color-2-of-2.color-alternate-shade-2,
        .bar.color-2-of-3.color-alternate-shade-2,
        .bar-label.bar-label-outside-bar.color-2-of-2.color-alternate-shade-2,
        .bar-label.bar-label-outside-bar.color-2-of-3.color-alternate-shade-2 {{
            fill: #25a;
        }}
        .chart-title .color-3-of-3,
        .bar.color-3-of-3,
        .bar-label.bar-label-outside-bar.color-3-of-3 {{
            fill: #393;
        }}
        .bar.color-3-of-3.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-3-of-3.emphasize-bar {{
            fill: #3a3;
        }}
        .bar.color-3-of-3.color-alternate-shade,
        .bar-label.bar-label-outside-bar.color-3-of-3.color-alternate-shade {{
            fill: #891;
        }}
        .bar.color-3-of-3.color-alternate-shade.emphasize-bar,
        .bar-label.bar-label-outside-bar.color-3-of-3.color-alternate-shade.emphasize-bar {{
            fill: #8a1;
        }}
        .chart-title .color-default,
        .bar-label.color-default {{
            fill: #ccc;
        }}

        .bar-label,
        .bar-value {{
            font-size: {self.bar_height*0.8}px;
        }}
        .bar-value.emphasize-bar,
        .bar-label.emphasize-bar {{
            font-weight: bold;
        }}

        rect.error-bar {{
            fill: rgba(0, 0, 0, 0.35);
        }}
        @media (prefers-color-scheme: dark) {{
            rect.error-bar {{
                fill: rgba(255, 255, 255, 0.35);
            }}
        }}

        text.group {{
            font-size:{self.bar_height*0.8}px;
        }}
    </style>

    <text
        class="chart-title"
        text-anchor="middle"
        x="{self.title_center_x}"
        y="{self.title_height}">{self.chart.name}</text>
    <text
        class="chart-subtitle"
        text-anchor="middle"
        x="{self.title_center_x}"
        y="{self.title_height + self.subtitle_height}">{self.chart.subtitle}</text>
"""
        )

    def svg_footer(self) -> None:
        self.svg.write("</svg>\n")


def munge_benchmark_name(benchmark_name: str) -> str:
    return {
        "build and test only my code": "build\nw/o deps",
        "full build and test": "build\nw/ deps",
        "incremental build and test (diagnostic_types.rs)": "incremental\ndiag_types.rs",
        "incremental build and test (lex.rs)": "incremental\nlex.rs",
        "incremental build and test (test_utf_8.rs)": "incremental\ntest_utf_8.rs",
        "test only": "test only",
    }[benchmark_name]


def munge_benchmark_name_portable(benchmark_name: str) -> str:
    return {
        "build and test only my code": "full build\nw/o deps",
        "full build and test": "full build\nw/ deps",
        "incremental build and test (diagnostic_types.rs)": "incremental\ndiag-types",
        "incremental build and test (diagnostic-types.h)": "incremental\ndiag-types",
        "incremental build and test (lex.rs)": "incremental\nlex",
        "incremental build and test (lex.cpp)": "incremental\nlex",
        "incremental build and test (test_utf_8.rs)": "incremental\ntest-utf-8",
        "incremental build and test (test-utf-8.cpp)": "incremental\ntest-utf-8",
        "test only": "test only",
    }[benchmark_name]


if __name__ == "__main__":
    main()
