import hashlib,json,pathlib,platform,subprocess,statistics,os
root=pathlib.Path(__file__).resolve().parent
out=root/'raw';out.mkdir(exist_ok=True)
def call(argv):
 r=subprocess.run(argv,cwd=root,capture_output=True,text=True)
 if r.returncode: raise RuntimeError(str(argv)+'\n'+r.stdout+r.stderr)
 return r.stdout
files=['src/lib.rs','benches/portfolio.rs','runner.py']
manifest={'inputs':{f:hashlib.sha256((root/f).read_bytes()).hexdigest() for f in files},'hostname':platform.node(),'arch':platform.machine(),'uname':platform.uname()._asdict(),'rustc':call(['rustc','-Vv']),'features':call(['rustc','--print','cfg']),'cpus':os.cpu_count(),'affinity':sorted(os.sched_getaffinity(0)),'cpu':call(['sh','-c',"lscpu"]),'flags':'--edition 2024 -O; default target; no target-cpu=native; unpinned'}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
call(['rustc','--edition','2024','--test','src/lib.rs','-o',str(out/'tests')]);(out/'tests.log').write_text(call([str(out/'tests')]))
call(['rustc','--edition','2024','--crate-name','segment_trees','--crate-type','rlib','-O','src/lib.rs','-o',str(out/'libsegment_trees.rlib')])
call(['rustc','--edition','2024','-O','benches/portfolio.rs','--extern','segment_trees='+str(out/'libsegment_trees.rlib'),'-o',str(out/'portfolio')])
(out/'disassembly.txt').write_text(call(['objdump','-d',str(out/'portfolio')]))
candidates=['scan','prefix','eager','lazy'];cells=['tiny','read_only','mixed','short','update_heavy','hot','adversarial','lifecycle'];rows=[]
for cell in cells:
 for block in range(8):
  order=candidates[block%4:]+candidates[:block%4]
  if block>=4:order=list(reversed(order))
  for c in order:
   row=json.loads(call([str(out/'portfolio'),c,cell]));row.update(cell=cell,block=block,candidate=c);rows.append(row)
   with (out/'attempts.jsonl').open('a') as f:f.write(json.dumps(row)+'\n')
summary={}
for cell in cells:
 data={c:[r['ns'] for r in rows if r['cell']==cell and r['candidate']==c] for c in candidates}
 assert len({r['checksum'] for r in rows if r['cell']==cell})==1
 med={c:statistics.median(v) for c,v in data.items()};best=min(med,key=med.get)
 ratios={c:[data[c][i]/data[best][i] for i in range(8)] for c in candidates if c!=best}
 winner=best if all(min(v)>1.05 for v in ratios.values()) else 'unresolved'
 summary[cell]={'winner':winner,'median_ns':med,'range_ns':{c:[min(v),max(v)] for c,v in data.items()},'paired_against_median_best':{c:{'median':statistics.median(v),'range':[min(v),max(v)]} for c,v in ratios.items()}}
(out/'rows.json').write_text(json.dumps(rows,indent=2));(out/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps({'processes':len(rows),'winners':{c:s['winner'] for c,s in summary.items()}}))
