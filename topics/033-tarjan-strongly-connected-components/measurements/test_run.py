#!/usr/bin/env python3
"""Behavioral checks for run.py against stub compiler and benchmark processes.

A stub `rustc` on PATH writes a Python stand-in for every output binary; a stub
`taskset` runs the pinned command directly. The stand-in answers the contract and
demo runs with plain text and the bench runs with the JSON line run.py parses. A
candidate named in STUB_HANG_CANDIDATE sleeps instead of answering, so the
runner's process bound is observable; one named in STUB_GARBAGE_CANDIDATE exits 0
with unparsable stdout.
Run from anywhere: python3 topics/033-tarjan-strongly-connected-components/measurements/test_run.py
"""
import json
import os
import pathlib
import signal
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

TASKSET = """#!/usr/bin/env python3
import os, sys
os.execv(sys.argv[3], sys.argv[3:])
"""

BINARY = """#!/usr/bin/env python3
import json, os, sys, time
if len(sys.argv) != 3:
    print('ok'); sys.exit(0)
workload, candidate = sys.argv[1:]
if candidate == os.environ.get('STUB_HANG_CANDIDATE'):
    time.sleep(600)
if candidate == os.environ.get('STUB_GARBAGE_CANDIDATE'):
    print('not json'); sys.exit(0)
print(json.dumps({'workload': workload, 'candidate': candidate, 'vertices': 7, 'edges': 8, 'components': 4,
                  'reps': 1, 'ns_total': 1000, 'ns_per_op': 1000.0, 'certified': True}))
"""


def run_runner(tmp, extra_env, bound_s, argv=None):
    bin_dir = pathlib.Path(tmp, "bin")
    bin_dir.mkdir(exist_ok=True)
    for name, body in [("rustc", RUSTC), ("taskset", TASKSET)]:
        tool = bin_dir / name
        tool.write_text(body)
        tool.chmod(0o755)
    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"], STUB_BINARY=BINARY)
    for name in ["STUB_HANG_CANDIDATE", "STUB_GARBAGE_CANDIDATE"]:
        env.pop(name, None)
    env.update(extra_env)
    out = pathlib.Path(tmp, "out")
    process = subprocess.Popen([sys.executable, str(RUNNER), *([str(out)] if argv is None else argv)], cwd=tmp, env=env, text=True,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    try:
        stdout, stderr = process.communicate(timeout=bound_s)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait()
        return None, "", out
    return process.returncode, stdout + stderr, out


def records(out):
    path = out / "records.jsonl"
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


class RunTests(unittest.TestCase):
    def test_hung_candidate_is_retained_as_a_timed_out_record(self):
        with tempfile.TemporaryDirectory(prefix="topic033-") as tmp:
            code, output, out = run_runner(tmp, {"STUB_HANG_CANDIDATE": "gabow", "TOPIC_033_PROCESS_TIMEOUT_S": "1"}, 30)
            retained = records(out)
            if code is None:
                self.fail(f"run.py never bounded the hung candidate; retained records: {retained!r}")
            self.assertNotEqual(code, 0, output)
            self.assertEqual([(r["workload"], r["candidate"]) for r in retained], [("tiny-first", "tarjan"), ("tiny-first", "gabow")])
            timed_out = retained[-1]
            self.assertEqual(timed_out["exit"], "timeout")
            self.assertEqual(timed_out["timeout_s"], 1.0)
            self.assertEqual((timed_out["phase"], timed_out["block"], timed_out["position"]), ("warmup", -1, 1))
            self.assertGreaterEqual(timed_out["wall_ns"], 1_000_000_000)
            environment = json.loads((out / "environment.json").read_text())
            self.assertNotIn("finished", environment)
            self.assertEqual(len(environment["binary_sha256"]), 64)

    def test_unparsable_success_output_is_retained_and_stops_the_campaign(self):
        with tempfile.TemporaryDirectory(prefix="topic033-") as tmp:
            code, output, out = run_runner(tmp, {"STUB_GARBAGE_CANDIDATE": "kosaraju"}, 30)
            retained = records(out)
            if code is None:
                self.fail("run.py did not stop within 30 s")
            self.assertNotEqual(code, 0, output)
            self.assertEqual([r["candidate"] for r in retained], ["tarjan", "gabow", "kosaraju"])
            self.assertEqual((retained[-1]["exit"], retained[-1]["stdout"]), ("unparsed-stdout", "not json\n"))

    def test_flag_argument_is_refused_before_any_output_directory_exists(self):
        with tempfile.TemporaryDirectory(prefix="topic033-") as tmp:
            code, output, _ = run_runner(tmp, {}, 30, argv=["--help"])
            self.assertNotEqual(code, 0)
            self.assertIn("usage", output)
            self.assertEqual(sorted(p.name for p in pathlib.Path(tmp).iterdir()), ["bin"])
            code, output, _ = run_runner(tmp, {}, 30, argv=[])
            self.assertNotEqual(code, 0)
            self.assertIn("usage", output)

    def test_complete_campaign_retains_every_declared_process(self):
        with tempfile.TemporaryDirectory(prefix="topic033-") as tmp:
            code, output, out = run_runner(tmp, {}, 120)
            if code is None:
                self.fail("run.py did not finish the stub campaign within 120 s")
            self.assertEqual(code, 0, output)
            retained = records(out)
            self.assertEqual(len(retained), 350)
            self.assertEqual(sum(r["phase"] == "warmup" for r in retained), 50)
            self.assertTrue(all(r["exit"] == 0 and r["certified"] for r in retained))
            self.assertIn("finished", json.loads((out / "environment.json").read_text()))


if __name__ == "__main__":
    unittest.main()
