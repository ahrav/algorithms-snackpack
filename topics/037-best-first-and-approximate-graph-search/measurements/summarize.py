#!/usr/bin/env python3
"""Fixed quality gate and all-pairs five-percent selection; descriptive ranges."""
import json,pathlib,statistics,sys
p=pathlib.Path(sys.argv[1]);rows=[json.loads(s) for s in (p/'samples.jsonl').read_text().splitlines()];result=[]
for case in sorted({r['case'] for r in rows}):
 grouped={m:[r for r in rows if r['case']==case and r['method']==m and r['block']>=0] for m in ['flat','4','16','64']}
 if not grouped['flat']:continue
 assert all([r['block'] for r in rs]==list(range(len(grouped['flat']))) for rs in grouped.values()),('incomplete or misordered blocks',case)
 stats={m:dict(median_ns=statistics.median(r['ns_per_query'] for r in rs),min_ns=min(r['ns_per_query'] for r in rs),max_ns=max(r['ns_per_query'] for r in rs),recall=rs[0]['recall'],full=rs[0]['full'],distances=rs[0]['distances']/32,expanded=rs[0]['expanded']/32,frontier_max=rs[0]['frontier'],build_median_ns=statistics.median(r['build_ns'] for r in rs)) for m,rs in grouped.items()}
 qualified=[m for m,s in stats.items() if s['recall']>=.95 and s['full']]
 fastest=min(qualified,key=lambda m:stats[m]['median_ns'])
 winner=next((m for m in qualified if all(all(a['ns_per_query']<.95*b['ns_per_query'] for a,b in zip(grouped[m],grouped[t])) for t in qualified if t!=m)),None)
 ratios={m:[a['ns_per_query']/b['ns_per_query'] for a,b in zip(grouped[m],grouped['flat'])] for m in qualified}
 result.append(dict(case=case,stats=stats,qualified=qualified,fastest_median=fastest,resolved_winner=winner,paired_ratios_to_flat=ratios))
print(json.dumps(result,indent=2))
