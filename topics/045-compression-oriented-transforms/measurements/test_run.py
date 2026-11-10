#!/usr/bin/env python3
"""Behavioral check for run.py attempt retention against stub rustc and taskset processes.

The runner copies its inputs into a temporary tree, so a stub `rustc` that writes an
executable for every `-o` target keeps the repository and the real toolchain out of the
test. That executable answers `dump <case> <mode>` with an empty workload whose frame
header the Python oracle accepts for every mode. A stub `taskset` plays the timed
experiment process. In the failure cases its first call exits 3 with diagnostic output,
prints text that is not JSON, prints partial output and sleeps past the one second
timeout set through TOPIC_045_PROCESS_TIMEOUT_S, prints JSON of the wrong shape, exits 2
after a byte that is not UTF-8, or prints a complete object whose mode or operation
differs from the invocation. The runner must write that attempt as a row in raw.jsonl,
then halt. One stub fails the first A/A control process after a complete main schedule.
One stub answers every invocation, and the 1020 retained rows must follow the declared
schedule. The last cases vary `dump`. A frame for another mode, a header shorter than five
bytes, and decodable frames whose packets differ from the Python packet encoding must halt
the runner before any timed attempt. Hand-derived frames for a 129-byte run and for a pair
that straddles a full literal must pass. Run from anywhere:
python3 topics/045-compression-oriented-transforms/measurements/test_run.py
"""
import collections
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

MEASUREMENTS = pathlib.Path(__file__).resolve().parent
CRATE = MEASUREMENTS.parent
INPUTS = ["src/lib.rs", "examples/experiment.rs", "measurements/run.py"]

RUSTC = """#!/bin/sh
if [ "$1" = -vV ] || [ "$1" = --print ]; then echo 'rustc stub'; exit 0; fi
while [ $# -gt 1 ]; do [ "$1" = -o ] && out=$2; shift; done
cat > "$out" <<'STUB'
#!/bin/sh
if [ "$1" = dump ]; then
    DUMP
else
    echo stub
fi
STUB
chmod +x "$out"
"""

FAILING_TASKSET = """#!/bin/sh
echo 'stub failure'
exit 3
"""

MALFORMED_TASKSET = """#!/bin/sh
echo 'not json'
"""

HANGING_TASKSET = """#!/bin/sh
echo 'partial'
sleep 5
"""

NULL_TASKSET = """#!/bin/sh
echo 'null'
"""

ARRAY_TASKSET = """#!/bin/sh
echo '[]'
"""

MISSING_FIELDS_TASKSET = """#!/bin/sh
echo '{"case":0,"mode":0,"operation":"encode"}'
"""

BINARY_TASKSET = """#!/bin/sh
printf 'bad \\377 byte\\n'
exit 2
"""

WRONG_MODE_TASKSET = """#!/bin/sh
echo '{"case":0,"mode":1,"operation":"encode","n":16,"bytes":69,"reps":4096,"ns":40.5}'
"""

WRONG_OPERATION_TASKSET = """#!/bin/sh
echo '{"case":0,"mode":0,"operation":"decode","n":16,"bytes":69,"reps":4096,"ns":40.5}'
"""

def dump(values, frame):
    """Shell command that prints one dump object."""
    return "echo '" + json.dumps({"values": values, "frame": frame}, separators=(",", ":")) + "'"


def dump_cases(cases, otherwise):
    """Shell `case` on `<case> <mode>` (the dump arguments `$2 $3`) with a default answer."""
    arms = "".join(f"    '{key}') {command} ;;\n" for key, command in cases.items())
    return 'case "$2 $3" in\n' + arms + f"    *) {otherwise} ;;\n    esac"


# Empty workload: the frame is the five-byte header for the requested mode.
EMPTY_DUMP = 'echo "{\\"values\\":[],\\"frame\\":[$3,0,0,0,0]}"'

# One zero word: four zero bytes, stored raw in mode 0 and as one four-byte run in modes 1-5.
ZERO_WORD_DUMP = 'if [ "$3" = 0 ]; then ' + dump([0], [0, 1, 0, 0, 0, 0, 0, 0, 0]) + '; else echo "{\\"values\\":[0],\\"frame\\":[$3,1,0,0,0,131,0]}"; fi'
ALL_LITERAL_DUMP = ZERO_WORD_DUMP.replace("131,0", "3,0,0,0,0")
SHORT_RUNS_DUMP = ZERO_WORD_DUMP.replace("131,0", "129,0,129,0")

# Mode 1 stores the little-endian bytes. 129 sevens then three zeros: a full 128-byte run,
# a one-byte literal for the remaining seven, and a three-byte run.
CAP_VALUES = [0x07070707] * 32 + [7]
CAP_FRAME = [1, 33, 0, 0, 0, 255, 7, 0, 7, 130, 0]
CAP_TAIL_AS_RUN_FRAME = [1, 33, 0, 0, 0, 255, 7, 128, 7, 130, 0]

