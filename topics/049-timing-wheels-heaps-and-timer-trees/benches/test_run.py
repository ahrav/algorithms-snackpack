#!/usr/bin/env python3
"""Behavioral checks for run.py against stub measured processes.

Each test passes a stub executable as --binary; run.py launches it through
`taskset -c 0` exactly as it launches the native replay binary. The stub answers
come from `model`, a step simulation of the four timer mechanisms written from
BENCHMARK.md and README.md. It shares no code with run.py's oracle. Run on Linux:
python3 topics/049-timing-wheels-heaps-and-timer-trees/benches/test_run.py
"""
import importlib.util
import itertools
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

RUN = pathlib.Path(__file__).resolve().parent / "run.py"
CANDIDATES = ["scan", "heap", "tree", "wheel"]
CELLS = list(itertools.product([16, 4096], ["near", "far", "burst"], [0, 90]))
FIRST = ("scan", 16, "near", 0)

LOG = 'echo "$*" >> "$STUB_LOG"\n'
FAILING = "#!/bin/sh\n" + LOG + "echo 'stub output'\necho 'stub failure' >&2\nexit 3\n"
HANGING = "#!/bin/sh\n" + LOG + "echo 'partial'\nexec sleep 30\n"
FIXED = "#!/bin/sh\n" + LOG + 'cat "$STUB_ANSWER"\n'
MODEL = "#!/bin/sh\n" + LOG + 'cat "$STUB_ANSWERS/$1-$2-$3-$4.json"\n'
SCAN_ONLY = "#!/bin/sh\n" + LOG + '[ "$1" = scan ] || exit 3\ncat "$STUB_ANSWERS/$1-$2-$3-$4.json"\n'
NO_CPU0 = "#!/bin/sh\necho \"taskset: failed to set pid 1's affinity: Invalid argument\" >&2\nexit 1\n"


