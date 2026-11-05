#!/usr/bin/env python3
"""Behavioral check for run.py against stub rustc, lscpu, objdump and taskset processes.

The crate's measured input files are copied into a temporary tree so the runner's
build output never touches the repository. A stub `rustc` reports a version and
writes an executable for every `-o` target. A stub `taskset` plays the chunk
binary: its first call writes partial output and sleeps past the runner's one
second test timeout, its second exits 1, its third prints text that is not JSON,
its fourth a JSON array, its fifth an object missing the boundary keys, its sixth
an object with zero iterations that also carries runner keys, and every later call
prints a well-formed result whose boundary vectors match the oracle for `fixed`
and disagree with it for every other candidate. Every attempt is retained, the
schedule completes, and `failures` counts the non-passing processes. Run from
anywhere:
python3 topics/040-rolling-hashes-and-content-defined-chunking/measurements/test_run.py
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
INPUTS = ['src/lib.rs', 'benches/chunk.rs', 'tests/contracts.rs', 'examples/chunk.rs', 'measurements/run.py']
PROCESSES = 556

RUSTC = """#!/bin/sh
if [ "$1" = -Vv ] || [ "$1" = --print ]; then echo 'rustc stub'; exit 0; fi
while [ $# -gt 1 ]; do [ "$1" = -o ] && out=$2; shift; done
printf '#!/bin/sh\\necho stub\\n' > "$out" && chmod +x "$out"
"""

TASKSET = f"""#!{sys.executable}
import json, os, sys, time
path = os.environ['STUB_COUNT']
count = int(open(path).read()) if os.path.exists(path) else 0
open(path, 'w').write(str(count + 1))
if count == 0:
    sys.stderr.write('partial\\n'); sys.stderr.flush(); time.sleep(5); sys.exit(0)
if count == 1:
    sys.stderr.write('stub failure\\n'); sys.exit(1)
if count == 2:
    print('not json'); sys.exit(0)
if count == 3:
    print('[1, 2]'); sys.exit(0)
if count == 4:
    print('{{"ns_total":1000,"iterations":10}}'); sys.exit(0)
if count == 5:
    print('{{"ns_total":1,"iterations":0,"oracle_ok":true,"ends":[256],"edited_ends":[256,273]}}'); sys.exit(0)
candidate, n, target = sys.argv[4], int(sys.argv[5]), int(sys.argv[6])
def fixed(length):
    return list(range(target, length, target)) + ([length] if length else [])
ends, edited = (fixed(n), fixed(n + 17)) if candidate == 'fixed' else ([1], [1])
print(json.dumps({{'ns_total': 1000, 'iterations': 10, 'ends': ends, 'edited_ends': edited}}))
"""

ECHO = "#!/bin/sh\necho stub\n"


class RunTests(unittest.TestCase):
    def test_failed_and_timed_out_attempts_are_retained_and_the_schedule_completes(self):
        with tempfile.TemporaryDirectory(prefix='topic040-') as tmp:
            tmp = pathlib.Path(tmp)
            for rel in INPUTS:
                (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy(CRATE / rel, tmp / rel)
            bin_dir = tmp / 'bin'
            bin_dir.mkdir()
            for name, body in [('rustc', RUSTC), ('taskset', TASKSET), ('lscpu', ECHO), ('objdump', ECHO)]:
                (bin_dir / name).write_text(body)
                (bin_dir / name).chmod(0o755)
            env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'], STUB_COUNT=str(tmp / 'count'),
                       TOPIC_040_PROCESS_TIMEOUT_S='1')
            out = tmp / 'out'
            r = subprocess.run([sys.executable, str(tmp / 'measurements/run.py'), str(out)], cwd=tmp, env=env,
                               text=True, capture_output=True, timeout=600)
            output = r.stdout + r.stderr
            self.assertEqual(r.returncode, 0, output)
            records = [json.loads(line) for line in (out / 'results.jsonl').read_text().splitlines()]
            self.assertEqual(len(records), PROCESSES, output)
            self.assertEqual([rec['index'] for rec in records], list(range(PROCESSES)))
            timed_out, failed, malformed, array, partial, zero_iterations = records[:6]
            self.assertEqual((timed_out['returncode'], timed_out['stderr']), (None, 'partial\n'), timed_out)
            self.assertIn('TimeoutExpired', timed_out['error'])
            self.assertEqual((failed['returncode'], failed['stderr']), (1, 'stub failure\n'), failed)
            self.assertEqual((malformed['returncode'], malformed['stdout']), (None, 'not json\n'), malformed)
            self.assertEqual((array['returncode'], array['stdout']), (None, '[1, 2]\n'), array)
            self.assertEqual(partial['returncode'], None, partial)
            self.assertIn('KeyError', partial['error'])
            self.assertEqual(zero_iterations['returncode'], None, zero_iterations)
            self.assertIn('ZeroDivisionError', zero_iterations['error'])
            for rec in records[:6]:
                self.assertFalse({'ns_total', 'iterations', 'ns_per_operation', 'ends', 'edited_ends'} & set(rec), rec)
                self.assertIs(rec['oracle_ok'], False, rec)
            passing = [rec for rec in records[6:] if rec['candidate'] == 'fixed']
            # 12 cells x (1 warmup + 8 blocks) fixed processes; indices 0 and 5 are stub failure cases.
            self.assertEqual(len(passing), 12 * 9 - 2)
            for rec in records[6:]:
                self.assertEqual((rec['returncode'], rec['ns_per_operation']), (0, 100.0), rec)
                self.assertIs(rec['oracle_ok'], rec['candidate'] == 'fixed', rec)
            complete = json.loads((out / 'complete.json').read_text())
            self.assertEqual((complete['processes'], complete['failures']), (PROCESSES, PROCESSES - len(passing)))
            manifest = json.loads((out / 'manifest.json').read_text())
            self.assertEqual(set(manifest), {p.name for p in out.iterdir()} - {'manifest.json'})
            self.assertIn('results.jsonl', manifest)


if __name__ == '__main__':
    unittest.main()
