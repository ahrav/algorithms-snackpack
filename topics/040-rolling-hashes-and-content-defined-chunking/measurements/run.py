#!/usr/bin/env python3
"""Frozen Linux schedule with independent polynomial and exact-byte oracles."""
import hashlib, itertools, json, os, platform, subprocess, sys, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=Path(sys.argv[1] if len(sys.argv)>1 else 'run-040').resolve()
OUT.mkdir(parents=True,exist_ok=False)
CS=['fixed','direct','rolling','gear','normalized']; M=(1<<64)-1
TIMEOUT=float(os.environ.get('TOPIC_040_PROCESS_TIMEOUT_S',60))

def call(args): return subprocess.check_output(args,text=True,cwd=ROOT,stderr=subprocess.STDOUT)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def fixture(n,shape):
    s=7; out=[]
    for i in range(n):
        s^=(s<<13)&M;s^=s>>7;s^=(s<<17)&M
        out.append(0 if shape=='zero' else i%7 if shape=='periodic' else s&255)
    return bytes(out)
s=1; G=[]
for _ in range(256):
    s=(s+0x9e3779b97f4a7c15)&M;z=((s^(s>>30))*0xbf58476d1ce4e5b9)&M
    z=((z^(z>>27))*0x94d049bb133111eb)&M;G.append(z^(z>>31))
