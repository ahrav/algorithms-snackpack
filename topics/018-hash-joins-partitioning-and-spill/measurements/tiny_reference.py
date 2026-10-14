"""Fixed eight AB/BA process blocks; nested reference only admitted for tiny cell."""
from pathlib import Path
import argparse,subprocess,json,hashlib,platform,os,re,statistics
P=Path(__file__).resolve().parents[1]
def record(out,attempt):
 with (out/'attempts.jsonl').open('a') as f:
  f.write(json.dumps(attempt)+'\n')
def invoke(cmd,out,timeout=120):
 try:
  r=subprocess.run(cmd,text=True,capture_output=True,timeout=timeout)
 except subprocess.TimeoutExpired as e:
  record(out,{'argv':cmd,'exit':'timeout','stdout':str(e.stdout),'stderr':str(e.stderr)})
  raise
 record(out,{'argv':cmd,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
 assert r.returncode==0,r.stderr
 return r.stdout
def main():
 ap=argparse.ArgumentParser()
 ap.add_argument('--out',required=True)
 a=ap.parse_args()
 out=Path(a.out)
 out.mkdir(parents=True,exist_ok=False)
 files=['src/lib.rs','benches/tiny_reference.rs','measurements/tiny_reference.py']
 manifest={'inputs':{f:hashlib.sha256((P/f).read_bytes()).hexdigest() for f in files},'hostname':platform.node(),'arch':platform.machine(),'kernel':invoke(['uname','-a'],out),'compiler':invoke(['rustc','-vV'],out),'features':invoke(['rustc','--print','cfg','-C','target-cpu=native'],out),'cpu':invoke(['lscpu'],out) if Path('/proc/cpuinfo').exists() else 'local','available_cpus':len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),'flags':'Rust2024 -Copt-level=3 -Ctarget-cpu=native','boundary':'complete tiny resident count join, two warmups,100000 timed repeats; allocation/sort/drop/assert included; generation/oracle/startup excluded. Eight AB/BA fresh process blocks; means are units. Winner >5% allblocks; ranges descriptive. Inputs8/12 over17keys match primarytiny. Nested unmeasured largercells due quadratic cost; no claim best among5 outsidetiny.'}
 invoke(['rustc','--edition=2024','--crate-name','hash_joins','--crate-type','rlib','-C','opt-level=3','-C','target-cpu=native',str(P/'src/lib.rs'),'-o',str(out/'libhash_joins.rlib')],out)
 cmd=['rustc','--edition=2024','-C','opt-level=3','-C','target-cpu=native','--extern',f'hash_joins={out}/libhash_joins.rlib',str(P/'benches/tiny_reference.rs')]
 invoke(cmd+['-o',str(out/'tiny')],out)
 invoke(cmd+['--emit=asm','-o',str(out/'tiny.s')],out)
 manifest['binary_sha256']=hashlib.sha256((out/'tiny').read_bytes()).hexdigest()
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
 rows=[]
 for block in range(8):
  for pos,c in enumerate(['nested','merge'] if block%2==0 else ['merge','nested']):
   s=invoke([str(out/'tiny'),c],out)
   f=dict(re.findall(r'(\w+)=(\d+)',s))
   rows.append({'block':block,'position':pos,'candidate':c,'ns':int(f['ns'])/int(f['operations'])})
   (out/'rows.json').write_text(json.dumps(rows,indent=2))
 vals={c:[r['ns'] for r in rows if r['candidate']==c] for c in ['nested','merge']}
 winner='nested' if all(x*1.05<y for x,y in zip(vals['nested'],vals['merge'])) else 'merge' if all(y*1.05<x for x,y in zip(vals['nested'],vals['merge'])) else 'unresolved'
 summary={'winner':winner,'candidates':{c:{'median_ns':statistics.median(v),'min_ns':min(v),'max_ns':max(v)} for c,v in vals.items()}}
 (out/'summary.json').write_text(json.dumps(summary,indent=2))
 print(json.dumps(summary))
if __name__=='__main__':
 main()
