#!/usr/bin/env python3
"""Frozen 12-block, rotated/reversed process campaign, builtin Python only."""
import hashlib,json,math,os,platform,statistics,subprocess,time
from pathlib import Path
root=Path('.'); out=Path('results'); out.mkdir(exist_ok=False)
def run(args):
 p=subprocess.run(args,text=True,capture_output=True); assert p.returncode==0,(args,p.stderr); return p.stdout
source=Path('topic')
assert all(hashlib.sha256(Path(p).read_bytes()).hexdigest()==h for p,h in json.loads(Path('source-manifest.json').read_text()).items())
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':run(['uname','-a']),'cpuinfo':Path('/proc/cpuinfo').read_text(),'rustc':run(['rustc','-vV']),'target_cfg':run(['rustc','--print','cfg']),'available_cpus':sorted(os.sched_getaffinity(0)),'flags':['--edition=2024','-C','opt-level=3'],'boundary':'weight setup excluded; sampler validation/RNG/allocation/keys/output/sum/destruction included; startup separately recorded','warmup':'3 calls; cold cells 0','blocks':12,'assignment':'rotate then reverse alternate blocks; each method separate process; common block seed','n_max':'u32::MAX','distribution_limit':'X and keys finite precision; SplitMix64 not independence proof'}
(out/'environment.json').write_text(json.dumps(env,indent=2))
cpu=env['available_cpus'][0]
run(['rustc','--edition=2024','-C','opt-level=3','--crate-name','sampling_lab','--crate-type','rlib',str(source/'src/lib.rs'),'-o','libsampling_lab.rlib'])
for name,path,test in [('contracts',source/'tests/contracts.rs',True),('sampling',source/'benches/sampling.rs',False),('stream',source/'examples/stream.rs',False)]:
 args=['rustc','--edition=2024','-C','opt-level=3',str(path),'--extern','sampling_lab=libsampling_lab.rlib','-o',name]
 if test:args.append('--test')
 run(args)
(out/'contracts.stdout').write_text(run(['./contracts']))
(out/'example.stdout').write_text(run(['./stream']))
(out/'binary.sha256').write_text(hashlib.sha256(Path('sampling').read_bytes()).hexdigest())
try:
 p=subprocess.run(['objdump','-Cd','sampling'],capture_output=True,text=True);(out/'linked-assembly.txt').write_text(p.stdout);(out/'assembly-status.txt').write_text(str(p.returncode))
except FileNotFoundError:(out/'assembly-status.txt').write_text('unavailable')
cells=[('u-cold',16,3,'cold',['r','x','sort','heap'],1),('u-tiny',16,3,'flat',['r','x','sort','heap'],8192),('u-sparse',4096,8,'flat',['r','x','sort','heap'],128),('u-half',4096,2048,'flat',['r','x','sort','heap'],128),('u-large',65536,8,'flat',['r','x','sort','heap'],8),('u-full',4096,4096,'flat',['r','x','sort','heap'],128),('w-tiny',16,3,'flat',['wsort','wheap'],8192),('w-sparse',4096,8,'flat',['wsort','wheap'],128),('w-half',4096,2048,'flat',['wsort','wheap'],128),('w-skew',65536,8,'skew',['wsort','wheap'],8),('w-ascending',4096,8,'ascending',['wsort','wheap'],128),('w-full',4096,4096,'flat',['wsort','wheap'],128)]
records=[]
with (out/'attempts.jsonl').open('w') as f:
 for cell,n,k,shape,methods,reps in cells:
  for block in range(12):
   shift=block%len(methods);order=methods[shift:]+methods[:shift]
   if block%2:order=list(reversed(order))
   for method in order:
    args=['taskset','-c',str(cpu),'./sampling',method,str(n),str(k),shape,str(reps),str(9000+block*reps)]
    start=time.perf_counter_ns();stdout=run(args);wall=time.perf_counter_ns()-start
    ns,checksum=map(int,stdout.split());assert ns>0
    rec=dict(cell=cell,n=n,k=k,shape=shape,method=method,reps=reps,block=block,ns_per_call=ns/reps,wall_ns=wall,checksum=checksum,args=args)
    records.append(rec);f.write(json.dumps(rec)+'\n');f.flush()
print('CONTRACTS_AND_CAMPAIGN_OK',len(records))
