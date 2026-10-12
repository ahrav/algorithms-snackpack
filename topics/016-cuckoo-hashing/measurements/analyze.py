#!/usr/bin/env python3
"""Descriptive range and fixed all-pairs dominance; compatible with Python3.7."""
import json,pathlib,statistics,sys
CANDIDATES=['linear','cuckoo1','cuckoo4']
def summarize(rows,pairs):
 summary={}
 for case in dict.fromkeys(r['case'] for r in rows):
  measured={c:[r['ns_per_operation'] for r in sorted(rows,key=lambda r:r['block']) if r['case']==case and r['candidate']==c and r['status']=='ok'] for c in CANDIDATES}
  admissible=[c for c,v in measured.items() if len(v)==pairs]
  def beats(c,d):return all(x*1.05<y for x,y in zip(measured[c],measured[d]))
  winners=[c for c in admissible if all(beats(c,d) for d in admissible if d!=c)]
  contenders=[c for c in admissible if not any(beats(d,c) for d in admissible if d!=c)]
  summary[case]={'selected':winners[0] if len(winners)==1 else 'unresolved','contenders':contenders,'admissible':admissible,'timings':{c:{'median':statistics.median(v),'min':min(v),'max':max(v)} for c,v in measured.items() if v}}
 return summary
if __name__=='__main__':
 p=pathlib.Path(sys.argv[1]);m=json.loads((p/'manifest.json').read_text());r=summarize(json.loads((p/'rows.json').read_text()),m['pairs']);(p/'summary.json').write_text(json.dumps(r,indent=2));print(json.dumps(r,indent=2))
