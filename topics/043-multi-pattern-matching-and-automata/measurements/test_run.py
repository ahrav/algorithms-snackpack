#!/usr/bin/env python3
"""Behavioral check for run.py failure retention against stub rustc and taskset processes.

The runner copies its inputs into a temporary tree, so a stub `rustc` that writes
an executable for every `-o` target keeps the repository and the real toolchain out
of the test. A stub `taskset` plays the experiment binary: in one case its first
call exits 3 with diagnostic output, in the second it prints text that is not JSON,
and in the third it prints partial output and sleeps past the one second timeout
set through TOPIC_043_PROCESS_TIMEOUT_S. A fourth group prints JSON of the wrong
shape: `null`, `[]`, and an object missing the timing fields. A fifth stub exits 2
after printing a byte that is not UTF-8. A sixth prints a complete object whose
candidate label differs from the one the runner invoked. The runner must
write that attempt as a row in raw.jsonl, then halt. Run from
anywhere:
python3 topics/043-multi-pattern-matching-and-automata/measurements/test_run.py
"""
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
INPUTS = ['src/lib.rs', 'examples/experiment.rs', 'measurements/run.py']

RUSTC = """#!/bin/sh
if [ "$1" = -vV ] || [ "$1" = --print ]; then echo 'rustc stub'; exit 0; fi
while [ $# -gt 1 ]; do [ "$1" = -o ] && out=$2; shift; done
printf '#!/bin/sh\\necho stub\\n' > "$out" && chmod +x "$out"
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
echo '{"candidate":"dense","case":0,"endpoint":"scan"}'
"""

BINARY_TASKSET = """#!/bin/sh
printf 'bad \\377 byte\\n'
exit 2
"""

MISLABELED_TASKSET = """#!/bin/sh
echo '{"candidate":"direct","case":0,"endpoint":"scan","n":64,"r":2,"states":25,"table_bytes":51200,"matches":0,"reps":2000,"elapsed_ns":1000,"checksum":0}'
"""


def run_with_stub(taskset):
    with tempfile.TemporaryDirectory(prefix='topic043-test-') as tmp:
        tmp = pathlib.Path(tmp)
        for rel in INPUTS:
            (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(CRATE / rel, tmp / rel)
        bin_dir = tmp / 'bin'
        bin_dir.mkdir()
        for name, body in [('rustc', RUSTC), ('taskset', taskset)]:
            (bin_dir / name).write_text(body)
            (bin_dir / name).chmod(0o755)
        env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'], TMPDIR=str(tmp),
                   TOPIC_043_PROCESS_TIMEOUT_S='1')
        r = subprocess.run([sys.executable, str(tmp / 'measurements/run.py')], cwd=tmp, env=env,
                           text=True, capture_output=True, timeout=300)
        output = r.stdout + r.stderr
        prefix = 'Evidence directory: '
        evidence = [line[len(prefix):] for line in r.stdout.splitlines() if line.startswith(prefix)]
        rows = []
        if evidence:
            raw = pathlib.Path(evidence[0]) / 'raw.jsonl'
            if raw.exists():
                rows = [json.loads(line) for line in raw.read_text().splitlines()]
        return r.returncode, output, rows


class RunTests(unittest.TestCase):
    def assert_single_retained_attempt(self, returncode, output, rows, status):
        self.assertNotEqual(returncode, 0, output)
        self.assertEqual(len(rows), 1, output)
        row = rows[0]
        self.assertEqual(row['status'], status, row)
        self.assertEqual((row['candidate'], row['case'], row['endpoint']), ('dense', 0, 'scan'), row)
        self.assertEqual((row['block'], row['position'], row['warmup']), (-1, 0, True), row)
        self.assertEqual(row['args'][3:], ['./experiment', 'dense', '0', 'scan'], row)
        self.assertNotIn('elapsed_ns', row)
        self.assertIn('retained in raw.jsonl', output)

    def test_nonzero_exit_is_retained_before_halting(self):
        returncode, output, rows = run_with_stub(FAILING_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, 'failed')
        self.assertEqual((rows[0]['returncode'], rows[0]['output']), (3, 'stub failure\n'), rows[0])

    def test_malformed_output_is_retained_before_halting(self):
        returncode, output, rows = run_with_stub(MALFORMED_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, 'malformed')
        self.assertEqual(rows[0]['output'], 'not json\n', rows[0])
        self.assertIn('JSONDecodeError', rows[0]['error'])

    def test_timed_out_process_is_retained_before_halting(self):
        returncode, output, rows = run_with_stub(HANGING_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, 'timeout')
        self.assertEqual((rows[0]['timeout_s'], rows[0]['output']), (1.0, 'partial\n'), rows[0])
        self.assertNotIn('returncode', rows[0])

    def test_wrong_shape_json_is_retained_as_malformed_before_halting(self):
        for stub, printed in [(NULL_TASKSET, 'null\n'), (ARRAY_TASKSET, '[]\n'),
                              (MISSING_FIELDS_TASKSET, '{"candidate":"dense","case":0,"endpoint":"scan"}\n')]:
            with self.subTest(printed=printed):
                returncode, output, rows = run_with_stub(stub)
                self.assert_single_retained_attempt(returncode, output, rows, 'malformed')
                self.assertEqual(rows[0]['output'], printed, rows[0])
                self.assertIn('measurement object missing', rows[0]['error'])
                self.assertNotIn('reps', rows[0])

    def test_non_utf8_output_is_retained_before_halting(self):
        returncode, output, rows = run_with_stub(BINARY_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, 'failed')
        self.assertEqual((rows[0]['returncode'], rows[0]['output']), (2, 'bad \ufffd byte\n'), rows[0])

    def test_output_labels_must_match_the_invocation(self):
        returncode, output, rows = run_with_stub(MISLABELED_TASKSET)
        self.assert_single_retained_attempt(returncode, output, rows, 'malformed')
        self.assertIn('"candidate": "direct"', rows[0]['error'])
        self.assertIn('differ from invocation', rows[0]['error'])


if __name__ == '__main__':
    unittest.main()
