#!/usr/bin/env python3
"""Behavioral checks for run.py against stub compiler and benchmark processes.

A stub `rustc` on PATH writes a Python stand-in for every output binary. The
stand-in answers the test/demo runs with plain text and the bench runs with the
JSON line run.py parses. A candidate named in STUB_HANG_CANDIDATE sleeps instead
of answering, so the runner's process bound is observable.
Run from anywhere: python3 topics/032-topological-order-and-incremental-dags/scripts/test_run.py
"""
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

RUNNER = pathlib.Path(__file__).resolve().with_name("run.py")

RUSTC = """#!/usr/bin/env python3
import os, stat, sys
args = sys.argv[1:]
if '-vV' in args:
    print('rustc 1.0.0-stub (00000000 2026-01-01)\\nhost: stub'); sys.exit(0)
if '--print' in args:
    print('target_arch="stub"'); sys.exit(0)
out = args[args.index('-o') + 1]
open(out, 'w').write(os.environ['STUB_BINARY'])
os.chmod(out, os.stat(out).st_mode | stat.S_IXUSR)
"""

LSCPU = "#!/usr/bin/env python3\nprint('Architecture: stub')\n"

BINARY = """#!/usr/bin/env python3
import json, os, sys, time
if len(sys.argv) != 4:
    print('ok'); sys.exit(0)
family, candidate, workload = sys.argv[1:]
if candidate == os.environ.get('STUB_HANG_CANDIDATE'):
    time.sleep(60)
print(json.dumps({'ns': 1000, 'operations': 10, 'vertices': 4, 'edges': 3, 'graph_ns': 5, 'checksum': 7, 'oracle': True}))
"""


def run_runner(tmp, extra_env):
    bin_dir = pathlib.Path(tmp, "bin")
    bin_dir.mkdir()
    for name, body in [("rustc", RUSTC), ("lscpu", LSCPU)]:
        tool = bin_dir / name
        tool.write_text(body)
        tool.chmod(0o755)
    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"], STUB_BINARY=BINARY)
    env.pop("STUB_HANG_CANDIDATE", None)
    env.update(extra_env)
    out = pathlib.Path(tmp, "out")
    try:
        process = subprocess.run([sys.executable, str(RUNNER), "--out", str(out), "--source-commit", "stub"],
                                 env=env, text=True, capture_output=True, timeout=30)
    except subprocess.TimeoutExpired:
        process = None
    return process, out


def attempts(out):
    path = out / "attempts.jsonl"
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


class RunTests(unittest.TestCase):
    def test_hung_candidate_is_retained_as_a_timed_out_attempt(self):
        with tempfile.TemporaryDirectory(prefix="topic032-") as tmp:
            process, out = run_runner(tmp, {"STUB_HANG_CANDIDATE": "dfs", "TOPIC_032_PROCESS_TIMEOUT_S": "1"})
            records = attempts(out)
            if process is None:
                self.fail(f"run.py never bounded the hung candidate; retained attempts: {records!r}")
            self.assertNotEqual(process.returncode, 0, process.stdout + process.stderr)
            self.assertEqual([r["candidate"] for r in records], ["fifo", "dfs"])
            timed_out = records[-1]
            self.assertEqual(timed_out["returncode"], "timeout")
            self.assertEqual(timed_out["timeout_s"], 1.0)
            self.assertEqual((timed_out["family"], timed_out["workload"], timed_out["warmup"]), ("order", "tiny", True))
            self.assertGreaterEqual(timed_out["process_ns"], 1_000_000_000)
            self.assertFalse((out / "complete.json").exists())
            self.assertEqual(json.loads((out / "environment.json").read_text())["timeout_s"], 1.0)

    def test_complete_campaign_retains_every_declared_attempt(self):
        with tempfile.TemporaryDirectory(prefix="topic032-") as tmp:
            process, out = run_runner(tmp, {})
            if process is None:
                self.fail("run.py did not finish the stub campaign within 30 s")
            self.assertEqual(process.returncode, 0, process.stdout + process.stderr)
            records = attempts(out)
            self.assertEqual(len(records), 259)
            self.assertEqual(sum(r["warmup"] for r in records), 37)
            self.assertTrue(all(r["returncode"] == 0 for r in records))
            self.assertEqual(json.loads((out / "complete.json").read_text())["records"], 259)


if __name__ == "__main__":
    unittest.main()
