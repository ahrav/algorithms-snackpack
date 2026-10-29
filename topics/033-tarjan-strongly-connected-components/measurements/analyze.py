import collections,itertools,json,statistics,sys
from pathlib import Path
base=Path(sys.argv[1]);out={}
def environment(host):
 path=base/host/'results'/'environment.json'
 try:
  return json.loads(path.read_text())
 except (OSError,ValueError) as e:
  raise SystemExit(f'unreadable environment record {path}: {e}') from e
envs={host:environment(host) for host in ['arm','x86']}
assert envs['arm']['source_hashes']==envs['x86']['source_hashes'],('source_hashes differ across hosts',envs['arm']['source_hashes'],envs['x86']['source_hashes'])
for host in ['arm','x86']:
 run=base/host/'results'
 env=envs[host]
 recs=[json.loads(x) for x in (run/'records.jsonl').read_text().splitlines()]
 assert len(recs)==350 and all(r['exit']==0 and r['certified'] for r in recs)
 assert sum(r['phase']=='warmup' for r in recs)==50,'warmup process count'
 vals=collections.defaultdict(dict)
 for r in recs:
  if r['phase']=='measure':
   assert r['block'] not in vals[(r['workload'],r['candidate'])]
   vals[(r['workload'],r['candidate'])][r['block']]=r
 workloads=list(dict.fromkeys(r['workload'] for r in recs));cells={}
 assert workloads==['tiny-first','tiny','isolated','chain','cycle','groups','permuted','duplicates','dense','random'],workloads
 for w in workloads:
  shapes={(r['vertices'],r['edges'],r['components']) for r in recs if r['workload']==w}
  assert len(shapes)==1,('graph metadata differs across processes',w,shapes)
 for w in workloads:
  cs=['tarjan','gabow','kosaraju','prepared','tarjan-aa'];allstats={}
  for c in cs:
   rs=vals[(w,c)];assert set(rs)==set(range(6));v=[rs[b]['ns_per_op'] for b in range(6)]
   allstats[c]={'median_ns':statistics.median(v),'min_ns':min(v),'max_ns':max(v),'samples_ns':v}
  for a,b in itertools.combinations(cs,2):
   assert sum(vals[(w,a)][i]['position']<vals[(w,b)][i]['position'] for i in range(6))==3
  best=min(cs[:3],key=lambda c:allstats[c]['median_ns'])
  def decisive(a,b):
   return allstats[a]['median_ns']<=.95*allstats[b]['median_ns'] and all(vals[(w,a)][i]['ns_per_op']<vals[(w,b)][i]['ns_per_op'] for i in range(6))
  chosen=best if all(decisive(best,c) for c in cs[:3] if c!=best) else 'unresolved'
  ratios={c:[vals[(w,c)][b]['ns_per_op']/vals[(w,best)][b]['ns_per_op'] for b in range(6)] for c in cs}
  aa=[vals[(w,'tarjan-aa')][b]['ns_per_op']/vals[(w,'tarjan')][b]['ns_per_op'] for b in range(6)]
  prepared='prepared faster' if decisive('prepared',best) else (best+' faster' if decisive(best,'prepared') else 'unresolved')
  cells[w]={'outgoing_only_selection':chosen,'outgoing_only_median_leader':best,'stats':allstats,'ratios_to_leader':ratios,'aa_ratio':{'median':statistics.median(aa),'min':min(aa),'max':max(aa)},'prepared_conditional_comparison':prepared,'vertices':vals[(w,best)][0]['vertices'],'edges':vals[(w,best)][0]['edges'],'components':vals[(w,best)][0]['components']}
 out[host]={'hostname':env['hostname'],'architecture':env['architecture'],'cells':cells,'measured_processes':300,'warmup_processes':50,'source_hashes':env['source_hashes']}
Path(sys.argv[2]).write_text(json.dumps(out,indent=2)+'\n')
for host,d in out.items():
 print(host)
 for w,c in d['cells'].items():print(w,c['outgoing_only_selection'],c['outgoing_only_median_leader'],*[round(c['stats'][x]['median_ns']/1000,3) for x in ['tarjan','gabow','kosaraju','prepared']],c['prepared_conditional_comparison'])
 print('AA range',min(c['aa_ratio']['min'] for c in d['cells'].values()),max(c['aa_ratio']['max'] for c in d['cells'].values()))
