#!/usr/bin/env python3
"""Behavioral checks for run.py and native.sh against a stub measured process.

The stub answers every quality call with nine identical rows and every timed
call with one fixed line, so run.py completes its full schedule without the
Rust binary. The native.sh tests put a stub `rustc` on PATH that writes the
same stub to every `-o` path. Each test works in its own scratch directory.
Run on Linux:
python3 topics/062-inverted-file-vector-search-and-coarse-quantization/benches/test_run.py
"""
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

BENCHES = pathlib.Path(__file__).resolve().parent
OUTPUTS = ["processes.csv", "summary.json"] + [
    cell + "-quality.csv" for cell in ("tiny", "balanced", "boundary", "skew")
]
# 4 cells, 17 blocks of 8 slots each, plus the header row.
PROCESS_ROWS = 4 * 17 * 8 + 1

# No arguments: the contract-test or demo binary. Two arguments: a measured process.
STUB = """#!/bin/sh
echo "$*" >> "$STUB_LOG"
[ -n "$STUB_FAIL" ] && exit 3
[ $# -eq 0 ] && { echo 'stub pass'; exit 0; }
if [ "$2" = quality ]; then
    for mode in flat raw1 raw4 pq1 pq4 rr1 rr4 raw_all pq_all; do
        echo "$mode,480,5,96,1,0,7"
    done
else
    echo "1000,10,96,7"
fi
"""

RUSTC = """#!/bin/sh
while [ $# -gt 1 ]; do
    [ "$1" = -o ] && { cp "$STUB_PROCESS" "$2"; exit; }
    shift
done
echo 'rustc stub'
"""


class RunnerTest(unittest.TestCase):
    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.dir = pathlib.Path(scratch.name)
        self.out = self.dir / "evidence"
        self.log = self.dir / "log"
        self.binary = self.dir / "binary"
        self.binary.write_text(STUB)
        self.binary.chmod(0o755)

    def run_py(self, **extra):
        env = dict(os.environ, STUB_LOG=str(self.log), **extra)
        command = [sys.executable, str(BENCHES / "run.py"), str(self.binary), str(self.out)]
        return subprocess.run(command, capture_output=True, text=True, env=env, timeout=300, check=False)

    def snapshot(self):
        return {p.name: p.read_bytes() for p in sorted(self.out.iterdir())}

    def test_existing_destination_is_refused_unchanged(self):
        # A finished run, a partial run, and an aborted native attempt that
        # holds environment files only.
        for names in (["processes.csv", "summary.json"], ["tiny-quality.csv"], ["host.txt", "image.sha256"]):
            with self.subTest(existing=names):
                shutil.rmtree(self.out, ignore_errors=True)
                self.out.mkdir()
                for name in names:
                    (self.out / name).write_text("earlier attempt\n")
                before = self.snapshot()
                result = self.run_py()
                self.assertNotEqual(result.returncode, 0, "an existing destination was accepted")
                self.assertEqual(self.snapshot(), before, "the earlier attempt's files changed")
                self.assertFalse(self.log.exists(), "the measured binary was launched")

    def test_fresh_destination_runs_to_summary(self):
        result = self.run_py()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(sorted(self.snapshot()), sorted(OUTPUTS))
        rows = (self.out / "processes.csv").read_text().splitlines()
        self.assertEqual(len(rows), PROCESS_ROWS)
        # query_ns holds the stub's whole-batch time beside its search count.
        self.assertEqual(rows[0], "cell,block,position,mode,query_ns,build_ns,queries,signature")
        self.assertEqual(rows[1], "tiny,-1,0,flat,1000,10,96,7")

    def test_failed_process_leaves_no_summary(self):
        result = self.run_py(STUB_FAIL="1")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.out / "summary.json").exists())

    def topic(self):
        """A scratch topic directory holding the two scripts and placeholder sources."""
        topic = self.dir / "topic"
        for name in ("src/lib.rs", "examples/demo.rs", "tests/contracts.rs", "benches/search.rs"):
            (topic / name).parent.mkdir(parents=True, exist_ok=True)
            (topic / name).write_text("placeholder\n")
        for name in ("run.py", "native.sh"):
            shutil.copy(BENCHES / name, topic / "benches" / name)
        self.out = topic / "evidence"
        return topic

    def native(self, topic, cpus=None):
        """Run native.sh with a stub rustc that writes the stub process to every -o path."""
        tools = self.dir / "tools"
        tools.mkdir()
        (tools / "rustc").write_text(RUSTC)
        (tools / "rustc").chmod(0o755)
        env = dict(os.environ, PATH=f"{tools}{os.pathsep}{os.environ['PATH']}",
                   STUB_LOG=str(self.log), STUB_PROCESS=str(self.binary))
        restrict = (lambda: os.sched_setaffinity(0, cpus)) if cpus else None
        return subprocess.run(["bash", "benches/native.sh"], cwd=topic, capture_output=True,
                              text=True, env=env, timeout=300, check=False, preexec_fn=restrict)

    def test_native_refuses_existing_evidence_before_any_write(self):
        topic = self.topic()
        self.out.mkdir()
        for name in ("host.txt", "run.log", "summary.json"):
            (self.out / name).write_text("earlier attempt\n")
        before = self.snapshot()
        result = self.native(topic)
        self.assertNotEqual(result.returncode, 0, "a second replay was accepted")
        self.assertEqual(self.snapshot(), before, "the earlier attempt's files changed")
        self.assertFalse(self.log.exists(), "the replay continued past the evidence check")

    def test_native_writes_runner_output_below_environment_files(self):
        topic = self.topic()
        result = self.native(topic)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(sorted(p.name for p in (self.out / "run").iterdir()), sorted(OUTPUTS))
        self.assertEqual(
            sorted(p.name for p in self.out.iterdir()),
            ["contracts.log", "demo.log", "host.txt", "image.sha256", "library.s", "run", "run.log", "source.sha256"],
        )

    def test_native_records_the_inherited_cpu_list(self):
        cpu = min(os.sched_getaffinity(0))
        topic = self.topic()
        result = self.native(topic, cpus={cpu})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"Cpus_allowed_list:\t{cpu}\n", (self.out / "host.txt").read_text())


if __name__ == "__main__":
    unittest.main()
