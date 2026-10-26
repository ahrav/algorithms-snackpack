#!/usr/bin/env python3
"""Frozen traversal campaign. Each recorded row is a fresh process."""
import argparse, hashlib, json, os, pathlib, platform, subprocess, time
ROOT=pathlib.Path(__file__).resolve().parent
CASES=['tiny','path_small','star_small','path','star','grid','permuted_grid','sparse','dense','disconnected']
CANDIDATES=['deque','vector','csr','layers','frames','events']
FLAGS=['--edition=2024','-C','opt-level=3','-C','codegen-units=1','-C','target-cpu=native']
def command(args):
    return subprocess.check_output(args,stderr=subprocess.STDOUT,text=True)
def sample(binary, case, candidate):
    start=time.perf_counter_ns()
    row=json.loads(command([str(binary),case,candidate]))
    row['process_wall_ns']=time.perf_counter_ns()-start
    return row
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    ap=argparse.ArgumentParser();ap.add_argument('--out',required=True);ap.add_argument('--focused',action='store_true');args=ap.parse_args()
    out=pathlib.Path(args.out);out.mkdir(parents=True,exist_ok=False)
    env={'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'compiler':command(['rustc','-vV']),'target_cfg':command(['rustc','--print','cfg','-C','target-cpu=native']),'flags':FLAGS,'available_cpus':sorted(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),'lscpu':command(['lscpu']) if platform.system()=='Linux' else 'unavailable','sources':{p:sha(ROOT/p) for p in ['src/lib.rs','tests/contracts.rs','benches/traversal.rs','run.py']},'started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),'mode':'focused' if args.focused else 'committed'}
    (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
    command(['rustc',*FLAGS,'--test',str(ROOT/'tests/contracts.rs'),'-o',str(out/'contracts')])
    (out/'tests.txt').write_text(command([str(out/'contracts')]))
    command(['rustc',*FLAGS,str(ROOT/'benches/traversal.rs'),'-o',str(out/'traversal')])
    (out/'binary.sha256').write_text(sha(out/'traversal')+'\n')
    if not args.focused:
        (out/'symbols.txt').write_text(command(['nm','-C',str(out/'traversal')]))
        (out/'disassembly.txt').write_text(command(['objdump','-d','-C',str(out/'traversal')]))
    cases=['tiny'] if args.focused else CASES
    with (out/'samples.jsonl').open('w') as stream:
        for case in cases:
            for phase,blocks in [('warmup',1),('measure',2 if args.focused else 6)]:
                for block in range(blocks):
                    order=CANDIDATES if block%2==0 else CANDIDATES[::-1]
                    for position,candidate in enumerate(order):
                        row=sample(out/'traversal',case,candidate);row.update(phase=phase,block=block,position=position)
                        stream.write(json.dumps(row)+'\n');stream.flush()
    if not args.focused:
        with (out/'samples.jsonl').open('a') as stream:
            for phase,blocks in [('aa_warmup',1),('aa_measure',6)]:
                for block in range(blocks):
                    order=['deque_a','deque_b'] if block%2==0 else ['deque_b','deque_a']
                    for position,candidate in enumerate(order):
                        row=sample(out/'traversal','path',candidate);row.update(phase=phase,block=block,position=position)
                        stream.write(json.dumps(row)+'\n');stream.flush()
    rows=[json.loads(x) for x in (out/'samples.jsonl').read_text().splitlines()]
    for case in cases:
        observed=[r for r in rows if r['case']==case and not r['phase'].startswith('aa_')]
        assert len({(r['n'],r['edges'],r['reached'],r['signature']) for r in observed})==1
        for a in CANDIDATES:
            for b in CANDIDATES:
                if a==b:continue
                measured=[r for r in observed if r['phase']=='measure']
                before=sum(next(r['position'] for r in measured if r['block']==i and r['candidate']==a)<next(r['position'] for r in measured if r['block']==i and r['candidate']==b) for i in range(2 if args.focused else 6))
                assert before==(1 if args.focused else 3)
    manifest={p.name:sha(p) for p in out.iterdir() if p.is_file()}
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'out':str(out),'host':env['hostname'],'architecture':env['architecture'],'rows':len(rows),'source_sha256':env['sources'],'verified_order':True}))
if __name__=='__main__':main()
