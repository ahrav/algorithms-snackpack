#!/usr/bin/env python3
"""Frozen native runner. Each observation is a fresh balanced process."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--preflight', action='store_true')
args = p.parse_args()
root = Path(__file__).resolve().parents[1]
os.chdir(root)
out = root / ('preflight' if args.preflight else 'native')
out.mkdir(exist_ok=True)

def command(argv):
    return subprocess.check_output(argv, text=True, stderr=subprocess.STDOUT)

inputs = ['src/lib.rs', 'tests/contracts.rs', 'examples/edit.rs', 'benches/lookup.rs', 'measurements/run.py']
hashes = {f: hashlib.sha256((root/f).read_bytes()).hexdigest() for f in inputs}
flags = ['--edition=2024', '-O', '-C', 'target-cpu=native']
identity = dict(host=platform.node(), architecture=platform.machine(), uname=command(['uname','-a']),
                cpu=command(['lscpu']), compiler=command(['rustc','-vV']),
                features=command(['rustc','--print','cfg','-C','target-cpu=native']),
                affinity=sorted(os.sched_getaffinity(0)), flags=flags, source_sha256=hashes)
cpu = min(os.sched_getaffinity(0))
(out/'identity.json').write_text(json.dumps(identity, indent=2)+'\n')
command(['rustc', *flags, '--crate-name','topic_041_edit','--crate-type','rlib','src/lib.rs','-o',str(out/'libtopic_041_edit.rlib')])
external = ['--extern', 'topic_041_edit='+str(out/'libtopic_041_edit.rlib')]
command(['rustc', *flags, *external, '--test','tests/contracts.rs','-o',str(out/'contracts')])
(out/'contracts.log').write_text(command([str(out/'contracts')]))
command(['rustc', *flags, *external,'examples/edit.rs','-o',str(out/'example')])
(out/'example.log').write_text(command([str(out/'example')]))
command(['rustc', *flags, *external,'benches/lookup.rs','-o',str(out/'lookup')])
identity['binary_sha256'] = hashlib.sha256((out/'lookup').read_bytes()).hexdigest()
(out/'identity.json').write_text(json.dumps(identity, indent=2)+'\n')
(out/'code.s').write_text(command(['objdump','-d',str(out/'lookup')]))
cells = [(n,m,s,3 if s=='dense' else 1) for n in [32,4096] for m in [8,48] for s in ['random','prefix','dense']]
cells += [(4096,48,'random',4),(4096,48,'prefix',4),(4096,48,'dense',0),(32,64,'random',1),(32,65,'random',1)]
if args.preflight:
    cells = [(32,8,'random',1),(32,8,'dense',1)]
records = []
methods = ['row','band','myers','trie']
schedule = []
with (out/'samples.jsonl').open('w') as log:
    for cell in cells:
        n,m,shape,k = cell
        repeats = 1 if args.preflight else (64 if n==32 else 3)
        rounds = [-1,0] if args.preflight else [-1,*range(8)]
        for block in rounds:
            order = methods if block<0 else [methods[(block+j)%4] for j in range(4)]
            if block>=4: order = list(reversed(order))
            for method in order:
                argv = ['taskset','-c',str(cpu),str(out/'lookup'),method,str(n),str(m),shape,str(k),str(repeats)]
                started = time.perf_counter_ns()
                record = json.loads(command(argv))
                record["whole_process_ns"] = time.perf_counter_ns() - started
                record.update(block=block, cpu=cpu, ns_per_query=record['elapsed_ns']/repeats)
                schedule.append(dict(cell=cell, block=block, method=method))
                assert record['oracle']
                records.append(record)
                log.write(json.dumps(record)+'\n'); log.flush()
summary=[]
for cell in cells:
    n,m,shape,k=cell
    selected=[r for r in records if (r['n'],r['m'],r['shape'],r['k'])==cell and r['block']>=0]
    medians={method:statistics.median([r['ns_per_query'] for r in selected if r['method']==method]) for method in methods}
    best=min(medians,key=medians.get)
    clear=all(all(next(r['ns_per_query'] for r in selected if r['method']==best and r['block']==b) <
                       .95*next(r['ns_per_query'] for r in selected if r['method']==other and r['block']==b)
                       for b in range(1 if args.preflight else 8)) for other in methods if other!=best)
    summary.append(dict(cell=cell, median_ns=medians, fastest_median=best, selection=best if clear else 'unresolved',
                        range_ns={a:[min(r['ns_per_query'] for r in selected if r['method']==a),max(r['ns_per_query'] for r in selected if r['method']==a)] for a in methods},
                        trie_build_median_ns=statistics.median([r['build_ns'] for r in selected if r['method']=='trie']),
                        hits=selected[0]['hits'],nodes=next(r['nodes'] for r in selected if r['method']=='trie'),
                        edges=next(r['edges'] for r in selected if r['method']=='trie')))
(out/'schedule.json').write_text(json.dumps(schedule,indent=2)+'\n')
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(dict(host=identity['host'],architecture=identity['architecture'],records=len(records),cells=len(cells),oracles=True)))

if not args.preflight:
    aa=[]
    for block in range(8):
        for arm in (['A','B'] if block%2==0 else ['B','A']):
            r=json.loads(command(['taskset','-c',str(cpu),str(out/'lookup'),'row','32','48','random','1','64']))
            r.update(block=block,arm=arm)
            aa.append(r)
    (out/'aa.json').write_text(json.dumps(aa,indent=2)+'\n')
