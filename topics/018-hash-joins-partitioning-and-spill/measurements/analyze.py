import statistics

def summarize(rows,pairs):
 result={}
 for case in sorted({r['case'] for r in rows}):
  selected=[r for r in rows if r['case']==case]
  assert len(selected)==pairs*4
  by={c:sorted([r for r in selected if r['candidate']==c],key=lambda r:r['block']) for c in {r['candidate'] for r in selected}}
  assert all(len(v)==pairs and all(r['status']=='ok' for r in v) for v in by.values())
  vals={c:[r['ns_per_operation'] for r in v] for c,v in by.items()}
  winners=[c for c in vals if all(all(x*1.05<y for x,y in zip(vals[c],vals[d])) for d in vals if d!=c)]
  contenders=[c for c in vals if not any(all(x*1.05<y for x,y in zip(vals[d],vals[c])) for d in vals if d!=c)]
  result[case]={'winner':winners[0] if len(winners)==1 else 'unresolved','contenders':sorted(contenders),'candidates':{c:{'median_ns':statistics.median(v),'min_ns':min(v),'max_ns':max(v),'pairs':by[c][0]['pairs_count'],'build':by[c][0]['build'],'probe':by[c][0]['probe'],'budget':by[c][0]['budget']} for c,v in vals.items()}}
 return result
