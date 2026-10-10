"""Reduce paired process samples without treating batches as independent samples."""
import json,pathlib,statistics,sys
root=pathlib.Path(sys.argv[1]); records=[json.loads(x) for x in (root/'samples.jsonl').read_text().splitlines()]
assert len(records)==360 and all(r['oracle'] for r in records)
result=[]
for workload in sorted({r['workload'] for r in records}):
 data={c:sorted([r for r in records if r['workload']==workload and r['candidate']==c],key=lambda r:r['block']) for c in ['unfiltered','scalar','word']}
 assert all([r['block'] for r in rows]==list(range(12)) for rows in data.values())
 medians={c:statistics.median(r['ns']/r['queries']/r['repeats'] for r in rows) for c,rows in data.items()}
 fastest=min(medians,key=medians.get)
 ratios={c:[row['ns']/winner['ns'] for row,winner in zip(rows,data[fastest])] for c,rows in data.items() if c!=fastest}
 unique=all(min(r)>1.05 for r in ratios.values())
 result.append({'workload':workload,'selected':fastest if unique else 'unresolved','lowest_median':fastest,'ns_per_operation':{c:{'median':medians[c],'min':min(r['ns']/r['queries']/r['repeats'] for r in rows),'max':max(r['ns']/r['queries']/r['repeats'] for r in rows),'groups_per_query':rows[0]['groups']/rows[0]['queries'],'equalities_per_query':rows[0]['equalities']/rows[0]['queries']} for c,rows in data.items()},'paired_rival_over_fastest_ranges':{c:[min(r),max(r)] for c,r in ratios.items()}})
print(json.dumps({'identity':json.loads((root/'identity.json').read_text()),'results':result},indent=2))
