#!/usr/bin/env python3
"""Frozen six-block exploratory screen. Run beside lib.rs and experiment.rs."""
import hashlib,json,os,pathlib,platform,subprocess,time,tempfile,shutil
source=pathlib.Path(__file__).resolve().parent
root=pathlib.Path(tempfile.mkdtemp(prefix='topic043-run-'))
lib=source/'lib.rs' if (source/'lib.rs').exists() else source.parent/'src/lib.rs'
experiment=source/'experiment.rs' if (source/'experiment.rs').exists() else source.parent/'examples/experiment.rs'
for src,name in [(lib,'lib.rs'),(experiment,'experiment.rs'),(source/'run.py','run.py')]: shutil.copyfile(src,root/name)
os.chdir(root)
print('Evidence directory: '+str(root),flush=True)
def call(args): return subprocess.check_output(args,text=True,stderr=subprocess.STDOUT)
TIMEOUT=float(os.environ.get('TOPIC_043_PROCESS_TIMEOUT_S',60))
REQUIRED={'candidate','case','endpoint','n','r','states','table_bytes','matches','reps','elapsed_ns','checksum'}
def measure(out,args,**meta):
    row: dict = {'status': 'ok'}
    def text(captured): return (captured or b'').decode(errors='replace')
    try:
        output = text(subprocess.check_output(args, stderr=subprocess.STDOUT, timeout=TIMEOUT))
    except subprocess.CalledProcessError as failed:
        row.update(status='failed', returncode=failed.returncode, output=text(failed.output))
    except subprocess.TimeoutExpired as expired:
        row.update(status='timeout', timeout_s=TIMEOUT, output=text(expired.output))
    else:
        try:
            decoded = json.loads(output)
            if not isinstance(decoded, dict) or not decoded.keys() >= REQUIRED:
                raise ValueError('measurement object missing ' + ','.join(sorted(REQUIRED)))
            invocation = {k: meta[k] for k in ('candidate', 'case', 'endpoint')}
            printed = {k: decoded[k] for k in invocation}
            if printed != invocation:
                raise ValueError('measurement labels ' + json.dumps(printed) + ' differ from invocation ' + json.dumps(invocation))
            row.update(decoded)
        except ValueError as malformed:
            row.update(status='malformed', output=output, error=repr(malformed))
    row.update(args=args, **meta)
    out.write(json.dumps(row) + '\n')
    out.flush()
    if row['status'] != 'ok':
        raise SystemExit('attempt ' + row['status'] + '; retained in raw.jsonl: ' + json.dumps(row))
# Rebuild exact byte-identified inputs on the executing host.
flags=['--edition=2024','-C','opt-level=3','-C','target-cpu=native']
subprocess.run(['rustc',*flags,'--crate-name','topic043','--crate-type','rlib','lib.rs','-o','libtopic043.rlib'],check=True)
subprocess.run(['rustc',*flags,'experiment.rs','--extern','topic043=libtopic043.rlib','-o','experiment'],check=True)
subprocess.run(['rustc',*flags,'--test','lib.rs','-o','contracts'],check=True)
contracts=call(['./contracts'])
example=call(['./experiment','example'])
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
env={'hostname':platform.node(),'architecture':platform.machine(),'uname':call(['uname','-a']),'cpu':call(['lscpu']),'available_cpus':len(os.sched_getaffinity(0)),'affinity':sorted(os.sched_getaffinity(0)),'toolchain':call(['rustc','-vV']),'features':call(['rustc','--print','cfg','-C','target-cpu=native']),'flags':flags,'process_timeout_s':TIMEOUT,'inputs':{p:sha(p) for p in ['lib.rs','experiment.rs','run.py']},'binary_sha256':sha('experiment'),'contracts':contracts,'example':example,'started_utc':time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())}
pathlib.Path('environment.json').write_text(json.dumps(env,indent=2)+'\n')
cpu=min(os.sched_getaffinity(0))
with open('raw.jsonl','w') as out:
    for case in range(11):
        for endpoint in ['scan','build']:
            for block in range(-1,6):
                base=['direct','sparse','dense']
                order=base[block%3:]+base[:block%3]
                if block>=3: order=list(reversed(order))
                for position,candidate in enumerate(order):
                    measure(out,['taskset','-c',str(cpu),'./experiment',candidate,str(case),endpoint],candidate=candidate,case=case,endpoint=endpoint,block=block,position=position,warmup=block<0,cpu=cpu,pid_sample='fresh process')
    # Identical binary, direct baseline at case 3, alternating labels and AB/BA order.
    for block in range(6):
        for label in (['A','B'] if block%2==0 else ['B','A']):
            measure(out,['taskset','-c',str(cpu),'./experiment','direct','3','scan'],candidate='direct',case=3,endpoint='scan',block=block,label=label,control='AA',warmup=False,cpu=cpu)
print(json.dumps({'status':'complete','records':474,'raw_sha256':sha('raw.jsonl'),'binary_sha256':sha('experiment')}))
