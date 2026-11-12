#!/usr/bin/env python3
"""Behavioral checks for run.py against stub rustc and taskset processes.

Each test copies run.py, main.rs and src/lib.rs into a temporary topic tree and puts stub
`rustc` and `taskset` executables first on PATH. The stub `rustc` requires each source
path to exist, then writes an executable for the `-o` target. The stub `taskset` plays the
measured process and appends its arguments to a log. Run on Linux from any directory:
python3 topics/046-streaming-sketches/test_run.py
"""
import collections
import hashlib
import importlib.util
import json
import math
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

TOPIC = pathlib.Path(__file__).resolve().parent
INPUTS = ["run.py", "main.rs", "src/lib.rs"]
CANDIDATES = ["exact-frequency", "count-min", "exact-distinct", "hll", "exact-quantile", "rank-summary"]
WORKLOADS = [(128, "uniform"), (4096, "uniform"), (16384, "uniform"), (16384, "skew"), (16384, "sorted"), (16384, "reverse")]
FIRST = "warmup-0-exact-frequency"
SECOND = "warmup-0-count-min"

LOG = 'echo "taskset $*" >> "$STUB_LOG"\n'
RUSTC = """#!/bin/sh
echo "rustc $*" >> "$STUB_LOG"
case " $* " in *" -vV "*|*" --print "*) echo 'rustc stub'; exit 0;; esac
for arg; do case "$arg" in *.rs) [ -f "$arg" ] || { echo "missing $arg" >&2; exit 1; };; esac; done
while [ $# -gt 1 ]; do [ "$1" = -o ] && out=$2; shift; done
printf '#!/bin/sh\\necho stub\\n' > "$out"
chmod +x "$out"
"""
FAILING = "#!/bin/sh\n" + LOG + "echo 'stub failure'\nexit 3\n"
HANGING = "#!/bin/sh\n" + LOG + "echo 'partial'\nexec sleep 30\n"
ANSWERING = "#!/bin/sh\n" + LOG + 'cat "$STUB_ANSWER"\n'
FIRST_ONLY = "#!/bin/sh\n" + LOG + '[ "$4" = exact-frequency ] || { echo "stub failure"; exit 3; }\ncat "$STUB_ANSWER"\n'
MODEL = f"""#!{sys.executable}
import json, os, sys
sys.path[:0] = [os.environ["STUB_TOPIC"], os.environ["STUB_TESTS"]]
import run, test_run
with open(os.environ["STUB_LOG"], "a") as log:
    log.write("taskset " + " ".join(sys.argv[1:]) + "\\n")
candidate, n, shape, seed, repeats = sys.argv[4:]
print(json.dumps(test_run.record(run, candidate, int(n), shape, int(seed), int(repeats))))
"""


