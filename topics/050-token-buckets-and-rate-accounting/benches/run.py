#!/usr/bin/env python3
"""Exact-source fresh-process screen with an independent accepted-cost ledger."""
import argparse
import hashlib
import json
import os
import platform
import re
import statistics
import subprocess
import time
from pathlib import Path

if not __debug__:
    raise SystemExit('run.py needs assertions enabled: run it without -O and with PYTHONOPTIMIZE unset')

INPUTS = ['src/lib.rs', 'tests/contracts.rs', 'examples/running.rs', 'benches/replay.rs', 'benches/run.py']
MODES = ['wide', 'bounded', 'virtual']
MASK = (1 << 64) - 1

def events(n, flows, pattern):
    for i in range(n):
        rd = i // flows
        tick = rd // 1024 if pattern == 'burst' else rd * 100 if pattern == 'idle' else rd // 2 if pattern == 'weighted' else rd
        cost = [0, 1, 6, 11, 4, 10, 3][i % 7] if pattern == 'weighted' else 6
        yield i % flows, tick, cost

def oracle(n, flows, pattern):
    # Accepted-cost prefix total and maximum suffix-envelope intercept.
    total, anchor = [0]*flows, [0]*flows
    history = [[] for _ in range(flows)]
    output, digest = [], 0
    for i, (f, tick, cost) in enumerate(events(n, flows, pattern)):
        available = min(10, 10 + 3*tick - total[f] - anchor[f])
        if i < 128:
            suffix, slow = 0, 10
            for t, c in reversed(history[f]):
                suffix += c
                slow = min(slow, 10+3*(tick-t)-suffix)
            assert slow == available
        ok = cost <= available
        if ok:
            anchor[f] = max(anchor[f], 3*tick-total[f])
            total[f] += cost
            if i < 128: history[f].append((tick,cost))
        credit = available-cost if ok else available
        output.append([ok, credit])
        digest = (digest*1_000_003 + credit*2+int(ok)) & MASK
    return digest, output

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def command(args):
    return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT)

def committed(source, name):
    """Bytes of one input at the source commit, or None when this repository lacks the commit or the path."""
    blob = subprocess.run(['git', 'cat-file', 'blob', f'{source}^{{commit}}:./{name}'], capture_output=True)
    return blob.stdout if blob.returncode == 0 else None

def pinned():
    """The path-to-sha256 object in INPUTS.json, which a frozen-source archive carries; None in a plain checkout."""
    try:
        return json.loads(Path('INPUTS.json').read_text())
    except FileNotFoundError:
        return None
    except ValueError as error:
        raise SystemExit(f'INPUTS.json is not JSON: {error}') from error

def tracked():
    """True when Git tracks every input in this directory, as in a checkout."""
    try:
        return subprocess.run(['git', 'ls-files', '--error-unmatch', '--', *INPUTS], capture_output=True).returncode == 0
    except OSError:
        return False

