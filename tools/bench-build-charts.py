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
    make_chart_mold_vs_default_linker(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cranelift_vs_llvm(all_runs=latest_runs, output_dir=output_dir)
    make_chart_optimized_rustc_flags(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cargo_nextest(all_runs=latest_runs, output_dir=output_dir)
    make_chart_rust_layouts(all_runs=latest_runs, output_dir=output_dir)
    make_chart_rust_toolchains(all_runs=latest_runs, output_dir=output_dir)
    make_chart_cpp_vs_rust(all_runs=latest_runs, output_dir=output_dir)


def make_chart_mold_vs_default_linker(
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
            ),
        )
    chart = BarChart(
        name="Linker: Mold barely beats default",
        subtitle="tested on Linux. lower is better.",
        groups=[
            BarChartGroup(name=group_name, bars=group_bars)
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(chart=chart, path=output_dir / "mold-vs-default-linker.svg")


def make_chart_cranelift_vs_llvm(
    all_runs: typing.List, output_dir: pathlib.Path
) -> None:
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
            ),
        )
    chart = BarChart(
        name="Rust backend: LLVM (default) beats Cranelift",
        subtitle="lower is better.",
        groups=[
            BarChartGroup(name=group_name, bars=group_bars)
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
        group_bars_by_name[munge_benchmark_name(run.benchmark_name)].append(
            BarChartBar(
                name={
                    "Rust Stable": "debug",
                    "Rust Stable quick-build-incremental": "quick, incremental=true",
                    "Rust Stable quick-build-nonincremental": "quick, incremental=false",
                }[run.toolchain_label],
                value=avg(run.samples),
                min=min(run.samples),
                max=max(run.samples),
            ),
        )
    chart = BarChart(
        name="rustc flags: quick build beats debug build",
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
                ),
            )
        chart = BarChart(
            name=f"Rust {'incremental' if is_incremental_chart else 'full'} builds: {'best layout is unclear' if is_incremental_chart else 'workspace layout is fastest'}",
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
    runs = [
        run
        for run in all_runs
        if run.hostname == "strapurp"
        and run.project == "rust"
        and run.toolchain_label
        in (
            "Rust Nightly Mold quick-build-incremental cargo-nextest",
            "Rust Nightly Mold quick-build-incremental",
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
            ),
        )
    chart = BarChart(
        name="cargo-nextest does not speed up build+test",
        subtitle="tested on Linux. lower is better.",
        groups=[
            BarChartGroup(name=group_name, bars=group_bars)
            for group_name, group_bars in group_bars_by_name.items()
        ],
    )
    write_chart(chart=chart, path=output_dir / "cargo-nextest.svg")


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


def make_chart_cpp_vs_rust(all_runs: typing.List, output_dir: pathlib.Path) -> None:
    for hostname in ("strammer.lan", "strapurp"):
        if hostname == "strapurp":
            toolchains = {
                "Rust Nightly Mold quick-build-incremental": "Rust Nightly",
                "Clang Custom PGO BOLT libstdc++ PCH Mold -fpch-instantiate-templates": "Clang libstdc++",
                "Clang Custom PGO BOLT libc++ PCH Mold -fpch-instantiate-templates": "Clang libc++",
                "GCC 12 PCH -g0 Mold": "GCC",
            }
            toolchain_order = [
                "Rust Nightly",
                "Clang libstdc++",
                "Clang libc++",
                "GCC",
            ]
        else:
            toolchains = {
                "Rust Nightly quick-build-incremental": "Rust Nightly",
                "Clang libc++ PCH -g0 -fpch-instantiate-templates": "Clang",
            }
            toolchain_order = [
                "Rust Nightly",
                "Clang",
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
            group_bars_by_name[
                munge_benchmark_name_portable(run.benchmark_name)
            ].append(
                BarChartBar(
                    name=toolchains[run.toolchain_label],
                    value=avg(run.samples),
                    min=min(run.samples),
                    max=max(run.samples),
                    emphasize=toolchains[run.toolchain_label] == "Rust Nightly",
                ),
            )
        chart = BarChart(
            name=f"C++ vs Rust build times",
            subtitle=f"tested on {'Linux' if hostname == 'strapurp' else 'macOS'}. lower is better.",
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
        svg_writer.axis_lines()

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
        self.group_gap = 5
        self.bar_value_labels_gap = 2
        self.bar_value_labels_width = 30
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
            self.graph_width - (self.bar_value_labels_gap + self.bar_value_labels_width)
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

        classes = []
        if bar.emphasize:
            classes.append("emphasize-bar")

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
        average_width_per_character = 7
        if any(
            len(cur_bar.name) * average_width_per_character > value_label_x_offset
            for cur_bar in group.bars
        ):
            label_x_offset = value_label_x_offset + 5

        self.svg.write(
            f"""
                <rect
                    class="bar {' '.join(classes)}"
                    style="fill:#ff3cff"
                    width="{self._bar_width(bar.value)}"
                    height="{self.bar_height}"
                    x="{self.graph_left}"
                    y="{y}" />

                <!-- vertical error bar -->
                <rect
                    class="error-bar {' '.join(classes)}"
                    width="{self._bar_width(bar.max - bar.min)}"
                    height="{self.error_bar_thickness}"
                    x="{self.graph_left + self._bar_width(bar.min)}"
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

    def axis_lines(self) -> None:
        self.svg.write(
            f"""
                <rect
                    style="fill:#ff3cff"
                    width="1"
                    height="{self.graph_bottom - self.graph_top}"
                    x="{self.graph_left}"
                    y="{self.graph_top}" />
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

        rect.bar.emphasize-bar {{
            fill: blue;
        }}
        text.bar-label,
        text.bar-value {{
            font-size:{self.bar_height*0.8}px;
        }}
        text.bar-value.emphasize-bar,
        text.bar-label.emphasize-bar {{
            font-weight: bold;
        }}

        rect.error-bar {{
            fill-color: black;
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
        "build and test only my code": "build\nw/o deps",
        "full build and test": "build\nw/ deps",
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