# 127 alternating bytes, a pair of fives, then 1 2 3: the first five fills the 128-byte
# literal and the second five opens the next literal.
SPLIT_BYTES = [i % 2 for i in range(127)] + [5, 5, 1, 2, 3]
SPLIT_VALUES = [int.from_bytes(bytes(SPLIT_BYTES[i:i + 4]), "little") for i in range(0, 132, 4)]
SPLIT_FRAME = [1, 33, 0, 0, 0, 127, *SPLIT_BYTES[:128], 3, *SPLIT_BYTES[128:]]
SPLIT_FLUSHED_EARLY_FRAME = [1, 33, 0, 0, 0, 126, *SPLIT_BYTES[:127], 4, *SPLIT_BYTES[127:]]

# Arguments are `-c <cpu> ./experiment <operation> <case> <mode>`.
ECHO = 'echo "{\\"case\\":$5,\\"mode\\":$6,\\"operation\\":\\"$4\\",\\"n\\":16,\\"bytes\\":69,\\"reps\\":4096,\\"ns\\":40.5}"'

ECHOING_TASKSET = "#!/bin/sh\n" + ECHO + "\n"

# The main schedule runs `encode 10 0` seven times; the eighth call is the first A/A process.
AA_FAILING_TASKSET = """#!/bin/sh
if [ "$4 $5 $6" = 'encode 10 0' ]; then
    echo x >> "$TMPDIR/aa-calls"
    if [ "$(wc -l < "$TMPDIR/aa-calls")" -eq 8 ]; then echo 'control failure'; exit 4; fi
fi
""" + ECHO + "\n"


def run_with_stub(taskset, dump=EMPTY_DUMP):
    with tempfile.TemporaryDirectory(prefix="topic045-test-") as tmp:
        tmp = pathlib.Path(tmp)
        for rel in INPUTS:
            (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(CRATE / rel, tmp / rel)
        bin_dir = tmp / "bin"
        bin_dir.mkdir()
        for name, body in [("rustc", RUSTC.replace("DUMP", dump)), ("taskset", taskset)]:
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)
        env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ["PATH"], TMPDIR=str(tmp),
                   TOPIC_045_PROCESS_TIMEOUT_S="1")
        r = subprocess.run([sys.executable, str(tmp / "measurements/run.py")], cwd=tmp, env=env,
                           text=True, capture_output=True, timeout=300)
        output = r.stdout + r.stderr
        prefix = "Evidence directory: "
        evidence = [line[len(prefix):] for line in r.stdout.splitlines() if line.startswith(prefix)]
        rows, environment = [], {}
        if evidence:
            raw = pathlib.Path(evidence[0]) / "raw.jsonl"
            if raw.exists():
                rows = [json.loads(line) for line in raw.read_text().splitlines()]
            recorded = pathlib.Path(evidence[0]) / "environment.json"
            if recorded.exists():
                environment = json.loads(recorded.read_text())
        return r.returncode, output, rows, environment