def main():
    p=argparse.ArgumentParser(); p.add_argument('--source',required=True); a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{40}', a.source):
        p.error('--source takes a 40-digit lowercase hexadecimal commit id')
    os.chdir(Path(__file__).resolve().parents[1])
    inputs={name:sha(name) for name in INPUTS}
    if pinned() not in (None, inputs):
        raise SystemExit('INPUTS.json differs from the sha256 of '+', '.join(INPUTS))
    check='git' if tracked() else 'label'
    for name in INPUTS if check=='git' else []:
        if committed(a.source,name)!=Path(name).read_bytes():
            raise SystemExit(f'{name} differs from commit {a.source}, or this repository lacks that commit')
    Path('raw').mkdir()
    cpu=min(os.sched_getaffinity(0))
    env={'source':a.source,'source_check':check,'hostname':platform.node(),'uname':command(['uname','-a']), 'cpu':command(['lscpu']), 'rustc':command(['rustc','-Vv']), 'affinity':sorted(os.sched_getaffinity(0)), 'available_cpus':len(os.sched_getaffinity(0)), 'pinned_cpu':cpu, 'flags':['--edition=2024','-O','-C','target-cpu=native'], 'start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'inputs':inputs}
    Path('raw/environment.json').write_text(json.dumps(env,indent=2))
    flags=['--edition=2024','-O','-C','target-cpu=native']
    command(['rustc',*flags,'--crate-name','topic050_rate_accounting','--crate-type','rlib','src/lib.rs','-o','raw/libtopic050_rate_accounting.rlib'])
    command(['rustc',*flags,'--test','tests/contracts.rs','--extern','topic050_rate_accounting=raw/libtopic050_rate_accounting.rlib','-o','raw/contracts'])
    Path('raw/contracts.txt').write_text(command(['raw/contracts']))
    command(['rustc',*flags,'benches/replay.rs','-o','raw/replay'])
    command(['rustc',*flags,'--emit=asm','benches/replay.rs','-o','raw/replay.s'])
    records=[]
    def invoke(mode,n,flows,pattern,reps,phase,block,details=False):
        digest, want = oracle_cache[(n,flows,pattern)]
        args=['taskset','-c',str(cpu),'raw/replay',mode,str(n),str(flows),pattern,str(reps)]
        if details: args.append('details')
        r=json.loads(command(args)); assert r['digest']==digest,(mode,n,flows,pattern)
        if details: assert r['output']==want, (mode,n,flows,pattern,'full output')
        r.update(mode=mode,n=n,flows=flows,pattern=pattern,phase=phase,block=block)
        records.append(r)
        Path('raw/records.json').write_text(json.dumps(records))
    oracle_cache={}
    for n in [32,32768]:
      for flows in [1,4096]:
       for pattern in ['burst','steady','idle','weighted']:
        oracle_cache[(n,flows,pattern)]=oracle(n,flows,pattern)
        reps=256 if n==32 else 16
        for mode in MODES: invoke(mode,n,flows,pattern,1,'correctness',-1,True)
        for mode in MODES: invoke(mode,n,flows,pattern,reps,'warmup',-1)
        for block in range(6):
            order=MODES[block%3:]+MODES[:block%3]
            if block>=3: order=order[::-1]
            for mode in order: invoke(mode,n,flows,pattern,reps,'measured',block)
    for block in range(6):
      for side in ['a','b']: invoke('wide',32768,1,'weighted',16,'aa-'+side,block)
    summary=[]
    for key in oracle_cache:
        n,flows,pattern=key
        group=[r for r in records if r['phase']=='measured' and (r['n'],r['flows'],r['pattern'])==key]
        values={m:[r['ns']/r['reps'] for r in group if r['mode']==m] for m in MODES}
        winners=[m for m in MODES if all(all(x<.95*y for x,y in zip(values[m],values[o])) for o in MODES if o!=m)]
        summary.append({'n':n,'flows':flows,'pattern':pattern,'winner':winners[0] if winners else 'unresolved','ns':{m:{'median':statistics.median(v),'min':min(v),'max':max(v)} for m,v in values.items()}})
    aa=[next(r['ns'] for r in records if r['phase']=='aa-a' and r['block']==b)/next(r['ns'] for r in records if r['phase']=='aa-b' and r['block']==b) for b in range(6)]
    Path('raw/summary.json').write_text(json.dumps({'cells':summary,'aa_ratios':aa,'processes':len(records),'binary_sha256':sha('raw/replay'),'verified':True},indent=2))
    Path('raw/MANIFEST.json').write_text(json.dumps({str(p):sha(p) for p in Path('raw').iterdir() if p.is_file()},indent=2))
    print(json.dumps({'processes':len(records),'aa':aa,'winners':{m:sum(c['winner']==m for c in summary) for m in MODES+['unresolved']}}))
if __name__=='__main__': main()
