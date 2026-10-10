"""Frozen candidate-per-process campaign. Raw evidence stays outside Git."""
import hashlib,itertools,json,os,pathlib,platform,subprocess,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
os.chdir(ROOT)
RAW=ROOT/'raw'; RAW.mkdir(exist_ok=False)
FILES=['src/lib.rs','benches/metadata.rs','tests/contracts.rs','examples/metadata.rs','measurements/runner.py']
source={p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest() for p in FILES}
flags=['--edition','2024','-C','opt-level=3','-C','target-cpu=native']
workloads=['tiny','small_hit','small_miss','wide_miss','large87','bad_tag','cluster','deleted','build_probe','first_batch']
orders=list(itertools.permutations(['unfiltered','scalar','word']))*2
available=None; affinity=None
if hasattr(os,'sched_getaffinity'):
 available=sorted(os.sched_getaffinity(0));affinity=available[0];os.sched_setaffinity(0,{affinity})
def run(args):
 p=subprocess.run(args,text=True,capture_output=True); assert p.returncode==0,(args,p.stdout,p.stderr); return p.stdout
cpu=run(['lscpu']) if platform.system()=='Linux' else run(['sysctl','-n','machdep.cpu.brand_string'])
identity={'source':source,'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'toolchain':run(['rustc','-vV']),'cpu':cpu,'flags':flags,'target_cfg':run(['rustc','--print','cfg',*flags]),'available_cpus':available,'logical_cpus':os.cpu_count(),'affinity_cpu':affinity,'workloads':workloads,'orders':orders,'process_blocks':12,'warmup_batches':2,'selection':'lowest median; unique only when every paired rival/winner time ratio exceeds1.05 in all12 blocks; otherwise unresolved. Ranges are dispersion, not confidence intervals.','boundary':'prehashed queries, input/table preparation and independent oracle excluded; checksum included; final teardown excluded for steady lookup. build_probe includes allocation, inserts, probes, checksum, drop. first_batch has no warmup batches, but the oracle pass and the untimed diagnostics pass have each already run every query against the table; neither startup nor cold cache.','limitations':'no A/A calibration, forced cold cache, PMU or ISA attribution; local macOS has no pinned affinity'}
(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
extern=['--extern','swisstable_metadata_filtering=raw/libswisstable_metadata_filtering.rlib']
commands=[['rustc',*flags,'--test','src/lib.rs','-o','raw/lib-tests'],['rustc',*flags,'--crate-name','swisstable_metadata_filtering','--crate-type','rlib','src/lib.rs','-o','raw/libswisstable_metadata_filtering.rlib'],['rustc',*flags,'--test','tests/contracts.rs',*extern,'-o','raw/contracts'],['rustc',*flags,'examples/metadata.rs',*extern,'-o','raw/example'],['rustc',*flags,'benches/metadata.rs',*extern,'-o','raw/metadata'],['raw/lib-tests'],['raw/contracts'],['raw/example']]
logs=[{'argv':args,'stdout':run(args)} for args in commands]
(RAW/'correctness.json').write_text(json.dumps(logs,indent=2))
run(['rustc',*flags,'--crate-name','swisstable_metadata_filtering','--crate-type','rlib','--emit=asm','src/lib.rs','-o','raw/library.s'])
identity['binary_sha256']=hashlib.sha256((RAW/'metadata').read_bytes()).hexdigest()
(RAW/'identity.json').write_text(json.dumps(identity,indent=2))
records=[]
for block,order in enumerate(orders):
 for offset in range(len(workloads)):
  workload=workloads[(offset+block)%len(workloads)]
  for candidate in order:
   start=time.monotonic_ns();record=json.loads(run(['raw/metadata',workload,candidate]));record.update(block=block,order=list(order),process_elapsed_ns=time.monotonic_ns()-start)
   assert record['oracle'] and record['ns']>0
   records.append(record)
   with (RAW/'samples.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
(RAW/'completion.json').write_text(json.dumps({'source':source,'processes':len(records),'all_oracles':True},indent=2))
print(json.dumps({'processes':len(records),'all_oracles':True,'hostname':identity['hostname']}))
