#!/usr/bin/env python3
"""Behavioral check for run.py against stub rustc, lscpu, objdump and taskset processes.

The crate's measured input files are copied into a temporary tree so the runner's
build output never touches the repository. A stub `rustc` reports a version and
writes an executable for every `-o` target. A stub `taskset` plays the chunk
binary: its first call writes partial output and sleeps past the runner's one
second test timeout, its second exits 1, its third prints text that is not JSON,
its fourth prints a JSON array, and every later call prints a well-formed result
whose boundary vectors disagree with the oracle. Every attempt is retained and the
schedule completes. Run from anywhere:
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

TASKSET = """#!/bin/sh
count=$(cat "$STUB_COUNT" 2>/dev/null || echo 0)
echo $((count + 1)) > "$STUB_COUNT"
case $count in
  0) echo 'partial' >&2; exec sleep 5 ;;
  1) echo 'stub failure' >&2; exit 1 ;;
  2) echo 'not json' ;;
  3) echo '[1, 2]' ;;
  *) echo '{"ns_total":1000,"iterations":10,"ends":[1],"edited_ends":[1]}' ;;
esac
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
            timed_out, failed, malformed, array = records[:4]
            self.assertEqual((timed_out['returncode'], timed_out['stderr']), (None, 'partial\n'), timed_out)
            self.assertIn('TimeoutExpired', timed_out['error'])
            self.assertEqual((failed['returncode'], failed['stderr']), (1, 'stub failure\n'), failed)
            self.assertEqual((malformed['returncode'], malformed['stdout']), (None, 'not json\n'), malformed)
            self.assertEqual((array['returncode'], array['stdout']), (None, '[1, 2]\n'), array)
            self.assertTrue(all('ns_per_operation' not in rec for rec in records[:4]))
            self.assertTrue(all(rec['ns_per_operation'] == 100.0 and rec['oracle_ok'] is False for rec in records[4:]), output)
            complete = json.loads((out / 'complete.json').read_text())
            self.assertEqual((complete['processes'], complete['failures']), (PROCESSES, PROCESSES))
            manifest = json.loads((out / 'manifest.json').read_text())
            self.assertEqual(set(manifest), {p.name for p in out.iterdir()} - {'manifest.json'})
            self.assertIn('results.jsonl', manifest)


if __name__ == '__main__':
    unittest.main()
