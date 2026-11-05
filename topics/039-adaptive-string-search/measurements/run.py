#!/usr/bin/env python3
"""Run a fixed, balanced fresh-process schedule on the current Linux host."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = Path(sys.argv[1] if len(sys.argv)>1 else 'run-039').resolve()
OUT.mkdir(parents=True, exist_ok=False)
CANDIDATES = ['direct', 'horspool', 'bm', 'tw', 'adaptive']
def call(args):
    return subprocess.check_output(args, text=True, cwd=ROOT, stderr=subprocess.STDOUT)
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
inputs = ['src/lib.rs','benches/search.rs','tests/contracts.rs','examples/search.rs','measurements/run.py']
manifest = {p:sha(ROOT/p) for p in inputs}
(OUT/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
cpu = min(os.sched_getaffinity(0))
metadata = {'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'rustc':call(['rustc','-Vv']),'cpus':call(['lscpu']),'available_cpus':sorted(os.sched_getaffinity(0)),'affinity_cpu':cpu,'flags':['-C','opt-level=3','-C','target-cpu=native','--edition=2024'],'start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
(OUT/'environment.json').write_text(json.dumps(metadata,indent=2)+'\n')
lib = OUT/'libtopic_039_adaptive_string_search.rlib'
call(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native','--crate-type=rlib','--crate-name','topic_039_adaptive_string_search','src/lib.rs','-o',str(lib)])
extern = 'topic_039_adaptive_string_search='+str(lib)
for source, name, extra in [('tests/contracts.rs','contracts',['--test']),('examples/search.rs','example',[]),('benches/search.rs','search',[])]:
    call(['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native',source,'--extern',extern,'-o',str(OUT/name)]+extra)
(OUT/'contracts.txt').write_text(call([str(OUT/'contracts')]))
(OUT/'example.txt').write_text(call([str(OUT/'example')]))
(OUT/'binary.sha256').write_text(sha(OUT/'search')+'\n')
(OUT/'disassembly.txt').write_text(call(['objdump','-d','-C',str(OUT/'search')]))
cells=list(itertools.product([64,32768],[4,32],['wide','small','prefix','suffix','early'],['prepared','oneshot']))
schedule=[]
for n,m,shape,endpoint in cells:
    for c in CANDIDATES: schedule.append(dict(n=n,m=m,shape=shape,endpoint=endpoint,candidate=c,arm=c,phase='warmup',block=-1))
    for block in range(8):
        order=CANDIDATES if block%2==0 else list(reversed(CANDIDATES))
        for c in order:schedule.append(dict(n=n,m=m,shape=shape,endpoint=endpoint,candidate=c,arm=c,phase='measured',block=block))
# Identical direct-search labels use the same invocation and parser paths.
for block in range(8):
    for label in (['A','B'] if block%2==0 else ['B','A']):
        schedule.append(dict(n=32768,m=32,shape='wide',endpoint='prepared',candidate='direct',arm=label,phase='aa',block=block))
(OUT/'schedule.json').write_text(json.dumps(schedule,indent=2)+'\n')
with (OUT/'results.jsonl').open('w') as rows:
    for index,row in enumerate(schedule):
        args=['taskset','-c',str(cpu),str(OUT/'search'),row['candidate'],str(row['n']),str(row['m']),row['shape'],row['endpoint']]
        try:
            process=subprocess.run(args,cwd=ROOT,text=True,capture_output=True,timeout=60)
            record: dict = dict(row,index=index,returncode=process.returncode,stdout=process.stdout,stderr=process.stderr)
            if process.returncode==0:
                try:
                    result=json.loads(process.stdout)
                    record.update(result,ns_per_query=result['ns_total']/result['iterations'])
                except (ValueError,KeyError) as malformed:
                    record.update(returncode=None,error=repr(malformed))
        except subprocess.TimeoutExpired as expired:
            record=dict(row,index=index,returncode=None,error=repr(expired),stdout=expired.stdout,stderr=expired.stderr)
        rows.write(json.dumps(record)+'\n');rows.flush()
(OUT/'complete.json').write_text(json.dumps({'processes':len(schedule),'source':manifest,'binary':sha(OUT/'search'),'end_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())},indent=2)+'\n')
print(json.dumps({'out':str(OUT),'processes':len(schedule)}))
