#!/usr/bin/env python3
"""Run a fixed balanced schedule. Retain every process aggregate and identity."""
import hashlib,itertools,json,os,pathlib,platform,subprocess,time
root=pathlib.Path(__file__).resolve().parents[1]
os.chdir(root)
out=root/'native';out.mkdir(exist_ok=True)
def run(args):
    return subprocess.run(args,check=True,text=True,capture_output=True).stdout
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
inputs={str(p.relative_to(root)):digest(p) for p in sorted(root.rglob('*')) if p.is_file() and p.suffix in ['.rs','.py','.toml','.lock'] and 'target' not in p.parts and 'native' not in p.parts}
(out/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
cpu=min(os.sched_getaffinity(0))
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':list(platform.uname()),'cpuinfo':pathlib.Path('/proc/cpuinfo').read_text(),'lscpu':run(['lscpu']),'rustc':run(['rustc','-Vv']),'cfg':run(['rustc','--print','cfg','-C','target-cpu=native']),'available_cpus':sorted(os.sched_getaffinity(0)),'pinned_cpu':cpu,'flags':['-O','-C','target-cpu=native','-C','codegen-units=1'],'started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
(out/'environment.json').write_text(json.dumps(env,indent=2)+'\n')
crate='topic038_kmp_and_prefix_function_matching'
flags=['--edition=2024','-O','-C','target-cpu=native','-C','codegen-units=1']
run(['rustc',*flags,'--crate-name',crate,'--crate-type','rlib','src/lib.rs','-o','native/libtopic038.rlib'])
run(['rustc',*flags,'--test','src/lib.rs','-o','native/unit'])
run(['rustc',*flags,'--test','tests/contracts.rs','--extern',f'{crate}=native/libtopic038.rlib','-o','native/contracts'])
run(['rustc',*flags,'benches/matching.rs','--extern',f'{crate}=native/libtopic038.rlib','-o','native/matching'])
(out/'contracts.txt').write_text(run(['native/unit'])+run(['native/contracts']))
with (out/'linked-disassembly.txt').open('w') as f:subprocess.run(['objdump','-d','native/matching'],stdout=f,check=True)
rows=[]
def attempt(case,candidate,boundary,phase,block,position):
    r=json.loads(run(['taskset','-c',str(cpu),'native/matching',case,candidate,boundary]))
    r.update(phase=phase,block=block,position=position)
    rows.append(r)
    with (out/'samples.jsonl').open('a') as f:f.write(json.dumps(r)+'\n')
cases=['tiny','random','large','near','dense','binary']
for case in cases:
    for boundary in ['total','prepared']:
        for i,c in enumerate(['direct','kmp','dfa']):attempt(case,c,boundary,'warmup',-1,i)
        for block,order in enumerate(itertools.permutations(['direct','kmp','dfa'])):
            for pos,c in enumerate(order):attempt(case,c,boundary,'measure',block,pos)
for case in ['tiny','near']:
    for i,c in enumerate(['kmp','kmp-aa']):attempt(case,c,'prepared','aa-warmup',-1,i)
    for block in range(6):
        for pos,c in enumerate(['kmp','kmp-aa'] if block%2==0 else ['kmp-aa','kmp']):attempt(case,c,'prepared','aa',block,pos)
manifest={str(p.relative_to(out)):digest(p) for p in sorted(out.iterdir()) if p.is_file()}
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'processes':len(rows),'measured':sum(r['phase']=='measure' for r in rows),'inputs':inputs,'binary_sha256':digest(out/'matching')}))
