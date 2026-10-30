#!/usr/bin/env python3
"""Behavioral checks for run.py against stub cargo, taskset and objdump processes.

The crate's measured input files are copied into a temporary tree so the runner's
build directory and output never touch the repository. A stub `cargo` answers the
test and example steps, writes the bench executable for the build step and reports
its path the way Cargo's JSON message stream does. A stub `taskset` runs the pinned
command directly. A stub `objdump` succeeds or fails on request.
Run from anywhere: python3 topics/034-dominator-trees/measurements/test_run.py
"""
import hashlib
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
INPUTS = ['Cargo.toml', 'measurements/portable/Cargo.toml', 'measurements/portable/Cargo.lock', 'src/lib.rs',
          'tests/contracts.rs', 'examples/demo.rs', 'benches/dominators.rs', 'measurements/run.py']
DEPS = pathlib.Path('measurements/portable/target/release/deps')

BENCH = "#!/bin/sh\n# %s\nprintf '{\"candidate\":\"%%s\",\"shape\":\"%%s\",\"n\":%%s,\"reps\":%%s}\\n' \"$1\" \"$2\" \"$3\" \"$4\"\n"

CARGO = """#!/usr/bin/env python3
import json, os, pathlib, stat, sys
args = sys.argv[1:]
if args[0] == 'build':
    deps = pathlib.Path(os.environ['STUB_DEPS'])
    deps.mkdir(parents=True, exist_ok=True)
    fresh = deps / 'dominators-7777777777777777'
    fresh.write_text(os.environ['STUB_FRESH'])
    fresh.chmod(fresh.stat().st_mode | stat.S_IXUSR)
    print(json.dumps({'reason': 'compiler-artifact', 'target': {'name': 'dominators', 'kind': ['bench']},
                      'executable': str(fresh.resolve())}))
    print(json.dumps({'reason': 'build-finished', 'success': True}))
print('stub cargo ok')
"""

TASKSET = "#!/bin/sh\nshift 2\nexec \"$@\"\n"

OBJDUMP = "#!/bin/sh\nif [ -n \"$STUB_OBJDUMP_FAIL\" ]; then echo 'stub objdump failed' >&2; exit 1; fi\necho 'stub disassembly'\n"


def sha256(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def run_runner(tmp, extra_env, stale_names=()):
    tmp = pathlib.Path(tmp)
    for rel in INPUTS:
        dst = tmp / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(CRATE / rel, dst)
    (tmp / DEPS).mkdir(parents=True)
    for name in stale_names:
        stale = tmp / DEPS / name
        stale.write_text(BENCH % ('stale ' + name))
        stale.chmod(0o755)
    bin_dir = tmp / 'bin'
    bin_dir.mkdir()
    for name, body in [('cargo', CARGO), ('taskset', TASKSET), ('objdump', OBJDUMP)]:
        tool = bin_dir / name
        tool.write_text(body)
        tool.chmod(0o755)
    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'],
               STUB_DEPS=str(tmp / DEPS), STUB_FRESH=BENCH % 'fresh')
    env.pop('STUB_OBJDUMP_FAIL', None)
    env.update(extra_env)
    out = tmp / 'out'
    r = subprocess.run([sys.executable, str(tmp / 'measurements/run.py'), str(out)], cwd=tmp, env=env,
                       text=True, capture_output=True, timeout=600)
    return r.returncode, r.stdout + r.stderr, out


class RunTests(unittest.TestCase):
    def test_completion_marker_is_written_only_after_assembly_is_retained(self):
        with tempfile.TemporaryDirectory(prefix='topic034-') as tmp:
            code, output, out = run_runner(tmp, {'STUB_OBJDUMP_FAIL': '1'})
            self.assertNotEqual(code, 0, output)
            self.assertFalse((out / 'assembly.txt').exists() and (out / 'assembly.txt').read_bytes(), output)
            self.assertFalse((out / 'finished.json').exists(),
                             'finished.json was written although assembly retention failed: ' + output)
            self.assertEqual(len((out / 'samples.jsonl').read_text().splitlines()), 210)

    def test_runner_times_the_executable_cargo_reports_for_this_build(self):
        stale = ['dominators-' + c * 16 for c in '0369cf']
        with tempfile.TemporaryDirectory(prefix='topic034-') as tmp:
            code, output, out = run_runner(tmp, {}, stale_names=stale)
            self.assertEqual(code, 0, output)
            environment = json.loads((out / 'environment.json').read_text())
            fresh = pathlib.Path(tmp, DEPS, 'dominators-7777777777777777')
            self.assertEqual(environment['binary_sha256'], sha256(fresh),
                             'the runner hashed a retained executable instead of the one Cargo just built')
            finished = json.loads((out / 'finished.json').read_text())
            self.assertEqual((finished['complete'], finished['records']), (True, 210))
            self.assertEqual((out / 'assembly.txt').read_text(), 'stub disassembly\n')


if __name__ == '__main__':
    unittest.main()
