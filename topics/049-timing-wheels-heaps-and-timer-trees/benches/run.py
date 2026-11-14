#!/usr/bin/env python3
"""Fixed six-block schedule; independent full-output and work-counter oracle for each process."""
import argparse
import functools
import hashlib
import itertools
import json
import os
import subprocess
from pathlib import Path

if not __debug__:
    raise SystemExit('run.py needs assertions enabled: run it without -O and with PYTHONOPTIMIZE unset')

CANDIDATES = ['scan', 'heap', 'tree', 'wheel']
TIMEOUT_S = float(os.environ.get('TOPIC_049_PROCESS_TIMEOUT_S', '60'))

@functools.cache
def expected(n, shape, cancel):
    horizon = 256 if shape == 'near' else 65536
    def deadline(i, offset):
        return horizon if shape == 'burst' else 1 + (i * 73 + offset) % horizon
    live = {i: deadline(i, 0) for i in range(n)}
    for i in range(0, n, 4):
        live[i] = deadline(i, 17)
    for i in range(n):
        if i % 10 < cancel // 10:
            live.pop(i, None)
    polls = [horizon] if shape == 'burst' else range(horizon // 256, horizon + 1, horizon // 256)
    fired = []
    for poll in polls:
        due = sorted((d, i) for i, d in live.items() if d <= poll)
        for d, i in due:
            fired.append([poll, d, i])
            del live[i]
    return fired, sorted([i, d] for i, d in live.items())

@functools.cache
def work(candidate, n, shape, cancel):
    """(inspections, ticks, peak_records) from the workload: every arm and cancel precedes the first poll."""
    horizon = 256 if shape == 'near' else 65536
    def deadline(i, offset):
        return horizon if shape == 'burst' else 1 + (i * 73 + offset) % horizon
    arms = n + len(range(0, n, 4))
    live = [deadline(i, 17 if i % 4 == 0 else 0) for i in range(n) if i % 10 >= cancel // 10]
    if candidate == 'scan':
        # Every poll inspects every ID slot.
        return n * (1 if shape == 'burst' else 256), 0, n
    if candidate == 'heap':
        # Every record is popped once by the last poll, which equals the horizon.
        return arms, 0, arms
    if candidate == 'tree':
        # A rearm replaces its key and a cancel removes it, so only live keys are removed as due.
        return len(live), 0, n
    # A replaced or cancelled record leaves at its first slot visit. A live record is
    # visited once per revolution until its deadline: ceil(deadline / 64) times.
    stale = arms - len(live)
    return stale + sum(-(-d // 64) for d in live), horizon, arms

def check(spec, stdout):
    """Timing and counters of one accepted output; raises when the output is rejected."""
    data = json.loads(stdout)
    want_fired, want_live = expected(spec['n'], spec['shape'], spec['cancel'])
    assert data['fired'] == want_fired and data['live'] == want_live, 'fired or live differs from the oracle'
    counters = (data['inspections'], data['ticks'], data['peak_records'])
    assert all(type(value) is int for value in counters), 'a work counter is not an integer'
    assert counters == work(spec['candidate'], spec['n'], spec['shape'], spec['cancel']), 'work counters differ from the oracle'
    assert type(data['ns']) is int and data['ns'] > 0, 'ns is not a positive integer'
    return {'ns': data['ns'], 'inspections': counters[0], 'ticks': counters[1], 'peak_records': counters[2]}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True)
    parser.add_argument('--out', required=True)
    args = parser.parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=False)
    binary = str(Path(args.binary).resolve())
    schedule = []
    for n, shape, cancel in itertools.product([16, 4096], ['near', 'far', 'burst'], [0, 90]):
        for c in CANDIDATES:
            schedule.append(dict(n=n, shape=shape, cancel=cancel, candidate=c, phase='warmup', block=-1))
        for block in range(6):
            order = CANDIDATES if block % 2 == 0 else list(reversed(CANDIDATES))
            for c in order:
                schedule.append(dict(n=n, shape=shape, cancel=cancel, candidate=c, phase='main', block=block))
    for block in range(6):
        labels = ['A', 'B'] if block % 2 == 0 else ['B', 'A']
        for label in labels:
            schedule.append(dict(n=4096, shape='near', cancel=90, candidate='heap', phase='aa', block=block, label=label))
    (out / 'schedule.json').write_text(json.dumps(schedule, indent=2) + '\n')
    (out / 'binary.sha256').write_text(hashlib.sha256(Path(binary).read_bytes()).hexdigest() + '\n')
    with (out / 'attempts.jsonl').open('w') as attempts:
        for index, spec in enumerate(schedule):
            cmd = ['taskset', '-c', '0', binary, spec['candidate'], str(spec['n']), spec['shape'], str(spec['cancel'])]
            record: dict[str, object] = dict(spec, index=index, command=cmd)
            try:
                process = subprocess.run(cmd, capture_output=True, timeout=TIMEOUT_S, check=False)
                stdout, stderr = process.stdout, process.stderr
                record.update(exit=process.returncode, status='ok' if process.returncode == 0 else 'failed')
            except subprocess.TimeoutExpired as expired:
                stdout, stderr = expired.stdout or b'', expired.stderr or b''
                record.update(status='timeout', timeout_s=TIMEOUT_S)
            (out / f'{index:04d}.stdout').write_bytes(stdout)
            (out / f'{index:04d}.stderr').write_bytes(stderr)
            if record['status'] == 'ok':
                try:
                    record.update(check(spec, stdout.decode(errors='replace')), oracle='pass')
                except Exception as error:  # any parse or oracle failure is a rejected attempt
                    record.update(status='rejected', error=repr(error))
            # One classified line per attempt, on disk before the next process or the halt.
            attempts.write(json.dumps(record) + '\n')
            attempts.flush()
            if record['status'] != 'ok':
                raise SystemExit(f"attempt {index} {record['status']}; retained in {out}")
    print(json.dumps({'processes': len(schedule), 'oracle': 'all pass'}))

if __name__ == '__main__':
    main()
