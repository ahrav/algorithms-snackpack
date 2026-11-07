#!/usr/bin/env python3
"""Frozen exact-source process schedule. Run from an exported topic directory."""
import hashlib,json,os,platform,subprocess,sys,time
from pathlib import Path
root=Path(__file__).resolve().parent.parent
os.chdir(root)
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
def run(args):
 p=subprocess.run(args,text=True,capture_output=True)
 if p.returncode: raise RuntimeError(str(args)+'\n'+p.stdout+p.stderr)
 return p.stdout+p.stderr
files=['src/lib.rs','tests/contracts.rs','examples/compare.rs','measurements/runner.py']
manifest={p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in files}
(out/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
env={'hostname':platform.node(),'uname':list(platform.uname()),'affinity':sorted(os.sched_getaffinity(0)),'rustc':run(['rustc','-Vv']),'cpu':run(['lscpu']),'cfg':run(['rustc','--print','cfg','-C','target-cpu=native']),'flags':['-O','--edition=2024','-C','target-cpu=native']}
(out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
run(['rustc','--edition=2024','-O','-C','target-cpu=native','--crate-name','topic_042_lcs_diff','--crate-type','rlib','src/lib.rs','-o',str(out/'libtopic_042_lcs_diff.rlib')])
link=['--extern','topic_042_lcs_diff='+str(out/'libtopic_042_lcs_diff.rlib')]
run(['rustc','--edition=2024','-O','-C','target-cpu=native','examples/compare.rs',*link,'-o',str(out/'compare')])
run(['rustc','--edition=2024','--test','tests/contracts.rs',*link,'-o',str(out/'contracts')])
(out/'contracts.txt').write_text(run([str(out/'contracts')]))
(out/'binary.sha256').write_text(hashlib.sha256((out/'compare').read_bytes()).hexdigest()+'\n')
candidates=['matrix','hirschberg','myers'];shapes=['identical','sparse','unrelated','repeat','unequal']
plan=[]
for n in [16,256,1024]:
 for shape in shapes:
  reps={16:200,256:20,1024:10}[n]
  for c in candidates: plan.append(dict(phase='warmup',block=-1,candidate=c,shape=shape,n=n,reps=reps))
  for block in range(8):
   shift=(block//2)%3; order=candidates[shift:]+candidates[:shift]
   if block%2:order=order[::-1]
   for c in order:plan.append(dict(phase='measure',block=block,candidate=c,shape=shape,n=n,reps=reps))
for block in range(8):
 for label in (['A','B'] if block%2==0 else ['B','A']):plan.append(dict(phase='aa',block=block,label=label,candidate='matrix',shape='sparse',n=256,reps=20))
(out/'plan.json').write_text(json.dumps(plan,indent=2)+'\n')
cpu=str(min(os.sched_getaffinity(0)))
with (out/'records.jsonl').open('w') as f:
 for index,p in enumerate(plan):
  start=time.time_ns()
  result=json.loads(run(['taskset','-c',cpu,str(out/'compare'),p['candidate'],p['shape'],str(p['n']),str(p['reps'])]))
  f.write(json.dumps(dict(index=index,**p,result=result,start_ns=start))+'\n');f.flush()
(out/'complete.json').write_text(json.dumps(dict(processes=len(plan),inputs=manifest,completed_ns=time.time_ns()),indent=2)+'\n')
print(json.dumps(dict(processes=len(plan),host=platform.node(),out=str(out))))
