#!/usr/bin/env python3
"""Frozen standalone compiler/oracle and balanced process campaign; raw stays external."""
import argparse,datetime,hashlib,json,os,pathlib,platform,re,subprocess,statistics
P=pathlib.Path(__file__).resolve().parents[1]
FILES=['src/lib.rs','benches/portfolio.rs','examples/demo.rs','measurements/runner.py','measurements/analyze.py','tests/contracts.rs']
CASES=['tiny_miss','mid_miss','mid_hit90','large_miss','large_hit90','hot_miss','build_mid','first_batch']
CANDIDATES=['exact','bloom','blocked','cuckoo','quotient']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
ATTEMPT_ROOT=None

def invoke(cmd,out=None):
 try:
  r=subprocess.run(cmd,text=True,capture_output=True,check=False,timeout=600)
  attempt={'argv':cmd,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 except subprocess.TimeoutExpired as e:
  attempt={'argv':cmd,'exit':'timeout','stdout':str(e.stdout),'stderr':str(e.stderr)}
  if ATTEMPT_ROOT:
   with (ATTEMPT_ROOT/'attempts.jsonl').open('a') as f:f.write(json.dumps(attempt)+'\n')
  raise
 if ATTEMPT_ROOT:
  with (ATTEMPT_ROOT/'attempts.jsonl').open('a') as f:f.write(json.dumps(attempt)+'\n')
 if out:out.write_text(r.stdout+r.stderr)
 assert r.returncode==0,(cmd,r.returncode,r.stderr)
 return r.stdout

def main():
 global ATTEMPT_ROOT
 a=argparse.ArgumentParser();a.add_argument('--out',required=True);a.add_argument('--pairs',type=int,default=10);a.add_argument('--check-only',action='store_true');args=a.parse_args()
 out=pathlib.Path(args.out);out.mkdir(parents=True,exist_ok=False);ATTEMPT_ROOT=out
 manifest={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':{f:sha(P/f) for f in FILES},'hostname':platform.node(),'arch':platform.machine(),'kernel':platform.platform(),'available_cpus':len(os.sched_getaffinity(0)) if hasattr(os,'sched_getaffinity') else os.cpu_count(),'compiler':invoke(['rustc','-vV']),'flags':['--edition','2024','-C','opt-level=3','-C','target-cpu=native'],'cases':CASES,'candidates':CANDIDATES,'pairs':args.pairs,'criterion':'Candidate must beat all other admissible candidates by >5% in every paired process block. Otherwise unresolved among contenders. Descriptive observed ranges only, no confidence interval.'}
 for cmd in [['uname','-a'],['rustc','--print','cfg','-C','target-cpu=native']]:manifest[' '.join(cmd)]=invoke(cmd)
 if pathlib.Path('/proc/cpuinfo').exists():
  manifest['lscpu']=invoke(['lscpu'])
  text=pathlib.Path('/proc/cpuinfo').read_text();manifest['cpu_model']=next((x for x in text.splitlines() if x.startswith(('model name','CPU part'))),'unknown')
 else:manifest['cpu_model']=invoke(['sysctl','-n','machdep.cpu.brand_string'])
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
 invoke(['rustc','--edition','2024','--test',str(P/'src/lib.rs'),'-o',str(out/'tests')],out/'compile-tests.log');invoke([str(out/'tests')],out/'tests.log')
 invoke(['rustc','--edition','2024','--crate-name','membership_filters','--crate-type','rlib','-C','opt-level=3','-C','target-cpu=native',str(P/'src/lib.rs'),'-o',str(out/'libmembership_filters.rlib')],out/'compile-lib.log')
 common=['rustc','--edition','2024','-C','opt-level=3','-C','target-cpu=native','--extern',f'membership_filters={out}/libmembership_filters.rlib']
 invoke(common+['--test',str(P/'tests/contracts.rs'),'-o',str(out/'contracts')],out/'compile-contracts.log');invoke([str(out/'contracts')],out/'contracts.log')
 invoke(common+[str(P/'examples/demo.rs'),'-o',str(out/'demo')],out/'compile-demo.log');invoke([str(out/'demo')],out/'demo.log')
 if args.check_only:return
 invoke(common+[str(P/'benches/portfolio.rs'),'-o',str(out/'portfolio')],out/'compile-bench.log')
 invoke(common+['--emit=asm',str(P/'benches/portfolio.rs'),'-o',str(out/'portfolio.s')],out/'compile-asm.log')
 manifest['binary_sha256']=sha(out/'portfolio');(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
 rows=[]
 for case in CASES:
  for block in range(args.pairs):
   # Five rotations plus their reversals balance positions over ten process blocks.
   orders=[CANDIDATES[i:]+CANDIDATES[:i] for i in range(5)];orders+=list(reversed(x) for x in orders)
   for pos,candidate in enumerate(orders[block%10]):
    s=invoke([str(out/'portfolio'),candidate,case]);(out/f'{case}-{block}-{pos}-{candidate}.txt').write_text(s)
    row={'case':case,'block':block,'position':pos,'candidate':candidate,'stdout':s.strip()}
    if s.startswith('OK '):
     fields=dict(re.findall(r'(\w+)=([\d.]+)',s));row.update(status='ok',ns_per_operation=float(fields['ns'])/int(fields['operations']),bytes_payload=int(fields['bytes']),fp=int(fields['fp']),negatives=int(fields['negatives']),checks=int(fields['checks']))
    else:row.update(status='inadmissible')
    rows.append(row);(out/'rows.json').write_text(json.dumps(rows,indent=2))
  print(case,'complete',flush=True)
 from analyze import summarize
 summary=summarize(rows,args.pairs)
 (out/'summary.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary),flush=True)
if __name__=='__main__':main()