def events(n, shape, cancel):
    """The BENCHMARK.md workload: arm all, rearm every fourth, cancel, then poll."""
    horizon = 256 if shape == "near" else 65536

    def deadline(i, offset):
        return horizon if shape == "burst" else 1 + (73 * i + offset) % horizon

    result = [("arm", i, deadline(i, 0)) for i in range(n)]
    result += [("arm", i, deadline(i, 17)) for i in range(0, n, 4)]
    result += [("cancel", i, 0) for i in range(n) if i % 10 < cancel // 10]
    if shape == "burst":
        return result + [("advance", 0, horizon)]
    return result + [("advance", 0, k * horizon // 256) for k in range(1, 257)]


def model(candidate, n, shape, cancel):
    """One accepted process answer, stepping the candidate's queue for its counters."""
    live, fired = {}, []
    heap, buckets, queued = [], [[] for _ in range(64)], 0
    inspections = ticks = now = 0
    peak = n if candidate == "scan" else 0
    for serial, (kind, i, value) in enumerate(events(n, shape, cancel)):
        if kind == "arm":
            live[i] = (value, serial)
            heap.append((value, i, serial))
            buckets[value % 64].append((value, i, serial))
            queued += 1
            peak = max(peak, {"scan": n, "heap": len(heap), "tree": len(live), "wheel": queued}[candidate])
        elif kind == "cancel":
            live.pop(i, None)
        else:
            due = sorted((d, i) for i, (d, _) in live.items() if d <= value)
            if candidate == "scan":
                inspections += n
            elif candidate == "tree":
                inspections += len(due)
            elif candidate == "heap":
                inspections += sum(1 for record in heap if record[0] <= value)
                heap = [record for record in heap if record[0] > value]
            else:
                while now < value:
                    now += 1
                    ticks += 1
                    kept = []
                    for d, i, s in buckets[now % 64]:
                        inspections += 1
                        if live.get(i) == (d, s) and d > now:
                            kept.append((d, i, s))
                    queued -= len(buckets[now % 64]) - len(kept)
                    buckets[now % 64] = kept
            for d, i in due:
                fired.append([value, d, i])
                del live[i]
    return {"ns": 1000, "fired": fired, "live": sorted([i, d] for i, (d, _) in live.items()),
            "inspections": inspections, "ticks": ticks, "peak_records": peak}


def declared_schedule():
    """The BENCHMARK.md process order: 48 warmups, 288 measured, 12 A/A."""
    schedule = []
    for n, shape, cancel in CELLS:
        cell = {"n": n, "shape": shape, "cancel": cancel}
        schedule += [dict(cell, candidate=c, phase="warmup", block=-1) for c in CANDIDATES]
        for block in range(6):
            order = CANDIDATES if block % 2 == 0 else CANDIDATES[::-1]
            schedule += [dict(cell, candidate=c, phase="main", block=block) for c in order]
    for block in range(6):
        for label in ("AB" if block % 2 == 0 else "BA"):
            schedule.append({"n": 4096, "shape": "near", "cancel": 90, "candidate": "heap",
                             "phase": "aa", "block": block, "label": label})
    return schedule


class RunnerTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.shared = tempfile.TemporaryDirectory()
        cls.answers = pathlib.Path(cls.shared.name)
        cls.models = {}
        for candidate, (n, shape, cancel) in itertools.product(CANDIDATES, CELLS):
            answer = cls.models[candidate, n, shape, cancel] = model(candidate, n, shape, cancel)
            (cls.answers / f"{candidate}-{n}-{shape}-{cancel}.json").write_text(json.dumps(answer))

    @classmethod
    def tearDownClass(cls):
        cls.shared.cleanup()

    def setUp(self):
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.dir = pathlib.Path(scratch.name)
        self.out = self.dir / "out"
        self.log = self.dir / "log"
        self.log.write_text("")

    def start(self, stub, answer=None, flags=(), **extra):
        """Run run.py on a stub binary. `answer` is the bytes FIXED prints for every process."""
        binary = self.dir / "binary"
        binary.write_text(stub)
        binary.chmod(0o755)
        if answer is not None:
            (self.dir / "answer").write_bytes(answer)
        env = {k: v for k, v in os.environ.items() if k != "PYTHONOPTIMIZE"}
        env.update(STUB_LOG=str(self.log), STUB_ANSWER=str(self.dir / "answer"), STUB_ANSWERS=str(self.answers))
        env.update(extra)
        command = [sys.executable, *flags, str(RUN), "--binary", str(binary), "--out", str(self.out)]
        return subprocess.run(command, capture_output=True, text=True, env=env, timeout=300, check=False)

    def records(self):
        path = self.out / "attempts.jsonl"
        self.assertTrue(path.exists(), "no attempts.jsonl was retained")
        return [json.loads(line) for line in path.read_text().splitlines()]

    def launches(self):
        return self.log.read_text().splitlines()

    def test_refuses_to_start_with_assertions_disabled(self):
        for flags, extra in [(("-O",), {}), (("-OO",), {}), ((), {"PYTHONOPTIMIZE": "1"})]:
            with self.subTest(flags=flags, extra=extra):
                process = self.start(MODEL, flags=flags, **extra)
                self.assertNotEqual(process.returncode, 0)
                self.assertIn("assertions", process.stderr)
                self.assertFalse(self.out.exists())
                self.assertEqual(self.launches(), [])

    def test_nonzero_exit_is_retained_as_failed_and_halts(self):
        process = self.start(FAILING)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual((self.out / "0000.stdout").read_text(), "stub output\n")
        self.assertEqual((self.out / "0000.stderr").read_text(), "stub failure\n")
        (record,) = self.records()
        self.assertEqual((record["status"], record["exit"], record["index"]), ("failed", 3, 0))
        self.assertEqual(record["command"][:3], ["taskset", "-c", "0"])
        self.assertNotIn("oracle", record)
        self.assertEqual(len(self.launches()), 1)

    def test_unavailable_cpu0_is_retained_as_failed_and_halts(self):
        tools = self.dir / "tools"
        tools.mkdir()
        (tools / "taskset").write_text(NO_CPU0)
        (tools / "taskset").chmod(0o755)
        process = self.start(MODEL, PATH=f"{tools}{os.pathsep}{os.environ['PATH']}")
        self.assertNotEqual(process.returncode, 0)
        (record,) = self.records()
        self.assertEqual((record["status"], record["exit"]), ("failed", 1))
        self.assertEqual(record["command"][:3], ["taskset", "-c", "0"])
        self.assertIn("affinity", (self.out / "0000.stderr").read_text())
        self.assertEqual(self.launches(), [])

    def test_timeout_is_retained_with_partial_output(self):
        process = self.start(HANGING, TOPIC_049_PROCESS_TIMEOUT_S="1")
        self.assertNotEqual(process.returncode, 0)
        self.assertTrue((self.out / "0000.stdout").exists(), "partial stdout was not retained")
        self.assertEqual((self.out / "0000.stdout").read_text(), "partial\n")
        self.assertTrue((self.out / "0000.stderr").exists())
        (record,) = self.records()
        self.assertEqual((record["status"], record["timeout_s"], record["index"]), ("timeout", 1.0, 0))
        self.assertNotIn("oracle", record)
        self.assertEqual(len(self.launches()), 1)

    def test_unusable_output_is_retained_as_rejected(self):
        good = self.models[FIRST]

        def changed(**fields):
            return json.dumps({**good, **fields}).encode()

        cases = {
            "text": b"not json",
            "not utf-8": b"\xff\xfe",
            "null": b"null",
            "array": b"[]",
            "missing fields": b'{"ns": 1}',
            "fired order": changed(fired=good["fired"][1::-1] + good["fired"][2:]),
            "fired missing": changed(fired=good["fired"][:-1]),
            "live extra": changed(live=[[0, 1]]),
            "ns true": changed(ns=True),
            "ns zero": changed(ns=0),
            "ns negative": changed(ns=-1),
            "ns fraction": changed(ns=1.5),
            "ns string": changed(ns="5"),
            "ns null": changed(ns=None),
            "ns infinity": json.dumps(good).replace('"ns": 1000', '"ns": Infinity').encode(),
            "ns nan": json.dumps(good).replace('"ns": 1000', '"ns": NaN').encode(),
            "inspections": changed(inspections=good["inspections"] - 1),
            "ticks": changed(ticks=good["ticks"] + 1),
            "peak_records": changed(peak_records=good["peak_records"] + 1),
            "ticks false": changed(ticks=False),
            "inspections float": changed(inspections=float(good["inspections"])),
            "counter missing": json.dumps({k: v for k, v in good.items() if k != "ticks"}).encode(),
        }
        self.assertEqual(good["ticks"], 0)
        self.assertGreater(len(good["fired"]), 2)
        for name, answer in cases.items():
            with self.subTest(name), tempfile.TemporaryDirectory() as scratch:
                self.out = pathlib.Path(scratch) / "out"
                self.log.write_text("")
                process = self.start(FIXED, answer=answer)
                self.assertNotEqual(process.returncode, 0)
                self.assertEqual(len(self.launches()), 1)
                (record,) = self.records()
                self.assertEqual(record["status"], "rejected")
                self.assertTrue(record["error"])
                self.assertNotIn("oracle", record)
                self.assertEqual((self.out / "0000.stdout").read_bytes().decode(errors="replace"),
                                 answer.decode(errors="replace"))

    def test_accepted_attempt_survives_a_later_failure(self):
        process = self.start(SCAN_ONLY)
        self.assertNotEqual(process.returncode, 0)
        first, second = self.records()
        good = self.models[FIRST]
        self.assertEqual((first["status"], first["oracle"], first["exit"]), ("ok", "pass", 0))
        self.assertEqual([first[k] for k in ("ns", "inspections", "ticks", "peak_records")],
                         [good[k] for k in ("ns", "inspections", "ticks", "peak_records")])
        self.assertEqual((second["status"], second["exit"], second["candidate"]), ("failed", 3, "heap"))
        self.assertEqual(len(self.launches()), 2)

    def test_complete_campaign_keeps_the_declared_schedule(self):
        process = self.start(MODEL)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(json.loads(process.stdout), {"processes": 348, "oracle": "all pass"})
        schedule = declared_schedule()
        self.assertEqual([sum(s["phase"] == p for s in schedule) for p in ("warmup", "main", "aa")], [48, 288, 12])
        self.assertEqual(json.loads((self.out / "schedule.json").read_text()), schedule)
        records = self.records()
        self.assertEqual(len(records), 348)
        binary = str((self.dir / "binary").resolve())
        for index, spec in enumerate(schedule):
            record = records[index]
            self.assertEqual({k: record[k] for k in spec}, spec)
            self.assertEqual((record["index"], record["status"], record["oracle"], record["exit"]), (index, "ok", "pass", 0))
            self.assertEqual(record["command"], ["taskset", "-c", "0", binary, spec["candidate"],
                                                 str(spec["n"]), spec["shape"], str(spec["cancel"])])
        self.assertEqual(self.launches(), [" ".join(r["command"][4:]) for r in records])
        names = [f"{i:04d}.{stream}" for i in range(348) for stream in ("stderr", "stdout")]
        self.assertEqual(sorted(p.name for p in self.out.iterdir()),
                         sorted(names + ["attempts.jsonl", "binary.sha256", "schedule.json"]))

    def test_refuses_an_existing_output_directory(self):
        self.out.mkdir()
        (self.out / "earlier").write_text("kept")
        process = self.start(MODEL)
        self.assertNotEqual(process.returncode, 0)
        self.assertEqual([p.name for p in self.out.iterdir()], ["earlier"])
        self.assertEqual(self.launches(), [])

    def test_oracle_models_match_the_step_simulation_and_published_counts(self):
        spec = importlib.util.spec_from_file_location("run_049", RUN)
        assert spec is not None and spec.loader is not None
        run = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(run)
        for (candidate, n, shape, cancel), answer in self.models.items():
            with self.subTest(candidate=candidate, n=n, shape=shape, cancel=cancel):
                fired, live = run.expected(n, shape, cancel)
                self.assertEqual((list(fired), list(live)), (answer["fired"], answer["live"]))
                self.assertEqual(tuple(run.work(candidate, n, shape, cancel)),
                                 (answer["inspections"], answer["ticks"], answer["peak_records"]))
        # measurements/README.md, 4096/far/0.
        published = {"scan": (1048576, 0, 4096), "heap": (5120, 0, 5120),
                     "tree": (4096, 0, 4096), "wheel": (1987584, 65536, 5120)}
        for candidate, counts in published.items():
            self.assertEqual(tuple(run.work(candidate, 4096, "far", 0)), counts)


if __name__ == "__main__":
    unittest.main()
