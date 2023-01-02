import collections
import unittest
import math
import pathlib
import sqlite3
import typing

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


def format_ns(ns: NanosecondDuration) -> str:
    return f"{ns_to_ms(ns)}ms"


def ns_to_ms(ns: NanosecondDuration) -> MillisecondDuration:
    return int(math.ceil(ns / 1e6))


def avg(xs):
    return sum(xs) / len(xs)
