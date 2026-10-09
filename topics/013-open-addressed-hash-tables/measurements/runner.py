"""Frozen 12 balanced process blocks; one candidate/workload per process."""
import hashlib,itertools,json,os,pathlib,platform,subprocess,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RAW=ROOT/'raw'; RAW.mkdir(exist_ok=False)
FILES=['src/lib.rs','benches/probe.rs','tests/contracts.rs','examples/probe.rs','measurements/runner.py']
source={p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest() for p in FILES}
flags=['--edition','2024','-C','opt-level=3','-C','target-cpu=native','-A','dead_code']
workloads=['tiny','clean25','clean75','clean87','churn_read','mixed25','mixed75','cluster','build','first_lookup']
orders=list(itertools.permutations(['lazy','rebuild','shift']))*2
available=sorted(os.sched_getaffinity(0)); os.sched_setaffinity(0,{available[0]})
def run(args):
 p=subprocess.run(args,text=True,capture_output=True); assert p.returncode==0,(args,p.stdout,p.stderr); return p.stdout
identity={'source':source,'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'toolchain':run(['rustc','-vV']),'cpu':run(['lscpu']),'flags':flags,'target_cfg':run(['rustc','--print','cfg','-C','target-cpu=native']),'available_cpus':available,'affinity_cpu':available[0],'workloads':workloads,'orders':orders,'process_blocks':12,'warmup':2,'selection':'lowest median; unique only when every paired rival/winner ratio >1.05 across all12 blocks; otherwise unresolved; ranges not confidence intervals','boundary':'lookup/mixed excludes prepared table,input,oracle,startup and final teardown; includes operation result checksum and mutation maintenance; build includes new allocation,growth,checksum,drop; first_lookup skips warmup and is not a startup/cold-cache measurement'}
(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
logs=[]
for args in [['rustc',*flags,'--test','src/lib.rs','-o','raw/lib-tests'],['rustc',*flags,'--crate-name','open_addressed_hash_tables','--crate-type','rlib','src/lib.rs','-o','raw/libopen_addressed_hash_tables.rlib'],['rustc',*flags,'--test','tests/contracts.rs','--extern','open_addressed_hash_tables=raw/libopen_addressed_hash_tables.rlib','-o','raw/contracts'],['rustc',*flags,'examples/probe.rs','--extern','open_addressed_hash_tables=raw/libopen_addressed_hash_tables.rlib','-o','raw/example'],['rustc',*flags,'benches/probe.rs','-o','raw/probe'],['raw/lib-tests'],['raw/contracts'],['raw/example']]:
 logs.append({'argv':args,'stdout':run(args)})
(RAW/'correctness.json').write_text(json.dumps(logs,indent=2))
(RAW/'probe.s').write_text(run(['objdump','-d','raw/probe']))
identity['binary_sha256']=hashlib.sha256((RAW/'probe').read_bytes()).hexdigest()
(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
records=[]
for block,order in enumerate(orders):
 for offset in range(len(workloads)):
  workload=workloads[(offset+block)%len(workloads)]
  for candidate in order:
   start=time.monotonic_ns(); record=json.loads(run(['raw/probe',workload,candidate])); record.update(block=block,order=list(order),process_elapsed_ns=time.monotonic_ns()-start)
   assert record['oracle'] and record['ns']>0
   records.append(record)
   with (RAW/'samples.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
(RAW/'completion.json').write_text(json.dumps({'source':source,'processes':len(records),'all_oracles':True},indent=2))
print(json.dumps({'processes':len(records),'all_oracles':True,'hostname':identity['hostname']}))
