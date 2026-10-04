#!/usr/bin/env python3
"""Frozen process-level Top-K campaign; standard library only."""
import argparse
import hashlib
import itertools
import json
import math
import os
from pathlib import Path
import platform
import shutil
import statistics
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
NAMES = ['sort', 'standard', 'three_way', 'heap', 'buffer']
# name, length, K, shape, boundary, calls per process
WORKLOADS = [
    ('tiny', 32, 4, 'random', 'warm', 2048),
    ('small_k8', 4096, 8, 'random', 'warm', 32),
    ('large_k8', 65536, 8, 'random', 'warm', 4),
    ('ascending', 65536, 8, 'ascending', 'warm', 4),
    ('descending', 65536, 8, 'descending', 'warm', 4),
    ('duplicates', 65536, 8, 'duplicates', 'warm', 4),
    ('equal', 65536, 8, 'equal', 'warm', 4),
    ('organ', 65536, 8, 'organ', 'warm', 4),
    ('half', 4096, 2048, 'random', 'warm', 8),
    ('all', 4096, 4096, 'random', 'warm', 8),
    ('first', 65536, 8, 'random', 'first', 1),
]
BLOCKS = 20
FAMILY = len(WORKLOADS) * math.comb(len(NAMES), 2) + 1


def execute(args, out, name):
    result = subprocess.run(args, capture_output=True, text=True, check=False)
    (out / (name + '.stdout')).write_text(result.stdout)
    (out / (name + '.stderr')).write_text(result.stderr)
    if result.returncode:
        raise RuntimeError(f'{name}: exit {result.returncode}: {result.stderr}')
    return result.stdout


def hash_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def t_critical(df, tail):
    """Invert the Student-t CDF using composite Simpson quadrature."""
    c = math.exp(math.lgamma((df + 1) / 2) - math.lgamma(df / 2)) / math.sqrt(df * math.pi)
    def integral(x):
        steps = 4000
        h = x / steps
        def density(y):
            return c * (1 + y * y / df) ** (-(df + 1) / 2)
        return h / 3 * (density(0) + density(x) + sum(
            (4 if i % 2 else 2) * density(i * h) for i in range(1, steps)))
    lo, hi = 0., 32.
    for _ in range(50):
        mid = (lo + hi) / 2
        if .5 - integral(mid) > tail:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def interval(log_ratios, critical):
    mean = statistics.mean(log_ratios)
    half = critical * statistics.stdev(log_ratios) / math.sqrt(len(log_ratios))
    return {'ratio': math.exp(mean), 'low': math.exp(mean-half), 'high': math.exp(mean+half)}


