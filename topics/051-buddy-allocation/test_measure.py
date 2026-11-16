#!/usr/bin/env python3
"""Behavioral checks for measure.py against stub replay processes.

Each test passes a stub executable as the binary argument. The stub answers come
from `model`, an occupancy-map simulation of lowest-address aligned placement
over the traces of src/lib.rs `workload`. It shares no code with measure.py's
free-interval oracle. Run:
python3 topics/051-buddy-allocation/test_measure.py
"""
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

RUN = pathlib.Path(__file__).resolve().parent / "measure.py"
KINDS = ["scan", "buddy", "tree"]
CELLS = [(n, shape) for n in (64, 4096) for shape in ("tiny", "mixed", "holes", "churn")]
ORDERS = ["scan buddy tree", "scan tree buddy", "buddy scan tree",
          "buddy tree scan", "tree scan buddy", "tree buddy scan"]
WORK = {"scan": 11, "buddy": 22, "tree": 33}

LOG = 'echo "$*" >> "$STUB_LOG"\n'
MODEL = "#!/bin/sh\n" + LOG + 'cat "$STUB_ANSWERS/$1-$2-$3${4:+-check}"\n'
WRONG = "#!/bin/sh\n" + LOG + 'sed "s/^A 0 1 0$/A 0 1 5/" "$STUB_ANSWERS/$1-$2-$3${4:+-check}"\n'
FIXED = "#!/bin/sh\n" + LOG + 'cat "$STUB_ANSWER"\n'
TIMED_FIXED = ("#!/bin/sh\n" + LOG + 'if [ "$4" = check ]; then cat "$STUB_ANSWERS/$1-$2-$3-check"; '
               'else cat "$STUB_ANSWER"; fi\n')
REBUILT = ("#!/bin/sh\n" + LOG + 'if [ "$(wc -l < "$STUB_LOG")" -eq 10 ]; then echo "# rebuilt" >> "$0"; fi\n'
           'cat "$STUB_ANSWERS/$1-$2-$3${4:+-check}"\n')
FAILING = "#!/bin/sh\n" + LOG + "echo 'stub output'\necho 'stub failure' >&2\nexit 3\n"
HANGING = "#!/bin/sh\n" + LOG + "echo 'partial'\nexec sleep 30\n"


