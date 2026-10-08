#!/usr/bin/env python3
"""Frozen single-host process runner; no external Python packages required."""
import hashlib, itertools, json, os, pathlib, platform, subprocess, time

ROOT = pathlib.Path(__file__).resolve().parent.parent
WORKLOADS = [
    ("tiny", 32, 4, "uniform", 16, 1024, False),
    ("small", 4096, 4, "uniform", 16, 16, False),
    ("large16", 262144, 4, "uniform", 16, 1, False),
    ("large1024", 262144, 10, "uniform", 16, 1, False),
    ("large4096", 262144, 12, "uniform", 16, 1, False),
    ("skew90", 262144, 10, "skew", 16, 1, False),
    ("same", 262144, 10, "same", 16, 1, False),
    ("lowzero", 262144, 10, "zero", 16, 1, False),
    ("grouped", 262144, 10, "grouped", 16, 1, False),
    ("wide", 262144, 10, "uniform", 64, 1, False),
    ("first_call", 262144, 10, "uniform", 16, 1, True),
]
CANDIDATES = ["buckets", "scatter", "two_pass", "cycles"]
PERMS = list(itertools.permutations(CANDIDATES))[:6]
ORDERS = [order for p in PERMS for order in (p, tuple(reversed(p)))]
FLAGS = ["--edition", "2024", "-C", "opt-level=3", "-C", "target-cpu=native", "-A", "dead_code"]

def command(args):
    p = subprocess.run(args, cwd=ROOT, capture_output=True, text=True)
    if p.returncode: raise RuntimeError(f"{args}: {p.returncode}\n{p.stdout}\n{p.stderr}")
    return p.stdout + p.stderr

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    out = ROOT / "raw"
    out.mkdir(exist_ok=False)
    receipt = {"source": {p: digest(ROOT / p) for p in ["src/lib.rs", "benches/partition.rs", "measurements/runner.py", "tests/contracts.rs", "examples/partition.rs"]},
               "hostname": platform.node(), "architecture": platform.machine(), "uname": list(platform.uname()),
               "toolchain": command(["rustc", "-Vv"]), "flags": FLAGS,
               "cpu": command(["lscpu"]), "available_cpus": sorted(os.sched_getaffinity(0)),
               "workloads": WORKLOADS, "orders": ORDERS,
               "boundary": "candidate call including histogram/allocation/zero fill or clone, output observation and teardown; excludes input generation, grouping, warmup, independent oracle and process startup",
               "warmup_calls": 2, "process_blocks": 12,
               "selection": "lowest median among contract-valid candidates; unique only if every paired rival/winner ratio exceeds 1.05; otherwise unresolved; observed ranges have no confidence interpretation"}
    assert receipt["architecture"] in ["aarch64", "x86_64"]
    cpu = min(receipt["available_cpus"])
    receipt["affinity_cpu"] = cpu
    (out / "identity.json").write_text(json.dumps(receipt, indent=2))
    command(["rustc", "--edition", "2024", "-D", "warnings", "--test", "src/lib.rs", "-o", "checks"])
    (out / "checks.txt").write_text(command(["taskset", "-c", str(cpu), "./checks"]))
    command(["rustc", *FLAGS, "src/lib.rs", "--crate-name", "radix_partitioning", "--crate-type", "rlib", "-o", "libradix_partitioning.rlib"])
    command(["rustc", *FLAGS, "benches/partition.rs", "--extern", "radix_partitioning=libradix_partitioning.rlib", "-o", "bench"])
    command(["rustc", "--edition", "2024", "--test", "tests/contracts.rs", "--extern", "radix_partitioning=libradix_partitioning.rlib", "-o", "integration"])
    (out / "integration.txt").write_text(command(["./integration"]))
    command(["rustc", "--edition", "2024", "examples/partition.rs", "--extern", "radix_partitioning=libradix_partitioning.rlib", "-o", "example"])
    (out / "example.txt").write_text(command(["./example"]))
    receipt["binary_sha256"] = digest(ROOT / "bench")
    command(["rustc", *FLAGS, "--emit=asm", "benches/partition.rs", "--extern", "radix_partitioning=libradix_partitioning.rlib", "-o", "raw/bench.s"])
    (out / "identity.json").write_text(json.dumps(receipt, indent=2))
    with (out / "samples.jsonl").open("w") as f:
        for cell,n,bits,shape,width,reps,cold in WORKLOADS:
            for block,order in enumerate(ORDERS):
                for position,name in enumerate(order):
                    start=time.perf_counter_ns()
                    result=json.loads(command(["taskset","-c",str(cpu),"./bench",name,str(n),str(bits),shape,str(width),str(reps),str(int(cold))]))
                    result.update(cell=cell,block=block,position=position,process_ns=time.perf_counter_ns()-start)
                    f.write(json.dumps(result)+"\n"); f.flush()
            print(cell, "48 processes complete", flush=True)
    (out / "completion.json").write_text(json.dumps({"processes":528,"oracle":"all pass","source":receipt["source"]},indent=2))

if __name__ == "__main__": main()
