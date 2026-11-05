#!/usr/bin/env python3
"""Behavioral check for run.py against stub rustc, lscpu, objdump and taskset processes.

The crate's measured input files are copied into a temporary tree so the runner's
build output never touches the repository. A stub `rustc` reports a version and
writes an executable for every `-o` target. A stub `taskset` plays the search
binary: its first call sleeps past the runner's 60 second timeout, its second
prints text that is not JSON, its third exits 1, and every later call prints a
valid result. Run from anywhere: python3 topics/039-adaptive-string-search/measurements/test_run.py
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
INPUTS = ['src/lib.rs', 'benches/search.rs', 'tests/contracts.rs', 'examples/search.rs', 'measurements/run.py']
PROCESSES = 1816

RUSTC = """#!/bin/sh
if [ "$1" = -Vv ]; then echo 'rustc stub'; exit 0; fi
while [ $# -gt 1 ]; do [ "$1" = -o ] && out=$2; shift; done
printf '#!/bin/sh\\necho stub\\n' > "$out" && chmod +x "$out"
"""

TASKSET = """#!/bin/sh
count=$(cat "$STUB_COUNT" 2>/dev/null || echo 0)
echo $((count + 1)) > "$STUB_COUNT"
case $count in
  0) sleep 70 ;;
  1) echo 'not json' ;;
  2) echo 'stub failure' >&2; exit 1 ;;
  *) echo '{"ns_total":1000,"iterations":10,"comparisons":1,"windows":1,"fallbacks":0,"answer":0,"checksum":0}' ;;
esac
"""

ECHO = "#!/bin/sh\necho stub\n"


class RunTests(unittest.TestCase):
    def test_failed_and_timed_out_attempts_are_retained_and_the_schedule_completes(self):
        with tempfile.TemporaryDirectory(prefix='topic039-') as tmp:
            tmp = pathlib.Path(tmp)
            for rel in INPUTS:
                (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(CRATE / rel, tmp / rel)
            bin_dir = tmp / 'bin'
            bin_dir.mkdir()
            for name, body in [('rustc', RUSTC), ('taskset', TASKSET), ('lscpu', ECHO), ('objdump', ECHO)]:
                (bin_dir / name).write_text(body)
                (bin_dir / name).chmod(0o755)
            env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'], STUB_COUNT=str(tmp / 'count'))
            out = tmp / 'out'
            r = subprocess.run([sys.executable, str(tmp / 'measurements/run.py'), str(out)], cwd=tmp, env=env,
                               text=True, capture_output=True, timeout=900)
            output = r.stdout + r.stderr
            self.assertEqual(r.returncode, 0, output)
            records = [json.loads(line) for line in (out / 'results.jsonl').read_text().splitlines()]
            self.assertEqual(len(records), PROCESSES, output)
            self.assertEqual([rec['index'] for rec in records], list(range(PROCESSES)))
            timed_out, malformed, failed = records[:3]
            self.assertIsNone(timed_out['returncode'], timed_out)
            self.assertIn('TimeoutExpired', timed_out['error'])
            self.assertEqual((malformed['returncode'], malformed['stdout']), (None, 'not json\n'), malformed)
            self.assertEqual((failed['returncode'], failed['stderr']), (1, 'stub failure\n'), failed)
            self.assertTrue(all('ns_per_query' not in rec for rec in records[:3]))
            self.assertTrue(all(rec['ns_per_query'] == 100.0 for rec in records[3:]), output)
            self.assertEqual(json.loads((out / 'complete.json').read_text())['processes'], PROCESSES)


if __name__ == '__main__':
    unittest.main()
