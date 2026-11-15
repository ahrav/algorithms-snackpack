#!/usr/bin/env python3
"""Behavioral checks for run.py against stub tools and stub measured processes.

Each test copies run.py into a scratch topic directory beside four stub inputs
and runs the copy. A stub `rustc` on PATH writes a stub executable to every `-o`
path, so run.py launches the stub through `taskset` exactly as it launches the
native replay binary. The stub answers come from `model`, a refill simulation of
one meter per flow written from README.md and BENCHMARK.md. It shares no code
with run.py's accepted-cost ledger. The Git tests commit the scratch inputs to a
scratch repository. Run on Linux:
python3 topics/050-token-buckets-and-rate-accounting/benches/test_run.py
"""
import hashlib
import itertools
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

RUN = pathlib.Path(__file__).resolve().parent / "run.py"
MODES = ["wide", "bounded", "virtual"]
CELLS = list(itertools.product([32, 32768], [1, 4096], ["burst", "steady", "idle", "weighted"]))
INPUTS = ["src/lib.rs", "tests/contracts.rs", "examples/running.rs", "benches/replay.rs", "benches/run.py"]
SOURCE = "0" * 40
FIRST_PROCESS = "process wide 32 1 burst 1 details"

RUSTC = """#!/bin/sh
echo "rustc $*" >> "$STUB_LOG"
[ "$1" = -Vv ] && { echo 'rustc stub'; exit 0; }
while [ $# -gt 1 ]; do
    [ "$1" = -o ] && { cp "$STUB_PROCESS" "$2" && chmod +x "$2"; exit; }
    shift
done
exit 2
"""
# Zero arguments: the contract-test binary. Six arguments: a complete-output process.
PROCESS = """#!/bin/sh
echo "process $*" >> "$STUB_LOG"
[ $# -eq 0 ] && { echo 'stub contracts pass'; exit 0; }
cat "$STUB_ANSWERS/$1-$2-$3-$4-$5-$#.json"
"""


