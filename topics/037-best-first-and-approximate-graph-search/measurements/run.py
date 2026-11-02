#!/usr/bin/env python3
"""Compile frozen inputs; check independent full-sort oracle; run balanced processes."""
import argparse,hashlib,json,os,pathlib,platform,subprocess,time
HERE=pathlib.Path(__file__).resolve().parent.parent
METHODS=['flat','4','16','64']
ORDERS=[['flat','4','64','16'],['4','16','flat','64'],['16','64','4','flat'],['64','flat','16','4']]
def run(args):return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT)
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);p.add_argument('--focus',action='store_true');a=p.parse_args();out=pathlib.Path(a.out);out.mkdir(parents=True,exist_ok=False)
 inputs=['src/lib.rs','tests/contracts.rs','benches/search.rs','measurements/run.py','measurements/summarize.py']
 (out/'inputs.json').write_text(json.dumps({s:hashlib.sha256((HERE/s).read_bytes()).hexdigest() for s in inputs},indent=2))
 env=dict(hostname=platform.node(),uname=list(platform.uname()),cpu=run(['sh','-c','cat /proc/cpuinfo 2>/dev/null || sysctl -n machdep.cpu.brand_string']),rustc=run(['rustc','-vV']),flags='--edition=2024 -C opt-level=3 -C target-cpu=native',target_cfg=run(['rustc','--print','cfg','-C','target-cpu=native']),available_cpus=len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count())
 (out/'environment.json').write_text(json.dumps(env,indent=2))
 lib=out/'libgraph.rlib';exe=out/'search'
 common=['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native']
 run(common+['--crate-name','topic037_graph_search','--crate-type','rlib',str(HERE/'src/lib.rs'),'-o',str(lib)])
 run(common+[str(HERE/'benches/search.rs'),'--extern',f'topic037_graph_search={lib}','-o',str(exe)])
 run(common+['--test',str(HERE/'tests/contracts.rs'),'--extern',f'topic037_graph_search={lib}','-o',str(out/'contracts')])
 (out/'contracts.txt').write_text(run([str(out/'contracts')]))
 (out/'binary.sha256').write_text(hashlib.sha256(exe.read_bytes()).hexdigest())
 # Code inspection is retained as evidence; timings make no hardware-causal claim.
 run(common+[str(HERE/'benches/search.rs'),'--extern',f'topic037_graph_search={lib}','--emit=asm','-o',str(out/'search.s')])
 with (out/'samples.jsonl').open('w') as f:
  for case in range(1 if a.focus else 8):
   fixture=json.loads(run([str(exe),str(case),'fixture']));(out/f'fixture-{case}.json').write_text(json.dumps(fixture))
   points=fixture['points'];queries=fixture['queries'];oracles=[sorted((sum((x-y)**2 for x,y in zip(p,q)),i) for i,p in enumerate(points))[:4] for q in queries]
   for block in range(-1,1 if a.focus else 8):
    for method in (METHODS if block<0 else ORDERS[block%4]):
     start=time.monotonic_ns();row=json.loads(run([str(exe),str(case),method]));wall=time.monotonic_ns()-start
     hits=0;full=True
     for q,answer,expected in zip(queries,row['answers'],oracles):
      ids=[x[1] for x in answer];assert len(ids)==len(set(ids));assert len(ids)<=4;assert answer==sorted(answer)
      assert all(0<=i<len(points) for i in ids)
      assert all(dist==sum((x-y)**2 for x,y in zip(points[i],q)) for dist,i in answer)
      if method=='flat':assert answer==[list(x) for x in expected]
      hits+=len(set(ids)&{x[1] for x in expected});full &= len(ids)==4
     reps=row['count']//len(queries)
     assert row['checksum']==reps*sum(dist+i for answer in row['answers'] for dist,i in answer)
     row.update(case=case,method=method,block=block,ns_per_query=row['ns']/row['count'],recall=hits/(4*len(queries)),full=full,process_wall_ns=wall)
     f.write(json.dumps(row)+'\n');f.flush()
 (out/'summary.json').write_text(run(['python3',str(HERE/'measurements/summarize.py'),str(out)]))
 print(out)
if __name__=='__main__':main()
