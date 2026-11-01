#!/usr/bin/env python3
"""Descriptive paired medians, ranges and conservative fixed-schedule selection."""
import collections,json,pathlib,statistics,sys
rows=[json.loads(s) for s in (pathlib.Path(sys.argv[1])/'samples.jsonl').read_text().splitlines()]
groups=collections.defaultdict(list)
for r in rows:
 if r['block']>=0:groups[r['workload'],r['mode']].append(r)
result=[]
for (workload,mode),rs in groups.items():
 by={m:sorted([r for r in rs if r['method']==m],key=lambda r:r['block']) for m in ['full','eager','batch']}
 assert all(len(x)==6 for x in by.values())
 stats={m:{'median_ns':statistics.median(r['ns'] for r in xs),'min_ns':min(r['ns'] for r in xs),'max_ns':max(r['ns'] for r in xs),'hash_calls':sorted(set(r['hash_calls'] for r in xs))} for m,xs in by.items()}
 best=min(stats,key=lambda m:stats[m]['median_ns'])
 wins=all(by[best][i]['ns']*1.05<by[other][i]['ns'] for other in by if other!=best for i in range(6))
 result.append({'workload':workload,'mode':mode,'fastest_median':best,'selection':best if wins else 'unresolved','stats':stats})
print(json.dumps({'process_records':len(rows),'measured_records':sum(r['block']>=0 for r in rows),'cells':result},indent=2))