def events(n, flows, pattern):
    """The BENCHMARK.md workload: (flow, tick, cost) per attempt."""
    for i in range(n):
        rnd = i // flows
        tick = {"burst": rnd // 1024, "steady": rnd, "idle": 100 * rnd, "weighted": rnd // 2}[pattern]
        cost = (0, 1, 6, 11, 4, 10, 3)[i % 7] if pattern == "weighted" else 6
        yield i % flows, tick, cost


def model(n, flows, pattern):
    """Digest and (decision, remaining credit) per attempt: capacity 10, 3 credits per tick."""
    credit, last = [10] * flows, [0] * flows
    digest, output = 0, []
    for flow, tick, cost in events(n, flows, pattern):
        credit[flow] = min(10, credit[flow] + 3 * (tick - last[flow]))
        last[flow] = tick
        ok = cost <= credit[flow]
        if ok:
            credit[flow] -= cost
        output.append([ok, credit[flow]])
        digest = (digest * 1_000_003 + 2 * credit[flow] + ok) % 2**64
    return digest, output


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


class RunnerTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.shared = tempfile.TemporaryDirectory()
        cls.answers = pathlib.Path(cls.shared.name) / "answers"
        cls.wrong = pathlib.Path(cls.shared.name) / "wrong"
        cls.answers.mkdir()
        cls.wrong.mkdir()
        for n, flows, pattern in CELLS:
            digest, output = model(n, flows, pattern)
            reps = 256 if n == 32 else 16
            for mode in MODES:
                timed = {"ns": 1000 * reps, "reps": reps, "digest": digest, "state_bytes": 32 * flows, "output": []}
                full = {"ns": 1000, "reps": 1, "digest": digest, "state_bytes": 32 * flows, "output": output}
                for name, answer in [(f"{mode}-{n}-{flows}-{pattern}-{reps}-5.json", timed),
                                     (f"{mode}-{n}-{flows}-{pattern}-1-6.json", full)]:
                    (cls.answers / name).write_text(json.dumps(answer))
                    # A candidate that rejects every attempt and keeps 10 credits:
                    # its digest and its complete output differ from the model in every cell.
                    rejecting = dict(answer, digest=answer["digest"] ^ 1, output=[[False, 10]] * len(answer["output"]))
                    (cls.wrong / name).write_text(json.dumps(rejecting))

    @classmethod
    def tearDownClass(cls):
        cls.shared.cleanup()

    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.dir = pathlib.Path(scratch.name)
        self.repo = self.dir / "repo"
        self.topic = self.repo / "topics" / "050"
        self.raw = self.topic / "raw"
        self.log = self.dir / "log"
        self.log.write_text("")
        tools = self.dir / "tools"
        tools.mkdir()
        for name, text in [("rustc", RUSTC), ("process", PROCESS)]:
            (tools / name).write_text(text)
            (tools / name).chmod(0o755)
        for name in INPUTS:
            path = self.topic / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f"stub input {name}\n")
        shutil.copy(RUN, self.topic / "benches" / "run.py")
        self.hashes = {name: sha(self.topic / name) for name in INPUTS}
        self.env = {k: v for k, v in os.environ.items() if k != "PYTHONOPTIMIZE"}
        self.env.update(PATH=f"{tools}{os.pathsep}{os.environ['PATH']}", STUB_LOG=str(self.log),
                        STUB_PROCESS=str(tools / "process"), STUB_ANSWERS=str(self.answers),
                        GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1")

    def start(self, answers=None, flags=(), source=SOURCE, cwd=None, **extra):
        """Run the scratch copy of run.py against the stub tools."""
        env = dict(self.env, **extra)
        if answers:
            env["STUB_ANSWERS"] = str(answers)
        command = [sys.executable, *flags, str(self.topic / "benches" / "run.py"), "--source", source]
        return subprocess.run(command, cwd=cwd or self.topic, capture_output=True, text=True, env=env, timeout=600, check=False)

    def git(self, *args):
        command = ["git", "-c", "user.name=test", "-c", "user.email=test@example.invalid", "-c", "commit.gpgsign=false", *args]
        return subprocess.run(command, cwd=self.repo, capture_output=True, text=True, env=self.env, check=True).stdout.strip()

    def commit(self):
        self.git("add", "--all")
        self.git("commit", "--quiet", "--message", "inputs")
        return self.git("rev-parse", "HEAD")

    def pin(self, manifest):
        (self.topic / "INPUTS.json").write_text(json.dumps(manifest))

    def launches(self, tool="process"):
        return [line for line in self.log.read_text().splitlines() if line.startswith(tool)]

    def assert_manifest_matches_members(self):
        manifest = json.loads((self.raw / "MANIFEST.json").read_text())
        for name, digest in manifest.items():
            self.assertEqual(sha(self.topic / name), digest, f"{name} differs from its MANIFEST.json entry")
        return manifest

    def assert_refused(self, process, text):
        """The run stopped at startup: the message names the cause, raw/ is absent and no tool ran."""
        self.assertNotEqual(process.returncode, 0)
        self.assertIn(text, process.stderr)
        self.assertFalse(self.raw.exists(), "a refused start left raw/ behind")
        self.assertEqual(self.launches(""), [])

    def started(self, **arguments):
        """environment.json of a run that passed startup. The rejecting candidate halts it at its first process."""
        if self.raw.exists():
            shutil.rmtree(self.raw)
        self.log.write_text("")
        process = self.start(answers=self.wrong, **arguments)
        self.assertEqual(self.launches()[-1:], [FIRST_PROCESS], process.stderr)
        return json.loads((self.raw / "environment.json").read_text())

    def test_complete_run_is_verified_and_its_manifest_matches(self):
        process = self.start()
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(process.stdout)["processes"], 396)
        summary = json.loads((self.raw / "summary.json").read_text())
        self.assertEqual((summary["verified"], summary["processes"], len(summary["cells"])), (True, 396, 16))
        self.assertEqual(summary["binary_sha256"], sha(self.raw / "replay"))
        records = json.loads((self.raw / "records.json").read_text())
        self.assertEqual([sum(r["phase"] == p for r in records) for p in ("correctness", "warmup", "measured")],
                         [48, 48, 288])
        self.assertEqual(sum(r["phase"].startswith("aa-") for r in records), 12)
        manifest = self.assert_manifest_matches_members()
        self.assertEqual(sorted(manifest), sorted(f"raw/{p.name}" for p in self.raw.iterdir() if p.name != "MANIFEST.json"))
        environment = json.loads((self.raw / "environment.json").read_text())
        self.assertEqual((environment["inputs"], environment["source"], environment["source_check"]),
                         (self.hashes, SOURCE, "label"))

    def test_wrong_candidate_output_halts_without_a_summary(self):
        process = self.start(answers=self.wrong)
        self.assertNotEqual(process.returncode, 0)
        self.assertFalse((self.raw / "summary.json").exists())
        self.assertEqual(self.launches(), ["process ", FIRST_PROCESS])

    def test_refuses_to_start_with_assertions_disabled(self):
        self.pin(self.hashes)
        for flags, extra in [(("-O",), {}), (("-OO",), {}), ((), {"PYTHONOPTIMIZE": "1"})]:
            with self.subTest(flags=flags, extra=extra):
                self.assert_refused(self.start(answers=self.wrong, flags=flags, **extra), "assertions")

    def test_refuses_an_existing_output_directory(self):
        self.pin(self.hashes)
        self.raw.mkdir()
        (self.raw / "earlier.txt").write_text("kept")
        (self.raw / "MANIFEST.json").write_text(json.dumps({"raw/earlier.txt": sha(self.raw / "earlier.txt")}))
        before = {p.name: p.read_bytes() for p in self.raw.iterdir()}
        process = self.start()
        self.assert_manifest_matches_members()
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual({p.name: p.read_bytes() for p in self.raw.iterdir()}, before)
        self.assertEqual(self.launches(""), [])

    def test_matching_inputs_json_is_accepted(self):
        self.pin(self.hashes)
        self.assertEqual(self.started()["inputs"], self.hashes)

    def test_differing_inputs_json_is_refused(self):
        name = INPUTS[0]
        cases = {
            "wrong hash": dict(self.hashes, **{name: "0" * 64}),
            "missing path": {k: v for k, v in self.hashes.items() if k != name},
            "extra path": dict(self.hashes, **{"Cargo.toml": "0" * 64}),
            "empty": {},
        }
        for case, manifest in cases.items():
            with self.subTest(case):
                self.pin(manifest)
                self.assert_refused(self.start(), "INPUTS.json")
        (self.topic / "INPUTS.json").write_text("{")
        self.assert_refused(self.start(), "INPUTS.json is not JSON")

    def test_source_must_be_a_full_commit_id(self):
        self.pin(self.hashes)
        self.git("init", "--quiet")
        head = self.commit()
        for source in ["HEAD", head[:12], head.upper(), head + "0"]:
            with self.subTest(source=source):
                self.assert_refused(self.start(source=source), "--source")

    def test_tracked_inputs_must_equal_the_source_commit(self):
        self.pin(self.hashes)
        self.git("init", "--quiet")
        first = self.commit()
        self.assert_refused(self.start(), f"differs from commit {SOURCE}")
        environment = self.started(source=first)
        self.assertEqual((environment["source"], environment["source_check"]), (first, "git"))
        shutil.rmtree(self.raw)
        self.log.write_text("")
        (self.topic / "src" / "lib.rs").write_text("edited after the first commit\n")
        self.pin(dict(self.hashes, **{"src/lib.rs": sha(self.topic / "src" / "lib.rs")}))
        self.assert_refused(self.start(source=first), f"src/lib.rs differs from commit {first}")
        second = self.commit()
        self.assert_refused(self.start(source=first), f"src/lib.rs differs from commit {first}")
        self.assertEqual(self.started(source=second)["source_check"], "git")

    def test_runs_in_the_topic_directory_that_holds_it(self):
        self.assertEqual(self.started(cwd=self.dir)["inputs"], self.hashes)
        self.assertFalse((self.dir / "raw").exists())


if __name__ == "__main__":
    unittest.main()
