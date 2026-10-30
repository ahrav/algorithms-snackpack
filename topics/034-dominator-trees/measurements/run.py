#!/usr/bin/env python3
"""Six balanced fresh-process blocks. Run from the standalone crate root."""
import hashlib,json,os,pathlib,platform,subprocess,sys,time
root=pathlib.Path(__file__).resolve().parents[1]; os.chdir(root)
out=pathlib.Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=False)
inputs=['Cargo.toml','measurements/portable/Cargo.toml','measurements/portable/Cargo.lock','src/lib.rs','tests/contracts.rs','examples/demo.rs','benches/dominators.rs','measurements/run.py']
hashes=lambda:{p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest() for p in inputs}
h=hashes();(out/'source.json').write_text(json.dumps(h,indent=2))
env=os.environ.copy();env['RUSTFLAGS']='-C target-cpu=native -C opt-level=3'
def call(args):

 if args[0]=='cargo': args=args[:2]+['--manifest-path','measurements/portable/Cargo.toml']+args[2:]
 r=subprocess.run(args,env=env,text=True,capture_output=True,timeout=180);return {'code':r.returncode,'out':r.stdout,'err':r.stderr}
checks={}
for name,args in [('tests',['cargo','test']),('example',['cargo','run','--example','demo']),('build',['cargo','build','--release','--benches','--message-format=json-render-diagnostics'])]:
 checks[name]=call(args);(out/'checks.json').write_text(json.dumps(checks,indent=2));assert checks[name]['code']==0,checks[name]
try:
 artifacts=[json.loads(line) for line in checks['build']['out'].splitlines() if line.startswith('{')]
except json.JSONDecodeError as e:
 sys.exit(f'unparsable cargo message stream: {e}')
[binary]=[pathlib.Path(a['executable']) for a in artifacts if a.get('reason')=='compiler-artifact' and a['target']['name']=='dominators' and a['target']['kind']==['bench']]
cpus=sorted(os.sched_getaffinity(0));cpu=cpus[len(cpus)//2]
info={'hostname':platform.node(),'arch':platform.machine(),'uname':list(platform.uname()),'available_cpus':cpus,'pinned_cpu':cpu,'rustflags':env['RUSTFLAGS'],'rustc':call(['rustc','-vV']),'target_features':call(['rustc','--print','cfg','-C','target-cpu=native']),'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text(),'lscpu':call(['lscpu']),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()}
(out/'environment.json').write_text(json.dumps(info,indent=2))
static=[('tiny',7,1000),('chain',512,16),('permuted',256,2),('joins',512,16),('cycle',512,16),('dense',128,16),('unreachable',512,16)]
dynamic=[('noop',512,4),('bypass',512,4),('newreach',512,4)]
with (out/'samples.jsonl').open('w') as f:
 for dynamic_flag,cells,names in [(False,static,['sets','chk','control']),(True,dynamic,['rebuild','incremental','rebuild-tree'])]:
  for shape,n,reps in cells:
   for block in range(-1,6):
    order=names if block%2==0 else list(reversed(names))
    for candidate in order:
     start=time.monotonic_ns();r=call(['taskset','-c',str(cpu),str(binary),candidate,shape,str(n),str(reps)]);wall=time.monotonic_ns()-start
     assert r['code']==0,r
     data=json.loads(r['out']);data.update(block=block,wall_ns=wall,dynamic=dynamic_flag)
     f.write(json.dumps(data)+'\n');f.flush()
assert h==hashes(),'source changed during run'
subprocess.run(['objdump','-Cd',str(binary)],stdout=(out/'assembly.txt').open('w'),check=True)
(out/'finished.json').write_text(json.dumps({'records':210,'source':h,'complete':True}))
print(out,flush=True)
