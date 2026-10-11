"""Frozen, balanced candidate-per-process campaign; retained raw files stay out of Git."""
import hashlib,itertools,json,os,pathlib,platform,statistics,subprocess,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RAW=ROOT/'raw'; RAW.mkdir(exist_ok=False)
FILES=['src/lib.rs','benches/portfolio.rs','tests/contracts.rs','examples/demo.rs','measurements/runner.py']
source={p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest() for p in FILES}
flags=['--edition','2024','-C','opt-level=3','-C','target-cpu=native']
workloads=['tiny','small_hit','high_hit','high_miss','high_mix','large_mix','skew','same_home','churn','build','first_batch']
orders=list(itertools.permutations(['linear','robin','scan']))*2
available=None; affinity=None
if hasattr(os,'sched_getaffinity'):
 available=sorted(os.sched_getaffinity(0));affinity=available[0];os.sched_setaffinity(0,{affinity})
def run(args):
 p=subprocess.run(args,text=True,capture_output=True); assert p.returncode==0,(args,p.stdout,p.stderr); return p.stdout
identity={'source':source,'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'toolchain':run(['rustc','-vV']),'cpu':run(['lscpu']) if platform.system()=='Linux' else run(['sysctl','-n','machdep.cpu.brand_string']),'flags':flags,'target_cfg':run(['rustc','--print','cfg','-C','target-cpu=native']),'available_cpus':available,'logical_cpus':os.cpu_count(),'affinity_cpu':affinity,'workloads':workloads,'orders':orders,'process_blocks':12,'warmup_batches':1,'selection':'lowest median; unique only when each paired rival/winner time ratio exceeds 1.03 in all12 blocks. Otherwise unresolved. Ranges are dispersion, not confidence intervals.','boundary':'hash included; query generation, initial allocation/build, BTreeMap oracle, slot-count diagnostics and final drop excluded for lookup. Churn includes remove, reinsertion, lookup. Build includes allocation, all inserts, one lookup and drop. first_batch skips warmup but oracle previously accessed a separate table, so no cold/startup claim. Checksums and black_box included.','limitations':'equal Option<Entry> layout includes usize DIB in baseline to isolate placement/lookup; all inserts presearch duplicates; fixed deterministic hasher is not attack-resistant; no A/A calibration, forced cold cache, PMU, latency quantiles, generic-payload or ISA attribution; macOS affinity unpinned'}
(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
extern=['--extern','robin_hood_hashing=raw/librobin_hood_hashing.rlib']
commands=[['rustc',*flags,'--test','src/lib.rs','-o','raw/lib-tests'],['rustc',*flags,'--crate-name','robin_hood_hashing','--crate-type','rlib','src/lib.rs','-o','raw/librobin_hood_hashing.rlib'],['rustc',*flags,'--test','tests/contracts.rs',*extern,'-o','raw/contracts'],['rustc',*flags,'examples/demo.rs',*extern,'-o','raw/example'],['rustc',*flags,'benches/portfolio.rs',*extern,'-o','raw/portfolio'],['raw/lib-tests'],['raw/contracts'],['raw/example']]
(RAW/'correctness.json').write_text(json.dumps([{'argv':a,'stdout':run(a)} for a in commands],indent=2))
run(['rustc',*flags,'--crate-name','robin_hood_hashing','--crate-type','rlib','--emit=asm','src/lib.rs','-o','raw/library.s'])
identity['binary_sha256']=hashlib.sha256((RAW/'portfolio').read_bytes()).hexdigest();(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
records=[]
for block,order in enumerate(orders):
 for offset in range(len(workloads)):
  workload=workloads[(offset+block)%len(workloads)]
  checksums=[]
  for candidate in order:
   start=time.monotonic_ns();r=json.loads(run(['raw/portfolio',candidate,workload,str(100+block)]));r.update(block=block,order=list(order),process_elapsed_ns=time.monotonic_ns()-start)
   assert r['ns']>0
   checksums.append(r['checksum']);records.append(r)
   with (RAW/'samples.jsonl').open('a') as f:f.write(json.dumps(r)+'\n')
  assert len(set(checksums))==1,(workload,checksums)
summary={}
for w in workloads:
 rows=[r for r in records if r['workload']==w]
 costs={c:[r['ns']/r['units'] for r in rows if r['candidate']==c] for c in ['linear','robin','scan']}
 med={c:statistics.median(v) for c,v in costs.items()};best=min(med,key=med.get)
 ratios={c:[v/costs[best][i] for i,v in enumerate(values)] for c,values in costs.items() if c!=best}
 unique=all(min(v)>1.03 for v in ratios.values())
 summary[w]={'median_ns_per_unit':med,'range_ns_per_unit':{c:[min(v),max(v)] for c,v in costs.items()},'median_fastest':best,'selection':best if unique else 'unresolved','paired_rival_over_fastest_range':{c:[min(v),max(v)] for c,v in ratios.items()},'probe_counts_block0':{r['candidate']:{k:r[k] for k in ['query_probes','probe_p95','probe_max','capacity','live']} for r in rows if r['block']==0}}
(RAW/'summary.json').write_text(json.dumps(summary,indent=2))
(RAW/'completion.json').write_text(json.dumps({'source':source,'processes':len(records),'all_oracles_and_checksums':True},indent=2))
print(json.dumps({'processes':len(records),'all_oracles_and_checksums':True,'hostname':identity['hostname'],'selection':{w:s['selection'] for w,s in summary.items()}}))
