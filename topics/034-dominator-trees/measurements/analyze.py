"""Validate a complete two-host campaign and reduce process measurements."""
import json,pathlib,statistics,sys,hashlib
STATIC=[('tiny',7,1000),('chain',512,16),('permuted',256,2),('joins',512,16),('cycle',512,16),('dense',128,16),('unreachable',512,16)]
DYNAMIC=[('noop',512,4),('bypass',512,4),('newreach',512,4)]
def analyze(root):
 root=pathlib.Path(root);result={};common=None
 for host,arch in [('arm','aarch64'),('x86','x86_64')]:
  base=root/host;p=base/'results';rows=[json.loads(s) for s in (p/'samples.jsonl').read_text().splitlines()];finish=json.loads((p/'finished.json').read_text());source=json.loads((p/'source.json').read_text());env=json.loads((p/'environment.json').read_text())
  assert finish['complete'] and finish['records']==len(rows)==210
  assert finish['source']==source
  if common is None: common=source
  assert common==source,'cross-host source identity'
  for f,h in source.items():assert hashlib.sha256((base/f).read_bytes()).hexdigest()==h,f
  frozen=json.loads((base/'frozen.json').read_text())
  for f,h in frozen.items():assert hashlib.sha256((base/f).read_bytes()).hexdigest()==h,f
  assert env['arch']==arch and env['available_cpus'] and env['pinned_cpu'] in env['available_cpus']
  assert all(x['code']==0 for x in json.loads((p/'checks.json').read_text()).values())
  cursor=0;cells=[]
  for dynamic,workloads,names in [(False,STATIC,['sets','chk','control']),(True,DYNAMIC,['rebuild','incremental','rebuild-tree'])]:
   for shape,n,reps in workloads:
    samples={k:{} for k in names};meta=None
    for block in range(-1,6):
     for name in (names if block%2==0 else list(reversed(names))):
      r=rows[cursor];cursor+=1
      assert (r['candidate'],r['shape'],r['n'],r['reps'],r['block'],r['dynamic'],r['oracle'])==(name,shape,n,reps,block,dynamic,True)
      assert r['ns']>0 and r['wall_ns']>=r['ns']
      if meta is None:meta=r['m']
      assert meta==r['m']
      if block>=0:samples[name][block]=r['ns']/reps/1000
    distinct=names if dynamic else ['sets','chk']
    leader=min(distinct,key=lambda a:statistics.median(samples[a].values()))
    winner=leader if all(samples[leader][i]<=.95*samples[b][i] for b in distinct if b!=leader for i in range(6)) else 'unresolved'
    stats={a:{'median_us':statistics.median(v.values()),'range_us':[min(v.values()),max(v.values())]} for a,v in samples.items()}
    counters={a:{k:next(r[k] for r in rows if r['shape']==shape and r['candidate']==a) for k in ['visits','edges','steps']} for a in names}
    cell=dict(shape=shape,n=n,m=meta,dynamic=dynamic,winner=winner,median_leader=leader,candidates=stats,counters=counters)
    if not dynamic:cell['AA_ratio_range']=[min(samples['chk'][i]/samples['control'][i] for i in range(6)),max(samples['chk'][i]/samples['control'][i] for i in range(6))]
    cells.append(cell)
  assert cursor==len(rows)
  result[host]={'hostname':env['hostname'],'rustc':env['rustc']['out'],'cpu':env['pinned_cpu'],'binary_sha256':env['binary_sha256'],'cells':cells}
 return {'verified':True,'source':common,'hosts':result}
if __name__=='__main__':
 r=analyze(sys.argv[1]);pathlib.Path(sys.argv[2]).write_text(json.dumps(r,indent=2)+'\n')
 for h,x in r['hosts'].items():
  for c in x['cells']:print(h,c['shape'],c['winner'],{k:round(v['median_us'],3) for k,v in c['candidates'].items()})
