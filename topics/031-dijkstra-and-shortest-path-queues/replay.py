#!/usr/bin/env python3
"""Build exact dependency-free source and retain a balanced process campaign."""
import subprocess,json,pathlib,hashlib,platform,os,sys,time
root=pathlib.Path(__file__).resolve().parent
out=pathlib.Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=False)
def cmd(args):
 r=subprocess.run(args,text=True,capture_output=True,check=True,cwd=root)
 return r.stdout+r.stderr
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
files=['src/lib.rs','tests/contracts.rs','examples/running.rs','benches/queues.rs','replay.py']
identity={p:digest(root/p) for p in files}
(out/'source-hashes.json').write_text(json.dumps(identity,indent=2))
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'available_cpus':len(os.sched_getaffinity(0)),'rustc':cmd(['rustc','-Vv']),'cpu':cmd(['lscpu']),'flags':'--edition=2024 -C opt-level=3 -C target-cpu=native; default allocator; stats included'}
(out/'environment.json').write_text(json.dumps(env,indent=2))
cmd(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native','--crate-type=lib','--crate-name','shortest_path_queues','src/lib.rs','-o',str(out/'libshortest_path_queues.rlib')])
link=['--extern','shortest_path_queues='+str(out/'libshortest_path_queues.rlib')]
cmd(['rustc','--edition=2024','--test','tests/contracts.rs',*link,'-o',str(out/'contracts')])
(out/'contracts.txt').write_text(cmd([str(out/'contracts')]))
cmd(['rustc','--edition=2024','--test','src/lib.rs','-o',str(out/'unit-tests')])
(out/'unit-tests.txt').write_text(cmd([str(out/'unit-tests')]))
cmd(['rustc','--edition=2024','examples/running.rs',*link,'-o',str(out/'running')])
(out/'example.txt').write_text(cmd([str(out/'running')]))
cmd(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native','benches/queues.rs',*link,'-o',str(out/'queues')])
(out/'linked-symbols.txt').write_text(cmd(['nm','-C',str(out/'queues')]))
(out/'disassembly.txt').write_text(cmd(['objdump','-d','-C',str(out/'queues')]))
workloads=['tiny','sparse_small','sparse_large','bounded','zero_one','dense','decreases','wide_chain','disconnected']
with (out/'processes.jsonl').open('w') as stream:
 for workload in workloads:
  candidates=['scan','lazy','indexed','radix']
  if workload in ['tiny','bounded','zero_one','disconnected']:candidates+=['dial']
  if workload in ['zero_one','disconnected']:candidates+=['zero_one']
  candidates+=['lazy_aa']
  for block in range(-1,6):
   order=candidates if block%2==0 else list(reversed(candidates))
   for position,candidate in enumerate(order):
    start=time.monotonic_ns()
    row=json.loads(cmd([str(out/'queues'),workload,candidate]))
    row.update(block=block,position=position,process_wall_ns=time.monotonic_ns()-start)
    stream.write(json.dumps(row)+'\n');stream.flush()
  print(workload,flush=True)
assert identity=={p:digest(root/p) for p in files},'source changed'
manifest={str(p.relative_to(out)):digest(p) for p in out.rglob('*') if p.is_file()}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
print('COMPLETE',len(manifest),flush=True)
