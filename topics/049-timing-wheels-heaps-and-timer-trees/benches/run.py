#!/usr/bin/env python3
"""Fixed six-block schedule; independent full-output oracle for each process."""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import subprocess

CANDIDATES = ['scan', 'heap', 'tree', 'wheel']

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
    records = []
    for index, spec in enumerate(schedule):
        cmd = ['taskset', '-c', '0', binary, spec['candidate'], str(spec['n']), spec['shape'], str(spec['cancel'])]
        process = subprocess.run(cmd, text=True, capture_output=True, timeout=60)
        (out / f'{index:04d}.stdout').write_text(process.stdout)
        (out / f'{index:04d}.stderr').write_text(process.stderr)
        record = dict(spec, index=index, command=cmd, exit=process.returncode)
        records.append(record)
        (out / 'attempts.json').write_text(json.dumps(records, indent=2) + '\n')
        assert process.returncode == 0, record
        data = json.loads(process.stdout)
        want_fired, want_live = expected(spec['n'], spec['shape'], spec['cancel'])
        assert data['fired'] == want_fired and data['live'] == want_live, record
        assert data['ns'] > 0, record
        record.update(ns=data['ns'], inspections=data['inspections'], ticks=data['ticks'], peak_records=data['peak_records'], oracle='pass')
        (out / 'attempts.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps(dict(processes=len(records), oracle='all pass')))

if __name__ == '__main__':
    main()
