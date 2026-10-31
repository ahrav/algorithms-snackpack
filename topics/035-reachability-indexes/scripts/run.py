#!/usr/bin/env python3
"""Fixed six-block, all-permutations fresh-process experiment."""
import hashlib
import itertools
import json
import pathlib
import platform
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[1]
if len(sys.argv) != 2 or sys.argv[1].startswith('-'):
    sys.exit('usage: run.py NEW_OUTPUT_DIR')
out = pathlib.Path(sys.argv[1]).resolve()
out.mkdir(parents=True, exist_ok=False)

def command(args):
    return subprocess.check_output(args, text=True).strip()

inputs = [root / p for p in ['src/lib.rs', 'benches/reachability.rs', 'tests/contracts.rs', 'examples/running.rs', 'scripts/run.py']]
def hashes():
    return {str(p.relative_to(root)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
identity = hashes()
environment = {'hostname': platform.node(), 'architecture': platform.machine(), 'uname': command(['uname', '-a']), 'cpu': command(['sh', '-c', 'LC_ALL=C lscpu']), 'rustc': command(['rustc', '-Vv']), 'cpus': command(['nproc']), 'affinity': command(['sh','-c', 'taskset -pc $$']), 'flags': '-C opt-level=3 --edition=2024; generic target; no target-cpu=native', 'inputs': identity}
(out/'environment.json').write_text(json.dumps(environment,indent=2)+'\n')
subprocess.run(['rustc','--edition=2024','--crate-type=rlib','--crate-name=topic035','-C','opt-level=3',str(root/'src/lib.rs'),'-o',str(out/'libtopic035.rlib')],check=True)
binary=out/'bench'
subprocess.run(['rustc','--edition=2024','-C','opt-level=3',str(root/'benches/reachability.rs'),'--extern',f'topic035={out}/libtopic035.rlib','-o',str(binary)],check=True)
environment['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
(out/'environment.json').write_text(json.dumps(environment,indent=2)+'\n')
for source,name in [('tests/contracts.rs','tests'),('examples/running.rs','example')]:
    args=['rustc','--edition=2024',str(root/source),'--extern',f'topic035={out}/libtopic035.rlib','-o',str(out/name)]
    if name=='tests':args.append('--test')
    subprocess.run(args,check=True)
    (out/(name+'.txt')).write_text(command([str(out/name)]))
with (out/'samples.jsonl').open('w') as f:
    for block,order in enumerate(itertools.permutations(['dfs','filter','closure'])):
        for workload in ['tiny','sparse','dense','chain','negative','few','large-sparse']:
            for position,candidate in enumerate(order):
                row=json.loads(command([str(binary),candidate,workload]));row.update(block=block,position=position)
                f.write(json.dumps(row)+'\n');f.flush()
        for candidate in ['dfs','dfs-control'] if block%2==0 else ['dfs-control','dfs']:
            row=json.loads(command([str(binary),candidate,'sparse']));row.update(block=block,position=-1)
            f.write(json.dumps(row)+'\n');f.flush()
if hashes() != identity:
    sys.exit('measured inputs changed during the campaign; environment.json names the pre-change bytes')
print(json.dumps({'records':138,'output':str(out)}))
