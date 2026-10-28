#!/usr/bin/env python3
"""Build frozen inputs and retain every fresh-process attempt on one Linux host."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
CRATE = 'topic_032_topological_order_and_incremental_dags'
INPUTS = ['Cargo.toml', 'src/lib.rs', 'tests/contracts.rs', 'benches/dag.rs', 'examples/demo.rs', 'scripts/run.py']
ORDER = ['tiny', 'chain_root', 'islands', 'dense_root']
UPDATE = ['tiny', 'chain_root', 'chain_leaf', 'islands', 'layered_sink', 'dense_root', 'noop']

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def command(args, **kwargs):
    return subprocess.run(args, text=True, capture_output=True, check=True, **kwargs)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--blocks', type=int, default=6)
    parser.add_argument('--source-commit', required=True)
    args = parser.parse_args()
    if args.blocks != 6:
        parser.error('The declared campaign requires exactly six blocks.')
    args.out.mkdir(parents=True, exist_ok=False)
    build = args.out / 'build'
    build.mkdir()
    hashes = {p: digest(ROOT / p) for p in INPUTS}
    flags = ['--edition=2024', '-C', 'opt-level=3', '-C', 'target-cpu=native', '-C', 'codegen-units=1']
    rlib = build / ('lib' + CRATE + '.rlib')
    command(['rustc', *flags, '--crate-type=rlib', '--crate-name', CRATE, str(ROOT/'src/lib.rs'), '-o', str(rlib)])
    extern = ['--extern', CRATE + '=' + str(rlib)]
    for kind, source, extra in [('bench','benches/dag.rs',[]),('tests','tests/contracts.rs',['--test']),('demo','examples/demo.rs',[])]:
        command(['rustc', *flags, *extra, str(ROOT/source), *extern, '-o', str(build/kind)])
    tests = command([str(build/'tests')])
    demo = command([str(build/'demo')])
    (args.out/'tests.txt').write_text(tests.stdout+tests.stderr)
    (args.out/'demo.txt').write_text(demo.stdout+demo.stderr)
    environment = {
        'hostname': platform.node(), 'architecture': platform.machine(),
        'uname': list(platform.uname()), 'cpu_count': os.cpu_count(),
        'affinity': sorted(os.sched_getaffinity(0)),
        'cpuinfo': Path('/proc/cpuinfo').read_text(),
        'lscpu': command(['lscpu']).stdout,
        'rustc': command(['rustc','-vV']).stdout,
        'target_cfg': command(['rustc','--print','cfg','-C','target-cpu=native']).stdout,
        'flags': flags, 'source_commit': args.source_commit,
        'input_sha256': hashes, 'binary_sha256': digest(build/'bench'),
        'started_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
        'boundary': 'order allocation/output/drop; update preinitialized persistent buffers, edits+work counters+black_box, excludes final validation/destruction',
    }
    (args.out/'environment.json').write_text(json.dumps(environment, indent=2)+'\n')
    records=[]
    path=args.out/'attempts.jsonl'
    def attempt(family, candidate, workload, block, position, warmup=False):
        started=time.time_ns()
        result=subprocess.run([str(build/'bench'),family,candidate,workload], text=True,capture_output=True)
        record={'family':family,'candidate':candidate,'workload':workload,'block':block,'position':position,'warmup':warmup,'returncode':result.returncode,'started_unix_ns':started,'process_ns':time.time_ns()-started,'stderr':result.stderr,'stdout':result.stdout}
        if result.returncode == 0:
            record.update(json.loads(result.stdout))
            assert record['oracle'] is True
            record['ns_per_operation']=record['ns']/record['operations']
        with path.open('a') as stream: stream.write(json.dumps(record,sort_keys=True)+'\n')
        records.append(record)
        if result.returncode != 0: raise RuntimeError(record)
    for family,workloads,candidates in [('order',ORDER,['fifo','dfs','min']),('update',UPDATE,['full','dirty','heap'])]:
        for name in workloads:
            for c in candidates: attempt(family,c,name,-1,0,True)
            for block in range(6):
                ordering = candidates if block%2==0 else list(reversed(candidates))
                for position,c in enumerate(ordering): attempt(family,c,name,block,position)
            print(f'{family}/{name} complete',flush=True)
    for family,candidates in [('order',['fifo','fifo_aa']),('update',['full','full_aa'])]:
        for c in candidates: attempt(family,c,'tiny',-1,0,True)
        for block in range(6):
            ordering=candidates if block%2==0 else list(reversed(candidates))
            for position,c in enumerate(ordering): attempt(family,c,'tiny',block,position)
    assert hashes=={p:digest(ROOT/p) for p in INPUTS}
    assert len(records)==259
    (args.out/'complete.json').write_text(json.dumps({'records':len(records),'measured':222,'warmups':37,'input_sha256':hashes},indent=2)+'\n')

if __name__ == '__main__':
    main()
