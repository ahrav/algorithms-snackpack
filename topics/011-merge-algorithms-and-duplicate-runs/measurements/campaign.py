#!/usr/bin/env python3
"""Predeclared paired 12-block campaign, separate process per sample."""
import argparse, hashlib, json, os, platform, random, subprocess, time
from pathlib import Path

p=argparse.ArgumentParser(); p.add_argument('--out',type=Path,default=Path('results')); args=p.parse_args(); out=args.out; out.mkdir(exist_ok=False)
def run(cmd):
    p=subprocess.run(cmd,text=True,capture_output=True)
    assert p.returncode==0,(cmd,p.stderr)
    return p.stdout
assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest()==h for p,h in json.loads(Path('source-manifest.json').read_text()).items())
cpus=sorted(os.sched_getaffinity(0))[:4]; assert len(cpus)==4
env=dict(hostname=platform.node(),architecture=platform.machine(),uname=run(['uname','-a']),lscpu=run(['lscpu']),rustc=run(['rustc','-vV']),target_cfg=run(['rustc','--print','cfg']),available_cpus=sorted(os.sched_getaffinity(0)),cpuset=cpus,flags=['--edition=2024','-C','opt-level=3'],boundary='candidate entry through output destruction; allocation, output writes, initialization, co-ranking, thread launch/join included; input generation/sorting/oracle excluded; process wall retained separately',warmup='3 calls except cold cells 0; cold means first timed candidate after untimed correctness check, not cold cache/OS/allocator',blocks=12,assignment='6 shuffled candidate orders then their reversals; every pair balanced 6/6; same fixed inputs across blocks',winner='lowest paired geometric mean with all simultaneous 95 percent lower rival/fastest bounds >1.05; otherwise unresolved',family=152,stopping='exactly 12 blocks; no winner-driven reruns',record_bytes=16,limitations=['No A/A calibration','No hardware counters','Hosts/toolchains differ','Shared-host cpuset, no isolated CPUs or NUMA allocation control','Fixed seeded inputs; inference conditional on those inputs'])
(out/'environment.json').write_text(json.dumps(env,indent=2)); (out/'input-source-manifest.json').write_bytes(Path('source-manifest.json').read_bytes())
run(['rustc','--edition=2024','-C','opt-level=3','--crate-name','merge_lab','--crate-type','rlib','topic/src/lib.rs','-o','libmerge_lab.rlib'])
run(['rustc','--edition=2024','-C','opt-level=3','--test','topic/src/lib.rs','-o','contracts'])
run(['rustc','--edition=2024','-C','opt-level=3','topic/benches/portfolio.rs','--extern','merge_lab=libmerge_lab.rlib','-o','experiment'])
run(['rustc','--edition=2024','-C','opt-level=3','--test','topic/tests/contracts.rs','--extern','merge_lab=libmerge_lab.rlib','-o','integration'])
(out/'contracts.stdout').write_text(run(['./contracts'])+run(['./integration']))
(out/'example.stdout').write_text(run(['./experiment','join_equal1024','replay','1']))
(out/'binary.sha256').write_text(hashlib.sha256(Path('experiment').read_bytes()).hexdigest())
assembly=subprocess.run(['objdump','-Cd','experiment'],text=True,capture_output=True)
(out/'linked-assembly.txt').write_text(assembly.stdout); (out/'assembly-status.txt').write_text(str(assembly.returncode))
cells=[('merge_cold32',1),('merge_warm32',4096),('merge_random4096',64),('merge_interleaved1m',2),('merge_random1m',2),('merge_disjoint1m',2),('merge_equal1m',2),('merge_skew1m',2),('join_cold64',1),('join_sparse8192',64),('join_duplicates8192',16),('join_equal1024',8),('cascade_tiny',256),('cascade_4',16),('cascade_32',2),('cascade_skew',4)]
count=0
with (out/'attempts.jsonl').open('w') as f:
    for cell,reps in cells:
        methods=['sort','linear','gallop','parallel4'] if cell.startswith('merge') else ['probe','replay'] if cell.startswith('join') else ['sort','left','balanced','heap']
        orders=[]
        for b in range(6):
            order=methods.copy(); random.Random(1100+b).shuffle(order); orders.append(order)
        orders += [list(reversed(x)) for x in orders]
        for block,order in enumerate(orders):
            for position,method in enumerate(order):
                cmd=['taskset','-c',','.join(map(str,cpus)),'./experiment',cell,method,str(reps)]
                start=time.perf_counter_ns(); stdout=run(cmd); wall=time.perf_counter_ns()-start
                rec=json.loads(stdout); assert rec['ns']>0 and rec['iterations']==reps
                rec.update(cell=cell,method=method,block=block,position=position,ns_per_call=rec['ns']/reps,wall_ns=wall,args=cmd)
                f.write(json.dumps(rec)+'\n'); f.flush(); count+=1
        print(cell,'COMPLETE',flush=True)
print('CONTRACTS_AND_CAMPAIGN_OK',count)
