#!/usr/bin/env python3
"""Frozen six-block decode screen; run beside lib.rs and experiment.rs."""
import hashlib,json,os,pathlib,platform,shutil,subprocess,tempfile,time
source=pathlib.Path(__file__).resolve().parent
root=pathlib.Path(tempfile.mkdtemp(prefix='topic044-run-'))
lib=source/'lib.rs' if (source/'lib.rs').exists() else source.parent/'src/lib.rs'
example=source/'experiment.rs' if (source/'experiment.rs').exists() else source.parent/'examples/experiment.rs'
for src,name in [(lib,'lib.rs'),(example,'experiment.rs'),(source/'run.py','run.py')]: shutil.copyfile(src,root/name)
os.chdir(root)
print('Evidence directory: '+str(root),flush=True)
def call(args): return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
flags=['--edition=2024','-C','opt-level=3','-C','target-cpu=native']
subprocess.run(['rustc',*flags,'--crate-name','topic044','--crate-type','rlib','lib.rs','-o','libtopic044.rlib'],check=True)
subprocess.run(['rustc',*flags,'experiment.rs','--extern','topic044=libtopic044.rlib','-o','experiment'],check=True)
subprocess.run(['rustc',*flags,'--test','lib.rs','-o','contracts'],check=True)
contracts=call(['./contracts'])
example_result=call(['./experiment','example'])
# Independent Python division/modulo encoder and arithmetic decoder validate all workload bytes.
def encode(values):
    result=[]
    for x in values:
        while x>=128:
            result.append(x%128+128); x//=128
        result.append(x)
    return result
for case in range(8):
    row=json.loads(call(['./experiment','dump',str(case)]))
    assert encode(row['values'])==row['encoded']
    assert list(b''.join(x.to_bytes(4,'little') for x in row['values']))==row['fixed']
    assert row['decoded']==row['values']
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':call(['uname','-a']),'cpu':call(['lscpu']),'available_cpus':len(os.sched_getaffinity(0)),'affinity':sorted(os.sched_getaffinity(0)),'toolchain':call(['rustc','-vV']),'features':call(['rustc','--print','cfg','-C','target-cpu=native']),'flags':flags,'inputs':{p:sha(p) for p in ['lib.rs','experiment.rs','run.py']},'binary_sha256':sha('experiment'),'contracts':contracts,'example':example_result,'python_oracle':'all eight complete workloads pass','started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
pathlib.Path('environment.json').write_text(json.dumps(env,indent=2)+'\n')
cpu=min(os.sched_getaffinity(0))
with open('raw.jsonl','w') as out:
    for case in range(8):
        for block in range(-1,6):
            base=['scalar','unrolled','fixed']
            order=base[block%3:]+base[:block%3]
            if block>=3: order=list(reversed(order))
            for position,candidate in enumerate(order):
                row=json.loads(call(['taskset','-c',str(cpu),'./experiment',candidate,str(case)]))
                row.update(block=block,position=position,warmup=block<0,cpu=cpu)
                out.write(json.dumps(row)+'\n');out.flush()
    for block in range(6):
        for label in (['A','B'] if block%2==0 else ['B','A']):
            row=json.loads(call(['taskset','-c',str(cpu),'./experiment','scalar','6']))
            row.update(block=block,label=label,control='AA',warmup=False,cpu=cpu)
            out.write(json.dumps(row)+'\n');out.flush()
print(json.dumps({'status':'complete','records':180,'raw_sha256':sha('raw.jsonl'),'binary_sha256':sha('experiment')}))