def summarize(records):
    assert len(records) == BLOCKS * (len(WORKLOADS) * len(NAMES) + 2)
    assert abs(t_critical(11, .025) - 2.200985) < 0.00001
    critical = t_critical(BLOCKS-1, .05/(2*FAMILY))
    results = []
    for workload, *_ in WORKLOADS:
        cell = [r for r in records if r['workload'] == workload]
        by_name = {name: sorted([r for r in cell if r['candidate'] == name], key=lambda r: r['block']) for name in NAMES}
        for rows in by_name.values():
            assert [r['block'] for r in rows] == list(range(BLOCKS))
        timings = {name: math.exp(statistics.mean(math.log(r['ns_per_call']) for r in rows)) for name, rows in by_name.items()}
        fastest = min(NAMES, key=lambda n: timings[n])
        contrasts = {}
        for left, right in itertools.combinations(NAMES, 2):
            contrasts[left+'/'+right] = interval([math.log(a['ns_per_call']/b['ns_per_call']) for a, b in zip(by_name[left], by_name[right])], critical)
        contenders = [fastest]
        fastest_contrasts = {}
        for other in NAMES:
            if other == fastest:
                continue
            ci = interval([math.log(a['ns_per_call']/b['ns_per_call']) for a, b in zip(by_name[other], by_name[fastest])], critical)
            fastest_contrasts[other+'/'+fastest] = ci
            if ci['low'] <= 1.05:
                contenders.append(other)
        results.append(dict(workload=workload,geomean_ns=timings,point_fastest=fastest,
                            selection=fastest if len(contenders)==1 else 'unresolved',
                            unresolved_set=contenders,contrasts=contrasts,
                            fastest_contrasts=fastest_contrasts,
                            min_max_ns={name:[min(r['ns_per_call'] for r in rows),max(r['ns_per_call'] for r in rows)] for name,rows in by_name.items()}))
    aa = sorted([r for r in records if r['workload']=='aa'],key=lambda r:(r['block'],r['label']))
    aa_a = [r for r in aa if r['label']=='aa_a']
    aa_b = [r for r in aa if r['label']=='aa_b']
    aa_ci = interval([math.log(b['ns_per_call']/a['ns_per_call']) for a,b in zip(aa_a,aa_b)],critical)
    return dict(family=FAMILY,alpha=.05,critical_t=critical,interval_method='paired log t; Bonferroni two-sided familywise 95%; normal log-ratio model',
                aa=aa_ci,aa_mechanical='pass; same linked binary/candidate, complete blocks, equal checksums',results=results)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--analyze-only', action='store_true')
    parser.add_argument('--build-only', action='store_true')
    opts = parser.parse_args()
    out = opts.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    if opts.analyze_only:
        rows = [json.loads(line) for line in (out/'attempts.jsonl').read_text().splitlines()]
        summary = summarize(rows)
        (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
        print(json.dumps(summary))
        return
    if (out/'attempts.jsonl').exists():
        raise RuntimeError('refuse to overwrite an existing campaign')
    flags = ['--edition','2024','-C','opt-level=3','-C','debuginfo=1']
    lib = out/'libtop_k_portfolio.rlib'
    execute(['rustc',*flags,'--crate-name','top_k_portfolio','--crate-type','rlib',str(ROOT/'src/lib.rs'),'-o',str(lib)],out,'build-library')
    for source, name, extra in [('tests/contracts.rs','contracts',['--test']),('examples/top_k.rs','example',[]),('benches/top_k.rs','top_k',[])]:
        execute(['rustc',*flags,*extra,str(ROOT/source),'--extern',f'top_k_portfolio={lib}','-o',str(out/name)],out,'build-'+name)
    execute([str(out/'contracts')],out,'contracts')
    execute([str(out/'example')],out,'example')
    cfg = execute(['rustc','--print','cfg'],out,'target-cfg')
    version = execute(['rustc','-Vv'],out,'rustc-version')
    env = dict(hostname=platform.node(),architecture=platform.machine(),uname=list(platform.uname()),
               available_cpus=len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),
               affinity='none on macOS; first allowed CPU on Linux',flags=flags,target_cfg=cfg,compiler=version,
               start_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
               files={str(p.relative_to(ROOT)):hash_file(p) for p in sorted(ROOT.rglob('*')) if p.is_file() and (p.suffix=='.rs' or p.name in ['run.py','Cargo.toml','BENCHMARK.md'])},
               binary_sha256=hash_file(out/'top_k'),blocks=BLOCKS,family=FAMILY,
               seed='0xa11c00086a30',workloads=WORKLOADS)
    if platform.system()=='Darwin':
        env['cpu_model']=execute(['sysctl','-n','machdep.cpu.brand_string'],out,'cpu-model').strip()
        execute(['otool','-tvV',str(out/'top_k')],out,'linked-disassembly')
    else:
        env['cpu_model']=execute(['lscpu'],out,'cpu-model')
        execute(['objdump','-d',str(out/'top_k')],out,'linked-disassembly')
    (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
    if opts.build_only:
        return
    prefix = []
    if platform.system()=='Linux' and shutil.which('taskset'):
        cpu = min(os.sched_getaffinity(0))
        prefix = ['taskset','-c',str(cpu)]
    rows=[]
    ledger=out/'attempts.jsonl'
    for block in range(BLOCKS):
        rotation=(block//2)%len(NAMES)
        order=NAMES[rotation:]+NAMES[:rotation]
        if block%2:
            order=list(reversed(order))
        for workload,n,k,shape,mode,repeats in WORKLOADS:
            checksums=[]
            for position,name in enumerate(order):
                attempt=f'{block:02d}-{workload}-{name}'
                begin=time.monotonic_ns()
                output=execute([*prefix,str(out/'top_k'),name,str(n),str(k),shape,mode,str(repeats)],out,attempt)
                row=json.loads(output)
                assert row['candidate']==name and row['n']==n and row['shape']==shape and row['boundary']==mode and row['k']==k
                assert row['elapsed_ns']>0 and row['calls']==repeats
                row.update(workload=workload,block=block,position=position,label=name,wall_ns=time.monotonic_ns()-begin)
                rows.append(row);checksums.append(row['checksum'])
                with ledger.open('a') as f: f.write(json.dumps(row)+'\n')
            assert len(set(checksums))==1
        aa_order=['aa_a','aa_b'] if block%2==0 else ['aa_b','aa_a']
        for position,label in enumerate(aa_order):
            output=execute([*prefix,str(out/'top_k'),'standard','4096','8','random','warm','32'],out,f'{block:02d}-{label}')
            row=json.loads(output)
            row.update(workload='aa',block=block,position=position,label=label)
            assert row['elapsed_ns']>0
            rows.append(row)
            with ledger.open('a') as f: f.write(json.dumps(row)+'\n')
        assert rows[-1]['checksum']==rows[-2]['checksum']
        print(f'complete block {block+1}/{BLOCKS}',flush=True)
    summary=summarize(rows)
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps({r['workload']:{'point_fastest':r['point_fastest'],'selection':r['selection'],'unresolved_set':r['unresolved_set']} for r in summary['results']}))


if __name__=='__main__':
    main()