class RunTests(unittest.TestCase):
    def assert_single_retained_attempt(self, returncode, output, rows, status):
        self.assertNotEqual(returncode, 0, output)
        self.assertEqual(len(rows), 1, output)
        row = rows[0]
        self.assertEqual(row["status"], status, row)
        self.assertEqual((row["operation"], row["case"], row["mode"]), ("encode", 0, 0), row)
        self.assertEqual((row["block"], row["position"], row["warmup"]), (-1, 0, True), row)
        self.assertEqual(row["args"][3:], ["./experiment", "encode", "0", "0"], row)
        self.assertNotIn("ns", row)
        self.assertIn("retained in raw.jsonl", output)

    def test_nonzero_exit_is_retained_before_halting(self):
        returncode, output, rows, _ = run_with_stub(FAILING_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, "failed")
        self.assertEqual((rows[0]["returncode"], rows[0]["output"]), (3, "stub failure\n"), rows[0])

    def test_malformed_output_is_retained_before_halting(self):
        returncode, output, rows, _ = run_with_stub(MALFORMED_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, "malformed")
        self.assertEqual(rows[0]["output"], "not json\n", rows[0])
        self.assertIn("JSONDecodeError", rows[0]["error"])

    def test_timed_out_process_is_retained_before_halting(self):
        returncode, output, rows, _ = run_with_stub(HANGING_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, "timeout")
        self.assertEqual((rows[0]["timeout_s"], rows[0]["output"]), (1.0, "partial\n"), rows[0])
        self.assertNotIn("returncode", rows[0])

    def test_wrong_shape_json_is_retained_as_malformed_before_halting(self):
        for stub, printed in [(NULL_TASKSET, "null\n"), (ARRAY_TASKSET, "[]\n"),
                              (MISSING_FIELDS_TASKSET, '{"case":0,"mode":0,"operation":"encode"}\n')]:
            with self.subTest(printed=printed):
                returncode, output, rows, _ = run_with_stub(stub)
                self.assert_single_retained_attempt(returncode, output, rows, "malformed")
                self.assertEqual(rows[0]["output"], printed, rows[0])
                self.assertIn("measurement object missing", rows[0]["error"])
                self.assertNotIn("reps", rows[0])

    def test_non_utf8_output_is_retained_before_halting(self):
        returncode, output, rows, _ = run_with_stub(BINARY_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, "failed")
        self.assertEqual((rows[0]["returncode"], rows[0]["output"]), (2, "bad \ufffd byte\n"), rows[0])

    def test_output_labels_must_match_the_invocation(self):
        for stub, printed in [(WRONG_MODE_TASKSET, '"mode": 1'), (WRONG_OPERATION_TASKSET, '"operation": "decode"')]:
            with self.subTest(printed=printed):
                returncode, output, rows, _ = run_with_stub(stub)
                self.assert_single_retained_attempt(returncode, output, rows, "malformed")
                self.assertIn(printed, rows[0]["error"])
                self.assertIn("differ from invocation", rows[0]["error"])

    def test_a_failed_control_process_is_retained_before_halting(self):
        returncode, output, rows, _ = run_with_stub(AA_FAILING_TASKSET)
        self.assertNotEqual(returncode, 0, output)
        self.assertEqual(len(rows), 1009, output)
        self.assertTrue(all(row["status"] == "ok" for row in rows[:-1]), output)
        last = rows[-1]
        self.assertEqual((last["status"], last["returncode"], last["output"]), ("failed", 4, "control failure\n"), last)
        self.assertEqual((last["control"], last["label"], last["block"], last["warmup"]), ("AA", "A", 0, False), last)
        self.assertEqual((last["operation"], last["case"], last["mode"]), ("encode", 10, 0), last)
        self.assertIn("retained in raw.jsonl", output)

    def test_a_complete_run_retains_the_declared_schedule(self):
        returncode, output, rows, environment = run_with_stub(ECHOING_TASKSET)
        self.assertEqual(returncode, 0, output)
        self.assertIn('"status": "complete"', output)
        self.assertEqual(environment["process_timeout_s"], 1.0)
        self.assertEqual(len(rows), 1020, output)
        for row in rows:
            self.assertEqual(row["status"], "ok", row)
            self.assertEqual(row["args"][3:], ["./experiment", row["operation"], str(row["case"]), str(row["mode"])], row)
        control = [row for row in rows if row.get("control") == "AA"]
        main = [row for row in rows if "control" not in row]
        self.assertEqual((len(main), len([row for row in main if row["warmup"]]), len(control)), (1008, 144, 12))
        self.assertEqual(rows[-12:], control)
        self.assertEqual([row["label"] for row in control], list("ABBAABBAABBA"))
        self.assertTrue(all((row["operation"], row["case"], row["mode"]) == ("encode", 10, 0) for row in control))
        positions = collections.defaultdict(set)
        for row in main:
            if not row["warmup"]:
                positions[row["case"], row["operation"], row["mode"]].add(row["position"])
        self.assertEqual(len(positions), 12 * 2 * 6)
        self.assertTrue(all(seen == set(range(6)) for seen in positions.values()), positions)

    def test_frames_must_satisfy_the_python_oracle_before_timing(self):
        returncode, output, rows, _ = run_with_stub(ECHOING_TASKSET, dump=EMPTY_DUMP.replace("$3", "6"))
        self.assertNotEqual(returncode, 0, output)
        self.assertIn("assert frame[0] == mode", output)
        self.assertEqual(rows, [], output)

    def test_frames_equal_to_the_python_encoding_are_accepted(self):
        known = {"0 1": dump(CAP_VALUES, CAP_FRAME), "1 1": dump(SPLIT_VALUES, SPLIT_FRAME)}
        returncode, output, rows, _ = run_with_stub(ECHOING_TASKSET, dump=dump_cases(known, ZERO_WORD_DUMP))
        self.assertEqual(returncode, 0, output)
        self.assertEqual(len(rows), 1020, output)

    def test_workload_frames_must_equal_the_python_encoding_before_timing(self):
        rejected = {
            "one literal for four equal bytes": ALL_LITERAL_DUMP,
            "two short runs for four equal bytes": SHORT_RUNS_DUMP,
            "run tag on the byte left after a full run": dump_cases({"0 1": dump(CAP_VALUES, CAP_TAIL_AS_RUN_FRAME)}, ZERO_WORD_DUMP),
            "literal closed before its 128th byte": dump_cases({"1 1": dump(SPLIT_VALUES, SPLIT_FLUSHED_EARLY_FRAME)}, ZERO_WORD_DUMP),
            "header shorter than five bytes": 'echo "{\\"values\\":[],\\"frame\\":[$3]}"',
        }
        for name, stub in rejected.items():
            with self.subTest(name):
                returncode, output, rows, _ = run_with_stub(ECHOING_TASKSET, dump=stub)
                self.assertNotEqual(returncode, 0, output)
                self.assertIn("assert frame == [mode", output)
                self.assertEqual(rows, [], output)


if __name__ == "__main__":
    unittest.main()
