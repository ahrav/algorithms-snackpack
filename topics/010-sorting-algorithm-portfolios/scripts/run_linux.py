#!/usr/bin/env python3
"""Frozen 12-block process schedule. No third-party dependencies."""
import argparse,hashlib,json,os,platform,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--topic',default='topic',type=Path);p.add_argument('--out',default='results',type=Path);args=p.parse_args();source=args.topic;out=args.out;out.mkdir(exist_ok=False)
def run(args):
 p=subprocess.run(args,text=True,capture_output=True);assert p.returncode==0,(args,p.stderr);return p.stdout
if Path('source-manifest.json').exists():
 assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest()==h for p,h in json.loads(Path('source-manifest.json').read_text()).items())
(out/'input-source-manifest.json').write_text(json.dumps({str(f):hashlib.sha256(f.read_bytes()).hexdigest() for f in sorted(source.rglob('*.rs'))},indent=2))
env=dict(hostname=platform.node(),architecture=platform.machine(),uname=run(['uname','-a']),cpuinfo=Path('/proc/cpuinfo').read_text(),rustc=run(['rustc','-vV']),target_cfg=run(['rustc','--print','cfg']),available_cpus=sorted(os.sched_getaffinity(0)),flags=['--edition=2024','-C','opt-level=3'],boundary='sort entry to return, including algorithm scratch allocation/destruction; input cloning separately timed; generation/oracle/checksum/input teardown excluded; process wall separately retained',warmup='3 calls; cold means first candidate entry, not cold input/cache/allocator/OS',blocks=12,assignment='cyclic rotation with alternate reversal; common seed within block; each candidate separate process',record_size='size_of Record measured in example receipt; key u64 plus id u32; key-only comparator',winner='geometric mean paired log time; unique only if simultaneous lower other/fastest >1.05 for every same-contract rival',family=620,stopping='exactly12 blocks, no winner-driven reruns')
(out/'environment.json').write_text(json.dumps(env,indent=2));cpu=env['available_cpus'][0]
run(['rustc','--edition=2024','-C','opt-level=3','--crate-name','sorting_lab','--crate-type','rlib',str(source/'src/lib.rs'),'-o','libsorting_lab.rlib'])
for name,path,test in [('contracts',str(source/'tests/contracts.rs'),True),('portfolio',str(source/'benches/portfolio.rs'),False),('example',str(source/'examples/portfolio.rs'),False)]:
 args=['rustc','--edition=2024','-C','opt-level=3',path,'--extern','sorting_lab=libsorting_lab.rlib','-o',name]
 if test:args.append('--test')
 run(args)
(out/'contracts.stdout').write_text(run(['./contracts']));(out/'example.stdout').write_text(run(['./example']))
(out/'binary.sha256').write_text(hashlib.sha256(Path('portfolio').read_bytes()).hexdigest())
try:
 p=subprocess.run(['objdump','-Cd','portfolio'],text=True,capture_output=True);(out/'linked-assembly.txt').write_text(p.stdout);(out/'assembly-status.txt').write_text(str(p.returncode))
except FileNotFoundError:(out/'assembly-status.txt').write_text('unavailable')
methods=['unstable','stable','quick','heap','merge','radix','dispatch']
cells=[('cold16',16,'cold',1),('tiny16',16,'random',1024),('random4096',4096,'random',16),('random65536',65536,'random',2),('large1048576',1048576,'random',1),('equal4096',4096,'equal',16),('duplicates4096',4096,'duplicates',16),('sorted4096',4096,'sorted',16),('reverse4096',4096,'reverse',16),('organ4096',4096,'organ',16),('runs4096',4096,'runs',16),('nearly4096',4096,'nearly',16),('groups4096x16',4096,'groups',16),('skew4096',4096,'skew',16)]
records=[]
with (out/'attempts.jsonl').open('w') as f:
 for cell,n,shape,reps in cells:
  available=methods+(['groups'] if shape in ['groups','skew'] else [])
  for block in range(12):
   shift=(block//2)%len(available);order=available[shift:]+available[:shift]
   if block%2:order=list(reversed(order))
   for position,method in enumerate(order):
    args=['taskset','-c',str(cpu),'./portfolio',method,str(n),shape,str(reps),str(9000+block)]
    start=time.perf_counter_ns();stdout=run(args);wall=time.perf_counter_ns()-start
    ns,setup,checksum,unstable_cmp,stable_cmp=map(int,stdout.split());assert ns>0
    rec=dict(cell=cell,n=n,shape=shape,method=method,reps=reps,block=block,position=position,ns_per_call=ns/reps,clone_ns_per_call=setup/reps,wall_ns=wall,checksum=checksum,std_unstable_comparisons=unstable_cmp,std_stable_comparisons=stable_cmp,args=args)
    records.append(rec);f.write(json.dumps(rec)+'\n');f.flush()
  print(cell,'COMPLETE',flush=True)
print('CONTRACTS_AND_CAMPAIGN_OK',len(records))
