#!/usr/bin/env python3
"""Fixed six-block fresh-process SCC campaign; run from the topic directory."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = Path(sys.argv[1]).resolve()
OUT.mkdir(parents=True, exist_ok=False)
FILES = ["src/lib.rs", "tests/contracts.rs", "examples/components.rs", "benches/scc.rs", "measurements/run.py"]
def call(args, **kw):
    return subprocess.run(args, check=True, text=True, capture_output=True, **kw)
def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()
hashes = {p: sha(ROOT/p) for p in FILES}
allowed = sorted(os.sched_getaffinity(0))
cpu = allowed[len(allowed)//2]
env = {"hostname": platform.node(), "architecture": platform.machine(), "uname": list(platform.uname()),
       "rustc": call(["rustc", "-vV"]).stdout, "cpus": allowed, "affinity": cpu,
       "cpuinfo": Path('/proc/cpuinfo').read_text(), "flags": "--edition=2024 -O -C target-cpu=native -C codegen-units=1",
       "target_cfg": call(["rustc", "--print", "cfg", "-C", "target-cpu=native"]).stdout,
       "source_hashes": hashes, "started": time.time()}
(OUT/'environment.json').write_text(json.dumps(env, indent=2))
flags = ["--edition=2024", "-O", "-C", "target-cpu=native", "-C", "codegen-units=1"]
lib = OUT/'libtopic033.rlib'
call(["rustc", *flags, "--crate-name", "topic033", "--crate-type", "rlib", str(ROOT/'src/lib.rs'), "-o", str(lib)])
for name, source, extra in [('contracts','tests/contracts.rs',['--test']),('demo','examples/components.rs',[]),('scc','benches/scc.rs',[])]:
    call(["rustc", *flags, *extra, str(ROOT/source), "--extern", 'topic033='+str(lib), '-o', str(OUT/name)])
    if name != 'scc':
        (OUT/(name+'.txt')).write_text(call([str(OUT/name)]).stdout)
env['binary_sha256'] = sha(OUT/'scc')
try:
    (OUT/'disassembly.txt').write_text(call(['objdump','-Cd',str(OUT/'scc')]).stdout)
except (FileNotFoundError, subprocess.CalledProcessError) as e:
    (OUT/'disassembly-unavailable.txt').write_text(str(e))
workloads = ['tiny-first','tiny','isolated','chain','cycle','groups','permuted','duplicates','dense','random']
candidates = ['tarjan','gabow','kosaraju','prepared','tarjan-aa']
with (OUT/'records.jsonl').open('w') as log:
    def run(w,c,phase,block,position):
        before = time.monotonic_ns()
        p = subprocess.run(['taskset','-c',str(cpu),str(OUT/'scc'),w,c],text=True,capture_output=True,timeout=90)
        record = {'workload':w,'candidate':c,'phase':phase,'block':block,'position':position,
                  'exit':p.returncode,'wall_ns':time.monotonic_ns()-before,'stderr':p.stderr}
        if p.returncode == 0: record.update(json.loads(p.stdout))
        else: record['stdout'] = p.stdout
        log.write(json.dumps(record)+'\n');log.flush()
        if p.returncode: raise RuntimeError(record)
    for w in workloads:
        for i,c in enumerate(candidates): run(w,c,'warmup',-1,i)
        for b in range(6):
            shift = b//2
            order = candidates[shift:]+candidates[:shift]
            if b%2: order.reverse()
            for i,c in enumerate(order): run(w,c,'measure',b,i)
        print(w, 'complete', flush=True)
assert hashes == {p:sha(ROOT/p) for p in FILES}
env['finished'] = time.time()
(OUT/'environment.json').write_text(json.dumps(env, indent=2))
(OUT/'SHA256SUMS.json').write_text(json.dumps({p.name:sha(p) for p in OUT.iterdir() if p.is_file()},indent=2))
