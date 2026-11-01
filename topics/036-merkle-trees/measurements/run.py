#!/usr/bin/env python3
"""Frozen balanced process schedule and independent hashlib root oracle."""
import argparse, hashlib, itertools, json, os, pathlib, platform, subprocess, time
HERE=pathlib.Path(__file__).resolve().parent.parent
CASES=[('tiny',4,1,1,'cluster'),('small',64,8,32,'cluster'),('sparse',8192,1,32,'scatter'),('cluster',8192,64,32,'cluster'),('scatter',8192,64,32,'scatter'),('dense',8192,8192,32,'scatter'),('duplicate',8192,1024,32,'duplicate'),('noop',8192,64,32,'noop'),('large-record',1024,1,4096,'cluster')]
def root(xs):
 if len(xs)==1:return hashlib.sha256(b'\0'+xs[0]).digest()
 m=len(xs)//2
 return hashlib.sha256(b'\1'+root(xs[:m])+root(xs[m:])).digest()
def oracle(n,k,size,pattern,reps,mode):
 initial=[bytes((i*17+j*13)%251 for j in range(size)) for i in range(n)]
 xs=initial[:]
 for r in range(reps):
  if mode=='lifecycle':xs=initial[:]
  for q in range(k):
   i=q*4051%n if pattern=='scatter' else n//2 if pattern=='duplicate' else q%n
   v=initial[i]
   if pattern!='noop':
    v=bytes([(v[0]+(r+q)%250+1)%251])+v[1:]
    assert v!=xs[i],(r,q,i)
   xs[i]=v
 return root(xs).hex()
def run(cmd):return subprocess.check_output(cmd,text=True,stderr=subprocess.STDOUT)
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=pathlib.Path(a.out);out.mkdir(parents=True,exist_ok=False)
 inputs=['src/lib.rs','benches/updates.rs','measurements/run.py','measurements/summarize.py','tests/contracts.rs']
 (out/'inputs.json').write_text(json.dumps({s:hashlib.sha256((HERE/s).read_bytes()).hexdigest() for s in inputs},indent=2))
 env={'hostname':platform.node(),'uname':list(platform.uname()),'cpu':run(['sh','-c','cat /proc/cpuinfo 2>/dev/null || sysctl -n machdep.cpu.brand_string']),'rustc':run(['rustc','-vV']),'available_cpus':len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),'flags':'--edition=2024 -C opt-level=3 -C target-cpu=native; portable allocating SHA256; no crypto intrinsics','target_cfg':run(['rustc','--print','cfg','-C','target-cpu=native'])}
 (out/'environment.json').write_text(json.dumps(env,indent=2))
 lib=out/'libmerkle.rlib';exe=out/'updates';tests=out/'tests'
 run(['rustc','--edition=2024','--crate-name','topic036_merkle_trees','--crate-type','rlib','-C','opt-level=3','-C','target-cpu=native',str(HERE/'src/lib.rs'),'-o',str(lib)])
 run(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native',str(HERE/'benches/updates.rs'),'--extern',f'topic036_merkle_trees={lib}','-o',str(exe)])
 run(['rustc','--edition=2024','--test',str(HERE/'src/lib.rs'),'-o',str(tests)])
 (out/'tests.txt').write_text(run([str(tests)]))
 run(['rustc','--edition=2024','--test',str(HERE/'tests/contracts.rs'),'--extern',f'topic036_merkle_trees={lib}','-o',str(out/'contracts')])
 (out/'contracts.txt').write_text(run([str(out/'contracts')]))
 lengths=list(range(130))+[255,256,511,512,1000,4096]
 for n in lengths:
  got=run([str(exe),'hash',str(n)]).strip();assert got==hashlib.sha256(bytes(i%251 for i in range(n))).hexdigest(),n
 (out/'sha256-oracle.json').write_text(json.dumps({'lengths_checked':len(lengths),'status':'PASS'}))
 (out/'binary.sha256').write_text(hashlib.sha256(exe.read_bytes()).hexdigest()+'\n')
 permutations=list(itertools.permutations(['full','eager','batch']))
 with (out/'samples.jsonl').open('w') as f:
  for name,n,k,size,pattern in CASES:
   reps=min(2048,max(2,32768//n))
   for mode in ['update','lifecycle']:
    expected=oracle(n,k,size,pattern,reps,mode)
    for block in range(-1,6):
     order=permutations[block] if block>=0 else permutations[0]
     for method in order:
      wall=time.monotonic_ns();line=run([str(exe),str(n),str(k),str(size),pattern,str(reps),mode,method]).strip();wall=time.monotonic_ns()-wall
      elapsed,count,calls,build,got=line.split(',');assert got==expected,(name,mode,method,got,expected)
      row=dict(workload=name,n=n,k=k,bytes=size,pattern=pattern,mode=mode,block=block,method=method,ns=int(elapsed)/int(count),hash_calls=int(calls)/int(count),build_ns=int(build),process_wall_ns=wall,root=got)
      f.write(json.dumps(row)+'\n');f.flush()
 (out/'summary.json').write_text(run(['python3',str(HERE/'measurements/summarize.py'),str(out)]))
 print(out)
if __name__=='__main__':main()