P=[pow(257,j,1<<64) for j in range(32)]
def oracle(data,t,c):
    if c=='fixed': return list(range(t,len(data),t))+([len(data)] if data else [])
    ends=[];start=0
    for end in range(1,len(data)+1):
        length=end-start
        if length<t//4: continue
        if length>=t*4: cut=True
        else:
            if c in ('direct','rolling'):
                h=sum(b*p for b,p in zip(data[end-32:end],reversed(P)))&M; mask=t-1
            else:
                h=sum(G[b]<<j for j,b in enumerate(reversed(data[max(start,end-64):end])))&M
                mask=(2*t-1 if length<t else t//2-1) if c=='normalized' else t-1
            cut=h&mask==0
        if cut: ends.append(end);start=end
    if start<len(data):ends.append(len(data))
    return ends
def chunks(data,ends):return [data[a:b] for a,b in zip([0]+ends,ends)]
inputs=['src/lib.rs','benches/chunk.rs','tests/contracts.rs','examples/chunk.rs','measurements/run.py']
manifest={p:sha(ROOT/p) for p in inputs};(OUT/'inputs.json').write_text(json.dumps(manifest,indent=2)+'\n')
cpu=min(os.sched_getaffinity(0))
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'rustc':call(['rustc','-Vv']),'cpus':call(['lscpu']),'available_cpus':sorted(os.sched_getaffinity(0)),'affinity_cpu':cpu,'flags':['--edition=2024','-C','opt-level=3','-C','target-cpu=native'],'target_features':call(['rustc','--print','cfg','-C','target-cpu=native']),'process_timeout_s':TIMEOUT,'start_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
assert env['architecture'] in ('aarch64','x86_64')
(OUT/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
lib=OUT/'libtopic_040_chunking.rlib';flags=['--edition=2024','-C','opt-level=3','-C','target-cpu=native']
call(['rustc']+flags+['--crate-type=rlib','--crate-name','topic_040_chunking','src/lib.rs','-o',str(lib)])
for src,name,extra in [('tests/contracts.rs','contracts',['--test']),('examples/chunk.rs','example',[]),('benches/chunk.rs','chunk',[])]:
    call(['rustc']+flags+[src,'--extern','topic_040_chunking='+str(lib),'-o',str(OUT/name)]+extra)
(OUT/'contracts.txt').write_text(call([str(OUT/'contracts')]))
(OUT/'example.txt').write_text(call([str(OUT/'example')]))
(OUT/'binary.sha256').write_text(sha(OUT/'chunk')+'\n')
(OUT/'disassembly.txt').write_text(call(['objdump','-d','-C',str(OUT/'chunk')]))
cells=list(itertools.product([256,65536],[256,1024],['random','zero','periodic']))
expected={}; edited_expected={}; quality=[]
for n,t,shape in cells:
    data=fixture(n,shape);edit=data[:n//3]+bytes(range(17))+data[n//3:]
    for c in CS:
        ends=oracle(data,t,c);expected[n,t,shape,c]=ends
        edited_expected[n,t,shape,c]=oracle(edit,t,c)
        parts=chunks(data,ends);edited=chunks(edit,edited_expected[n,t,shape,c]);original=set(parts);sizes=list(map(len,parts))
        quality.append(dict(n=n,target=t,shape=shape,candidate=c,chunks=len(ends),min_size=min(sizes),max_size=max(sizes),reused_bytes=sum(len(x) for x in edited if x in original),edited_bytes=len(edit),input_sha256=hashlib.sha256(data).hexdigest()))
(OUT/'quality.json').write_text(json.dumps(quality,indent=2)+'\n')
schedule=[]
for n,t,shape in cells:
    for c in CS:schedule.append(dict(n=n,target=t,shape=shape,candidate=c,arm=c,phase='warmup',block=-1))
    for b in range(8):
        for c in CS if b%2==0 else list(reversed(CS)):
            schedule.append(dict(n=n,target=t,shape=shape,candidate=c,arm=c,phase='measured',block=b))
for b in range(8):
    for label in ['A','B'] if b%2==0 else ['B','A']:
        schedule.append(dict(n=65536,target=256,shape='random',candidate='rolling',arm=label,phase='aa',block=b))
(OUT/'schedule.json').write_text(json.dumps(schedule,indent=2)+'\n')
failures=0
with (OUT/'results.jsonl').open('w') as stream:
    for index,row in enumerate(schedule):
        args=['taskset','-c',str(cpu),str(OUT/'chunk'),row['candidate'],str(row['n']),str(row['target']),row['shape']]
        try:
            proc=subprocess.run(args,cwd=ROOT,text=True,capture_output=True,timeout=TIMEOUT)
            rec: dict=dict(row,index=index,returncode=proc.returncode,stdout=proc.stdout,stderr=proc.stderr,oracle_ok=False)
            if proc.returncode==0:
                try:
                    value=json.loads(proc.stdout)
                    if not isinstance(value,dict):
                        raise TypeError('result is not a JSON object')
                    key=(row['n'],row['target'],row['shape'],row['candidate'])
                    ns_per_operation=value['ns_total']/value['iterations']
                    oracle_ok=value['ends']==expected[key] and value['edited_ends']==edited_expected[key]
                    rec.update(ns_total=value['ns_total'],iterations=value['iterations'],ends=value['ends'],edited_ends=value['edited_ends'],ns_per_operation=ns_per_operation,oracle_ok=oracle_ok)
                except (ValueError,KeyError,TypeError,ZeroDivisionError) as malformed:
                    rec.update(returncode=None,error=repr(malformed))
        except subprocess.TimeoutExpired as expired:
            # TimeoutExpired carries the raw bytes captured before the kill, even in text mode.
            partial={k:(v.decode('utf-8','replace') if isinstance(v,bytes) else v) for k,v in (('stdout',expired.stdout),('stderr',expired.stderr))}
            rec=dict(row,index=index,returncode=None,error=repr(expired),oracle_ok=False,**partial)
        failures+=not (rec['returncode']==0 and rec['oracle_ok'])
        stream.write(json.dumps(rec)+'\n');stream.flush()
assert manifest=={p:sha(ROOT/p) for p in inputs}
(OUT/'complete.json').write_text(json.dumps({'processes':len(schedule),'failures':failures,'inputs':manifest,'binary_sha256':sha(OUT/'chunk'),'end_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())},indent=2)+'\n')
(OUT/'manifest.json').write_text(json.dumps({p.name:sha(p) for p in sorted(OUT.iterdir()) if p.name!='manifest.json'},indent=2)+'\n')
print(json.dumps({'processes':len(schedule),'failures':failures,'output':str(OUT)}))