def record(run, candidate, n, shape, seed, repeats):
    """One process answer that satisfies the oracle, built from the workload and sketch layouts."""
    data = run.inputs(n, shape)
    counts = collections.Counter(data)
    ordered = sorted(data)
    r = {"candidate": candidate, "n": n, "shape": shape, "seed": seed, "repeats": repeats,
         "ns": 1000.0, "estimate": 0.0, "values": [], "state": []}
    if candidate == "exact-frequency":
        r.update(values=[counts[x] for x in range(4096)], bytes=16 * len(counts))
    elif candidate == "count-min":
        table = [0] * 1280
        for x, count in counts.items():
            for row in range(5):
                table[row * 256 + run.mix(x, seed + row) % 256] += count
        values = [min(table[row * 256 + run.mix(x, seed + row) % 256] for row in range(5)) for x in range(4096)]
        r.update(state=table, values=values, bytes=10240)
    elif candidate == "exact-distinct":
        r.update(estimate=float(len(counts)), bytes=8 * len(counts))
    elif candidate == "hll":
        registers = [0] * 1024
        for x in counts:
            h = run.mix(x, seed)
            registers[h >> 54] = max(registers[h >> 54], 55 - (h % (1 << 54)).bit_length())
        zeros = registers.count(0)
        raw = 0.7213 / (1 + 1.079 / 1024) * 1024 ** 2 / sum(2.0 ** -v for v in registers)
        r.update(state=registers, bytes=1024, estimate=1024 * math.log(1024 / zeros) if raw <= 2560 and zeros else raw)
    else:
        r["values"] = [ordered[(n - 1) * q // 100] for q in range(101)]
        if candidate == "rank-summary":
            r["state"] = [word for x in ordered for word in (x, 1, 0)]
        r["bytes"] = 8 * len(r["state"]) if candidate == "rank-summary" else 8 * n
    return r


def schedule():
    """The invocation order BENCHMARK.md declares, as (candidate, n, shape, seed, repeats)."""
    calls = []
    for n, shape in WORKLOADS:
        calls += [(c, n, shape, 7, 1) for c in CANDIDATES]
        for block in range(6):
            pairs = [CANDIDATES[0:2], CANDIDATES[2:4], CANDIDATES[4:6]]
            pairs = pairs[block % 3:] + pairs[:block % 3]
            for pair in pairs:
                calls += [(c, n, shape, 7, 32 if n == 128 else 4) for c in (pair if block % 2 == 0 else pair[::-1])]
        calls += [("hll", n, shape, seed, 1) for seed in range(16)]
    return calls + [("exact-frequency", 16384, "uniform", 7, 4)] * 12


class RunnerTest(unittest.TestCase):
    def tree(self, taskset):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix="topic046-test-"))
        self.addCleanup(shutil.rmtree, tmp)
        topic = tmp / "topic"
        (topic / "src").mkdir(parents=True)
        for name in INPUTS:
            shutil.copyfile(TOPIC / name, topic / name)
        (tmp / "stubs").mkdir()
        for name, body in [("rustc", RUSTC), ("taskset", taskset)]:
            (tmp / "stubs" / name).write_text(body)
            (tmp / "stubs" / name).chmod(0o755)
        return topic

    def load(self, topic):
        spec = importlib.util.spec_from_file_location("run", topic / "run.py")
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def answer(self, topic, **changed):
        r = dict(record(self.load(topic), "exact-frequency", 128, "uniform", 7, 1), **changed)
        (topic.parent / "answer.json").write_text(json.dumps(r))

    def launch(self, topic, *flags, cwd=None, **extra):
        tmp = topic.parent
        env = {k: v for k, v in os.environ.items() if k != "PYTHONOPTIMIZE"}
        env.update(PATH=str(tmp / "stubs") + os.pathsep + env["PATH"], STUB_LOG=str(tmp / "stub.log"),
                   STUB_ANSWER=str(tmp / "answer.json"), STUB_TOPIC=str(topic), STUB_TESTS=str(TOPIC),
                   TOPIC_046_PROCESS_TIMEOUT_S="20")
        env.update(extra)
        return subprocess.run([sys.executable, *flags, str(topic / "run.py")], cwd=cwd or topic, env=env,
                              capture_output=True, text=True, timeout=600)

    def attempt(self, topic, label=FIRST):
        return json.loads((topic / "raw" / (label + ".json")).read_text())

    def files(self, root):
        return {str(p.relative_to(root)): p.read_bytes() for p in sorted(root.rglob("*")) if p.is_file()}

    def assert_halted(self, topic, done):
        self.assertNotEqual(done.returncode, 0, done.stderr)
        self.assertFalse((topic / "results.json").exists())
        self.assertFalse((topic / "manifest.json").exists())

    def test_nonzero_exit_is_retained(self):
        topic = self.tree(FAILING)
        self.assert_halted(topic, self.launch(topic))
        row = self.attempt(topic)
        self.assertEqual((row["status"], row["exit"], row["stdout"]), ("failed", 3, "stub failure\n"))
        self.assertEqual(row["command"][-5:], ["exact-frequency", "128", "uniform", "7", "1"])

    def test_timeout_is_retained_with_partial_output(self):
        topic = self.tree(HANGING)
        self.assert_halted(topic, self.launch(topic, TOPIC_046_PROCESS_TIMEOUT_S="1"))
        row = self.attempt(topic)
        self.assertEqual((row["status"], row["timeout_s"], row["stdout"]), ("timeout", 1.0, "partial\n"))

    def test_unusable_output_is_retained_and_never_marked_pass(self):
        cases = {
            "text": None,
            "wrong count": {"values": [9] * 4096},
            "other seed": {"seed": 8},
            "other repeats": {"repeats": 4},
            "nan time": {"ns": float("nan")},
            "zero time": {"ns": 0},
            "array": [],
        }
        for name, changed in cases.items():
            with self.subTest(name):
                topic = self.tree(ANSWERING)
                if isinstance(changed, dict):
                    self.answer(topic, **changed)
                else:
                    (topic.parent / "answer.json").write_text("not json\n" if changed is None else "[]\n")
                self.assert_halted(topic, self.launch(topic))
                row = self.attempt(topic)
                self.assertEqual(row["status"], "rejected")
                self.assertEqual(row["stdout"], (topic.parent / "answer.json").read_text())
                self.assertNotIn("oracle", row)
                self.assertEqual(len(list((topic / "raw").glob("*-*.json"))), 1)

    def test_accepted_attempt_is_marked_and_the_next_failure_halts(self):
        topic = self.tree(FIRST_ONLY)
        self.answer(topic)
        self.assert_halted(topic, self.launch(topic))
        row = self.attempt(topic)
        self.assertEqual((row["status"], row["oracle"], row["label"]), ("ok", "PASS", FIRST))
        self.assertEqual(self.attempt(topic, SECOND)["status"], "failed")

    def test_runs_from_another_directory_and_builds_inside_the_topic(self):
        topic = self.tree(FIRST_ONLY)
        self.answer(topic)
        self.launch(topic, cwd=topic.parent)
        self.assertEqual(self.attempt(topic)["status"], "ok")
        for name in ["sketch-lab", "contracts", "generated.s"]:
            self.assertTrue((topic / name).is_file(), name)
        self.assertEqual(sorted(os.listdir(topic.parent)), ["answer.json", "stub.log", "stubs", "topic"])

    def test_refuses_to_start_over_earlier_outputs(self):
        for name in ["raw", "raw/obsolete.json", "results.json", "manifest.json"]:
            with self.subTest(name):
                topic = self.tree(FAILING)
                if name == "raw":
                    (topic / "raw").mkdir()
                else:
                    (topic / name).parent.mkdir(exist_ok=True)
                    (topic / name).write_text("earlier campaign\n")
                before = self.files(topic), sorted(os.listdir(topic))
                done = self.launch(topic)
                self.assertNotEqual(done.returncode, 0)
                self.assertIn(str(topic / name.split("/")[0]), done.stderr)
                self.assertEqual((self.files(topic), sorted(os.listdir(topic))), before)
                self.assertFalse((topic.parent / "stub.log").exists())

    def test_second_start_keeps_the_failure_record_of_the_first(self):
        topic = self.tree(FAILING)
        self.launch(topic)
        before = self.files(topic), (topic.parent / "stub.log").read_text()
        done = self.launch(topic)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn(str(topic / "raw"), done.stderr)
        self.assertEqual((self.files(topic), (topic.parent / "stub.log").read_text()), before)

    def test_refuses_to_start_with_assertions_disabled(self):
        for flags, extra in [(["-O"], {}), (["-OO"], {}), ([], {"PYTHONOPTIMIZE": "1"})]:
            for changed in [{}, {"values": [9] * 4096}]:
                with self.subTest(flags=flags, extra=extra, changed=changed):
                    topic = self.tree(ANSWERING)
                    self.answer(topic, **changed)
                    done = self.launch(topic, *flags, **extra)
                    self.assertNotEqual(done.returncode, 0)
                    self.assertIn("assertions", done.stderr)
                    self.assertFalse((topic / "raw").exists())
                    self.assertFalse((topic.parent / "stub.log").exists())

    def test_import_creates_no_output(self):
        topic = self.tree(FAILING)
        self.load(topic)
        self.assertEqual(sorted(set(os.listdir(topic)) - {"__pycache__"}), ["main.rs", "run.py", "src"])

    def test_oracle_accepts_model_answers_and_requires_logical_bytes(self):
        run = self.load(self.tree(FAILING))
        for candidate in CANDIDATES:
            for n in [0, 128] if candidate in ("count-min", "hll") else [128]:
                with self.subTest(candidate, n=n):
                    r = record(run, candidate, n, "uniform", 7, 1)
                    self.assertTrue(run.check(r))
                    with self.assertRaises(AssertionError):
                        run.check(dict(r, bytes=r["bytes"] + 8))

    def test_oracle_requires_quantiles_and_summary_entries_to_be_observed_values(self):
        run = self.load(self.tree(FAILING))
        ordered = sorted(run.inputs(128, "uniform"))
        q, i = next((q, i) for q in range(1, 100) for i in [(128 - 1) * q // 100] if ordered[i + 1] - ordered[i] > 1)
        absent = ordered[i] + 1
        r = record(run, "rank-summary", 128, "uniform", 7, 1)
        values = r["values"][:q] + [absent] + r["values"][q + 1:]
        state = r["state"][:3 * i] + [absent, 1, 1] + r["state"][3 * i + 3:]
        self.assertTrue(run.check(r))
        for changed in [{"values": values}, {"state": state}]:
            with self.subTest(list(changed)[0]), self.assertRaises(AssertionError):
                run.check(dict(r, **changed))

    def test_complete_campaign_follows_the_declared_schedule_and_layout(self):
        topic = self.tree(MODEL)
        done = self.launch(topic)
        self.assertEqual(done.returncode, 0, done.stderr)
        calls = [tuple(line.split()[4:]) for line in (topic.parent / "stub.log").read_text().splitlines()
                 if line.startswith("taskset ")]
        self.assertEqual(calls, [tuple(map(str, call)) for call in schedule()])
        self.assertEqual(len(calls), 360)
        rows = [json.loads(p.read_text()) for p in (topic / "raw").glob("*-*.json")]
        self.assertEqual(len(rows), 360)
        self.assertEqual({(r["status"], r["oracle"]) for r in rows}, {("ok", "PASS")})
        result = json.loads((topic / "results.json").read_text())
        self.assertEqual((result["processes"], result["main_processes"], len(result["cells"])), (360, 216, 18))
        manifest = json.loads((topic / "manifest.json").read_text())
        self.assertEqual(len(manifest), 364)
        for name, digest in manifest.items():
            self.assertEqual(name.split("/")[0], "raw")
            self.assertEqual(hashlib.sha256((topic / name).read_bytes()).hexdigest(), digest)


if __name__ == "__main__":
    unittest.main()
