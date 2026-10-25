#!/usr/bin/env python3
"""Eight fresh processes per candidate/cell; a fixed balanced rotation."""
import json, pathlib, statistics, subprocess, sys
ROOT=pathlib.Path(__file__).resolve().parent
CELLS=['tiny','small','mixed','reads','unions','local','duplicates','first','repeat']
CANDIDATES=['labels','size','full','half']
out=pathlib.Path(sys.argv[1]); out.mkdir(exist_ok=False)
def execute(c,w):
    r=subprocess.run([str(ROOT/'bench'),c,w],capture_output=True,text=True,check=True)
    return json.loads(r.stdout)
subprocess.run(['rustc','--edition','2024','-O','-C','target-cpu=generic','-A','dead_code',str(ROOT/'src/bench.rs'),'-o',str(ROOT/'bench')],check=True)
rows=[]
for w in CELLS:
    for c in CANDIDATES: execute(c,w)
    for block in range(8):
        order=CANDIDATES[block%4:]+CANDIDATES[:block%4]
        if block >= 4: order=list(reversed(order))
        for c in order:
            r=execute(c,w); r['block']=block; rows.append(r)
            with (out/'raw.jsonl').open('a') as f: f.write(json.dumps(r)+'\n')
# A/A diagnostic, process assignment balanced by reversal.
for block in range(8):
    for c in (['half','half_aa'] if block%2==0 else ['half_aa','half']):
        r=execute(c,'mixed'); r['block']=block; r['aa']=True; rows.append(r)
        with (out/'raw.jsonl').open('a') as f:f.write(json.dumps(r)+'\n')
summary={}
for w in CELLS:
    values={c:[r['ns'] for r in rows if r['workload']==w and r['candidate']==c and not r.get('aa')] for c in CANDIDATES}
    medians={c:statistics.median(v) for c,v in values.items()}
    fastest=min(medians,key=medians.get)
    # Strong descriptive criterion: every paired rival/fastest exceeds 1.03.
    selected=fastest if all(min(a/b for a,b in zip(values[c],values[fastest]))>1.03 for c in CANDIDATES if c!=fastest) else 'unresolved'
    summary[w]={'fastest_median':fastest,'selected':selected,'candidates':{c:{'median_ns':medians[c],'min_ns':min(v),'max_ns':max(v),'samples':len(v),'median_setup_ns':statistics.median(r['setup_ns'] for r in rows if r['workload']==w and r['candidate']==c and not r.get('aa')),'median_teardown_ns':statistics.median(r['teardown_ns'] for r in rows if r['workload']==w and r['candidate']==c and not r.get('aa'))} for c,v in values.items()}}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
aa={c:[r['ns'] for r in rows if r.get('aa') and r['candidate']==c] for c in ['half','half_aa']}
ratios=[b/a for a,b in zip(aa['half'],aa['half_aa'])]
(out/'aa.json').write_text(json.dumps({'median_ratio':statistics.median(ratios),'min_ratio':min(ratios),'max_ratio':max(ratios)},indent=2)+'\n')
print(json.dumps({'measured_processes':len(rows),'warmups':36,'summary':summary,'aa':ratios}))