def trace(n, shape):
    """The operations of src/lib.rs `workload`: ("A", id, request) or ("F", id, 0)."""
    if shape == "holes":
        return ([("A", i, 1) for i in range(n)] + [("F", i, 0) for i in range(0, n, 2)]
                + [("A", i, 2) for i in range(0, n, 2)] + [("F", i, 0) for i in range(n)])
    ops = []
    for rnd in range(8):
        count = 8 if shape == "churn" else n // 2
        for i in range(count):
            request = {"tiny": 1, "mixed": [1, 3, 5, 9, 17, 33][(i + rnd) % 6],
                       "churn": [n // 2 - 1, 3, 1, n // 4 + 1][i % 4]}[shape]
            ops.append(("A", i, request))
        ops += [("F", i, 0) for i in reversed(range(count))]
    return ops


def model(n, shape):
    """(answer lines, final occupancy, results) of one trace, by first aligned free run."""
    used, live, lines, results = bytearray(n), {}, [], []
    for kind, i, request in trace(n, shape):
        if kind == "A":
            size = 1 << (request - 1).bit_length()
            result = -1
            if i not in live and size <= n:
                at = 0
                while result < 0:
                    at = used.find(bytes(size), at)
                    if at < 0:
                        break
                    if at % size == 0:
                        result = at
                    else:
                        at += size - at % size
            if result >= 0:
                live[i] = (result, size)
                used[result:result + size] = b"\x01" * size
        else:
            result = int(i in live)
            if result:
                at, size = live.pop(i)
                used[at:at + size] = bytes(size)
        lines.append(f"{kind} {i} {request} {result}")
        results.append(result)
    return lines, list(used), results


def fold(results):
    """The digest of examples/replay.rs: wrapping h * 31 + |result|."""
    digest = 0
    for value in results:
        digest = (digest * 31 + abs(value)) % 2 ** 64
    return digest


def declared_schedule():
    """The BENCHMARK.md process order as (kind, n, shape, phase, block, check)."""
    schedule = []
    for n, shape in CELLS:
        schedule += [(k, n, shape, "check", -1, True) for k in KINDS]
        schedule += [(k, n, shape, "warmup", -1, False) for k in KINDS]
        for block, order in enumerate(ORDERS):
            schedule += [(k, n, shape, "measure", block, False) for k in order.split()]
    for block in range(6):
        for label in ("AB" if block % 2 == 0 else "BA"):
            schedule.append(("scan", 4096, "mixed", "AA" + label, block, False))
    return schedule


class MeasureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.shared = tempfile.TemporaryDirectory()
        cls.answers = pathlib.Path(cls.shared.name)
        cls.summary, cls.check = {}, {}
        for n, shape in CELLS:
            lines, state, results = model(n, shape)
            for kind in KINDS:
                summary = {"kind": kind, "n": n, "shape": shape, "ns": 1000, "setup_ns": 10,
                           "ops": len(lines), "work": WORK[kind], "digest": fold(results)}
                text = "\n".join(lines + ["STATE " + json.dumps(state), json.dumps(summary)]) + "\n"
                cls.summary[kind, n, shape], cls.check[kind, n, shape] = summary, text
                (cls.answers / f"{kind}-{n}-{shape}").write_text(json.dumps(summary) + "\n")
                (cls.answers / f"{kind}-{n}-{shape}-check").write_text(text)

    @classmethod
    def tearDownClass(cls):
        cls.shared.cleanup()

    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.dir = pathlib.Path(scratch.name)
        self.results = self.dir / "out" / "results.jsonl"
        self.log = self.dir / "log"
        self.log.write_text("")

    def start(self, stub, answer=None, flags=(), **extra):
        """Run measure.py on a stub binary. `answer` is the text FIXED and TIMED_FIXED print."""
        self.binary = self.dir / "binary"
        self.binary.write_text(stub)
        self.binary.chmod(0o755)
        if answer is not None:
            (self.dir / "answer").write_text(answer)
        env = {k: v for k, v in os.environ.items() if k != "PYTHONOPTIMIZE"}
        env.update(STUB_LOG=str(self.log), STUB_ANSWER=str(self.dir / "answer"), STUB_ANSWERS=str(self.answers))
        env.update(extra)
        command = [sys.executable, *flags, str(RUN), str(self.binary), str(self.results)]
        return subprocess.run(command, capture_output=True, text=True, env=env, timeout=300, check=False)

    def records(self):
        self.assertTrue(self.results.exists(), "no results file was retained")
        return [json.loads(line) for line in self.results.read_text().splitlines()]

    def only_record(self):
        records = self.records()
        self.assertEqual(len(records), 1, "the halting attempt is not the one retained record")
        return records[0]

    @staticmethod
    def results_of(lines):
        return [int(line.split()[3]) for line in lines]

    def launches(self):
        return self.log.read_text().splitlines()

    def fresh(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.results = pathlib.Path(scratch.name) / "results.jsonl"
        self.log.write_text("")

    def test_refuses_to_start_with_assertions_disabled(self):
        for flags, extra in [(("-O",), {}), (("-OO",), {}), ((), {"PYTHONOPTIMIZE": "1"})]:
            with self.subTest(flags=flags, extra=extra):
                process = self.start(WRONG, flags=flags, **extra)
                self.assertNotEqual(process.returncode, 0)
                self.assertIn("assertions", process.stderr)
                self.assertNotIn("PASS", process.stdout)
                self.assertFalse(self.results.exists())
                self.assertEqual(self.launches(), [])

    def test_nonzero_exit_is_retained_as_failed_and_halts(self):
        process = self.start(FAILING)
        self.assertNotEqual(process.returncode, 0)
        record = self.only_record()
        self.assertEqual([record[k] for k in ("kind", "n", "shape", "phase", "block")],
                         ["scan", 64, "tiny", "check", -1])
        self.assertEqual((record["status"], record["exit"]), ("failed", 3))
        self.assertEqual(record["command"], [str(self.binary), "scan", "64", "tiny", "check"])
        self.assertEqual((record["stdout"], record["stderr"]), ("stub output\n", "stub failure\n"))
        self.assertEqual(self.launches(), ["scan 64 tiny check"])

    def test_timeout_is_retained_with_partial_output(self):
        process = self.start(HANGING, TOPIC_051_PROCESS_TIMEOUT_S="1")
        self.assertNotEqual(process.returncode, 0)
        record = self.only_record()
        self.assertEqual((record["status"], record["timeout_s"], record["stdout"]), ("timeout", 1.0, "partial\n"))
        self.assertEqual(record["command"], [str(self.binary), "scan", "64", "tiny", "check"])
        self.assertNotIn("ns", record)
        self.assertEqual(self.launches(), ["scan 64 tiny check"])

    def test_unusable_check_output_is_retained_as_rejected(self):
        good = self.check["scan", 64, "tiny"]
        summary = self.summary["scan", 64, "tiny"]

        def changed(**fields):
            return good.replace(json.dumps(summary), json.dumps({**summary, **fields}))

        lines = good.splitlines()
        mixed = self.check["scan", 64, "mixed"]
        self.assertEqual(lines[-3], "F 0 0 1")
        short = [*lines[:-3], "STATE " + json.dumps([1] + [0] * 63),
                 json.dumps({**summary, "ops": summary["ops"] - 1, "digest": fold(self.results_of(lines[:-3]))})]
        cases = {
            "empty": "",
            "text": "not json\n",
            "summary only": json.dumps({**summary, "ops": 0, "digest": 0}) + "\n",
            "last operation missing": "\n".join(short) + "\n",
            "other trace of equal length": mixed.replace('"shape": "mixed"', '"shape": "tiny"'),
            "no state": "\n".join(lines[:-2] + lines[-1:]) + "\n",
            "two states": "\n".join(lines[:-1] + lines[-2:]) + "\n",
            "ops": changed(ops=summary["ops"] - 1),
            "answer": good.replace("A 0 1 0", "A 0 1 5", 1),
            "failure for address 1": good.replace("A 1 1 1", "A 1 1 -1", 1),
            "state": good.replace("STATE [0", "STATE [1", 1),
            "digest": changed(digest=summary["digest"] ^ 1),
            "kind echo": changed(kind="tree"),
            "n echo": changed(n=4096),
            "shape echo": changed(shape="mixed"),
        }
        for name, answer in cases.items():
            with self.subTest(name):
                self.assertNotEqual(answer, good)
                self.fresh()
                process = self.start(FIXED, answer=answer)
                self.assertNotEqual(process.returncode, 0)
                self.assertNotIn("PASS", process.stdout)
                record = self.only_record()
                self.assertEqual((record["status"], record["exit"], record["phase"]), ("rejected", 0, "check"))
                self.assertTrue(record["error"])
                self.assertEqual(record["stdout"], answer)
                self.assertEqual(self.launches(), ["scan 64 tiny check"])

    def test_timed_process_must_agree_with_its_check_process(self):
        summary = self.summary["scan", 64, "tiny"]
        cases = {
            "digest": {"digest": summary["digest"] ^ 1},
            "ops": {"ops": summary["ops"] - 1},
            "work": {"work": summary["work"] + 1},
            "kind echo": {"kind": "buddy"},
            "n echo": {"n": 4096},
            "shape echo": {"shape": "holes"},
        }
        for name, fields in cases.items():
            with self.subTest(name):
                self.fresh()
                process = self.start(TIMED_FIXED, answer=json.dumps({**summary, **fields}) + "\n")
                self.assertNotEqual(process.returncode, 0)
                records = self.records()
                self.assertEqual([(r["phase"], r.get("status")) for r in records],
                                 [("check", "ok")] * 3 + [("warmup", "rejected")])
                self.assertEqual(records[3]["command"], [str(self.binary), "scan", "64", "tiny"])
                self.assertTrue(records[3]["error"])
                self.assertEqual(len(self.launches()), 4)

    def test_complete_campaign_keeps_the_declared_schedule(self):
        process = self.start(MODEL)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertIn("PASS 24 complete independent-oracle traces; 204 process records", process.stdout)
        self.assertIn("binary_sha256 " + hashlib.sha256(MODEL.encode()).hexdigest(), process.stdout)
        schedule = declared_schedule()
        self.assertEqual([sum(s[3].startswith(p) for s in schedule) for p in ("check", "warmup", "measure", "AA")],
                         [24, 24, 144, 12])
        records = self.records()
        self.assertEqual(len(records), 204)
        for index, (kind, n, shape, phase, block, _) in enumerate(schedule):
            self.assertEqual(records[index],
                             {**self.summary[kind, n, shape], "phase": phase, "block": block, "status": "ok"})
        self.assertEqual(self.launches(), [f"{kind} {n} {shape}" + (" check" if check else "")
                                           for kind, n, shape, _, _, check in schedule])
        checks = self.results.parent / "checks"
        self.assertEqual(sorted(p.name for p in checks.iterdir()),
                         sorted(f"{kind}-{n}-{shape}.txt" for kind in KINDS for n, shape in CELLS))
        self.assertEqual((checks / "tree-4096-holes.txt").read_text(), self.check["tree", 4096, "holes"])

    def test_changed_binary_stops_the_run_before_the_next_launch(self):
        process = self.start(REBUILT)
        self.assertNotEqual(process.returncode, 0)
        self.assertNotIn("PASS", process.stdout)
        self.assertNotIn("binary_sha256", process.stdout)
        self.assertIn(hashlib.sha256(REBUILT.encode()).hexdigest(), process.stderr)
        self.assertEqual([r["status"] for r in self.records()], ["ok"] * 10)
        self.assertEqual(len(self.launches()), 10)
        self.assertEqual(self.binary.read_text(), REBUILT + "# rebuilt\n")

    def test_refuses_an_existing_results_file(self):
        self.results.parent.mkdir()
        self.results.write_text("earlier\n")
        process = self.start(MODEL)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual(self.results.read_text(), "earlier\n")
        self.assertEqual(self.launches(), [])


if __name__ == "__main__":
    unittest.main()
