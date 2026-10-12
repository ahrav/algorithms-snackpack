#!/usr/bin/env python3
"""Frozen standalone compiler/oracle and balanced process campaign; raw stays external."""
import argparse,datetime,hashlib,json,os,pathlib,platform,re,subprocess,statistics
P=pathlib.Path(__file__).resolve().parents[1]
FILES=['src/lib.rs','benches/portfolio.rs','examples/demo.rs','measurements/runner.py','measurements/analyze.py','tests/contracts.rs']
CASES=['tiny_hit','small_hit','small_miss','hot','large_hit','large_miss','high_hit','high_miss','build35','build80','churn']
CANDIDATES=['linear','cuckoo1','cuckoo4']
TIMEOUT=600
ATTEMPTS=None
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def text_of(b):return b.decode(errors='replace') if isinstance(b,bytes) else b or ''
def invoke(cmd,out=None):
 try: r=subprocess.run(cmd,text=True,capture_output=True,check=False,timeout=TIMEOUT); rc,so,se=r.returncode,r.stdout,r.stderr
 except subprocess.TimeoutExpired as t: rc,so,se='timeout',text_of(t.stdout),text_of(t.stderr)
 with ATTEMPTS.open('a') as f:f.write(json.dumps({'argv':[str(c) for c in cmd],'returncode':rc,'timeout_s':TIMEOUT,'stdout':so,'stderr':se})+'\n')
 if out:out.write_text(so+se)
 assert rc==0,(cmd,rc,se)
 return so

def main():
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);a.add_argument('--pairs',type=int,default=12);a.add_argument('--check-only',action='store_true');args=a.parse_args()
 global ATTEMPTS
 out=pathlib.Path(args.out);out.mkdir(parents=True,exist_ok=False);ATTEMPTS=out/'attempts.jsonl'
 manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':{f:sha(P/f) for f in FILES},'hostname':platform.node(),'arch':platform.machine(),'kernel':platform.platform(),'available_cpus':len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),'compiler':invoke(['rustc','-vV']),'flags':['--edition','2024','-C','opt-level=3','-C','target-cpu=native'],'cases':CASES,'candidates':CANDIDATES,'pairs':args.pairs,'attempt_timeout_s':TIMEOUT,'criterion':'Candidate must beat all other admissible candidates by >5% in every paired process block. Otherwise unresolved among contenders. Descriptive observed ranges only, no confidence interval.'}
 for cmd in [['uname','-a'],['rustc','--print','cfg','-C','target-cpu=native']]:manifest[' '.join(cmd)]=invoke(cmd)
 if pathlib.Path('/proc/cpuinfo').exists():
  manifest['lscpu']=invoke(['lscpu'])
  text=pathlib.Path('/proc/cpuinfo').read_text();manifest['cpu_model']=next((x for x in text.splitlines() if x.startswith(('model name','CPU part'))),'unknown')
 else:manifest['cpu_model']=invoke(['sysctl','-n','machdep.cpu.brand_string'])
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
 invoke(['rustc','--edition','2024','--test',str(P/'src/lib.rs'),'-o',str(out/'tests')],out/'compile-tests.log');invoke([str(out/'tests')],out/'tests.log')
 invoke(['rustc','--edition','2024','--crate-name','cuckoo_hashing','--crate-type','rlib','-C','opt-level=3','-C','target-cpu=native',str(P/'src/lib.rs'),'-o',str(out/'libcuckoo_hashing.rlib')],out/'compile-lib.log')
 common=['rustc','--edition','2024','-C','opt-level=3','-C','target-cpu=native','--extern',f'cuckoo_hashing={out}/libcuckoo_hashing.rlib']
 invoke(common+['--test',str(P/'tests/contracts.rs'),'-o',str(out/'contracts')],out/'compile-contracts.log');invoke([str(out/'contracts')],out/'contracts.log')
 invoke(common+[str(P/'examples/demo.rs'),'-o',str(out/'demo')],out/'compile-demo.log');invoke([str(out/'demo')],out/'demo.log')
 if args.check_only:return
 invoke(common+[str(P/'benches/portfolio.rs'),'-o',str(out/'portfolio')],out/'compile-bench.log')
 invoke(common+['--emit=asm',str(P/'benches/portfolio.rs'),'-o',str(out/'portfolio.s')],out/'compile-asm.log')
 manifest['binary_sha256']=sha(out/'portfolio');(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
 rows=[]
 for case in CASES:
  for block in range(args.pairs):
   # Three-way rotations plus their reversals: each candidate occupies each position.
   orders=[CANDIDATES[i:]+CANDIDATES[:i] for i in range(3)];orders+=list(reversed(x) for x in orders)
   for pos,candidate in enumerate(orders[block%6]):
    s=invoke([str(out/'portfolio'),candidate,case]);(out/f'{case}-{block}-{pos}-{candidate}.txt').write_text(s)
    row={'case':case,'block':block,'position':pos,'candidate':candidate,'stdout':s.strip()}
    if s.startswith('OK '):
     fields=dict(re.findall(r'(\w+)=([\d.]+)',s));row.update(status='ok',ns_per_operation=float(fields['ns'])/int(fields['operations']),mean_slots=float(fields['mean_slots']),max_slots=int(fields['max_slots']),bytes_slots=int(fields['bytes_slots']))
    else:row.update(status='inadmissible')
    rows.append(row);(out/'rows.json').write_text(json.dumps(rows,indent=2))
  print(case,'complete',flush=True)
 from analyze import summarize
 summary=summarize(rows,args.pairs)
 (out/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary),flush=True)
if __name__=='__main__':main()
