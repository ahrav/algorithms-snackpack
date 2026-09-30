#!/usr/bin/env python3
"""Verify a frozen archive, run native checks, and retain every process sample."""
import hashlib
import json
import os
import platform
from pathlib import Path, PurePosixPath
import subprocess
import sys
import tarfile
import time

TOPIC = "topics/006-monotonic-stacks-and-deques"
CELLS = [
    ("random", 4096, 3), ("random", 32768, 3),
    ("random", 4096, 256), ("random", 32768, 256),
    ("increasing", 32768, 256), ("decreasing", 32768, 256),
    ("equal", 32768, 256), ("burst", 32768, 256),
]

def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def main():
    archive, expected_hash, commit, arch = sys.argv[1:]
    assert platform.machine() == arch, (platform.machine(), arch)
    assert len(commit) == 40 and all(c in "0123456789abcdef" for c in commit)
    assert digest(archive) == expected_hash
    receipt = Path("receipt")
    receipt.mkdir()
    source = Path("source")
    source.mkdir()
    with tarfile.open(archive) as bundle:
        for member in bundle:
            path = PurePosixPath(member.name)
            assert not path.is_absolute() and ".." not in path.parts
            allowed = member.name in ("Cargo.toml", "Cargo.lock") or member.name == "topics" or member.name == TOPIC or member.name.startswith(TOPIC + "/")
            assert allowed and (member.isdir() or member.isfile()), member.name
            target = source / member.name
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(bundle.extractfile(member).read())
    files = {str(p.relative_to(source)): digest(p) for p in source.rglob("*") if p.is_file()}
    (receipt / "source.json").write_text(json.dumps({"commit": commit, "archive_sha256": expected_hash, "runner_sha256": digest(__file__), "files": files}, indent=2))
    affinity = sorted(os.sched_getaffinity(0))
    cpu = affinity[0]
    host = {"hostname": platform.node(), "architecture": platform.machine(), "uname": list(platform.uname()), "cpu_count": os.cpu_count(), "available_cpu_count": len(affinity), "affinity": affinity, "measurement_cpu": cpu, "flags": ["--edition=2024", "-Copt-level=3", "-Cdebuginfo=1", "-Dwarnings", "-Funsafe-code"], "target_cpu": "compiler default", "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
    (receipt / "host.json").write_text(json.dumps(host, indent=2))
    def run(name, command, required=True):
        started = time.monotonic_ns()
        result = subprocess.run(command, capture_output=True, text=True, timeout=180)
        (receipt / (name + ".stdout")).write_text(result.stdout)
        (receipt / (name + ".stderr")).write_text(result.stderr)
        (receipt / (name + ".status.json")).write_text(json.dumps({"command": command, "returncode": result.returncode, "process_wall_ns": time.monotonic_ns() - started}))
        if required:
            assert result.returncode == 0, (name, result.stderr)
        return result
    run("cpu", ["lscpu"])
    run("compiler", ["rustc", "-Vv"])
    run("features", ["rustc", "--print", "cfg"])
    flags = ["--edition=2024", "-Copt-level=3", "-Cdebuginfo=1", "-Dwarnings", "-Funsafe-code"]
    root = source / TOPIC
    library = str(receipt / "libmonotonic_stacks_deques.rlib")
    run("compile-library", ["rustc", *flags, "--crate-type=rlib", "--crate-name=monotonic_stacks_deques", str(root / "src/lib.rs"), "-o", library])
    run("assembly", ["rustc", *flags, "--crate-type=rlib", "--crate-name=monotonic_stacks_deques", "--emit=asm", str(root / "src/lib.rs"), "-o", str(receipt / "library.s")])
    external = ["--extern", "monotonic_stacks_deques=" + library]
    run("compile-tests", ["rustc", *flags, "--test", str(root / "tests/contracts.rs"), *external, "-o", str(receipt / "tests")])
    run("tests", [str(receipt / "tests")])
    run("doctests", ["rustdoc", "--edition=2024", "--test", str(root / "src/lib.rs"), *external])
    binary = str(receipt / "experiment")
    run("compile-experiment", ["rustc", *flags, str(root / "examples/experiment.rs"), *external, "-o", binary])
    run("example", [binary])
    run("symbols", ["nm", "-C", binary])
    samples = receipt / "samples.jsonl"
    with samples.open("w") as output:
        for cell_index, (shape, n, width) in enumerate(CELLS + [("random", 32768, 256)]):
            candidates = ["deque", "blocks"] if cell_index < len(CELLS) else ["aa"]
            for candidate in candidates:
                for pair in range(12):
                    arms = ["a", "b"] if pair % 2 == 0 else ["b", "a"]
                    for position, arm in enumerate(arms):
                        algorithm = ("scan" if arm == "a" else candidate) if candidate != "aa" else "deque"
                        label = f"c{cell_index}-{candidate}-p{pair}-{arm}"
                        command = ["taskset", "-c", str(cpu), binary, "--measure", algorithm, shape, str(n), str(width), "16"]
                        result = run(label, command)
                        observation = json.loads(result.stdout)
                        assert observation["elapsed_ns"] > 0
                        assert observation["reps"] == 16 and observation["n"] == n and observation["w"] == width
                        observation.update(cell=cell_index, shape=shape, contrast=candidate, pair=pair, arm=arm, position=position)
                        output.write(json.dumps(observation) + "\n")
                        output.flush()
    assert sum(1 for _ in samples.open()) == 408
    run("perf", ["perf", "stat", "-e", "cycles,instructions,branches,branch-misses,cache-misses", "taskset", "-c", str(cpu), binary, "--measure", "deque", "random", "32768", "256", "16"], required=False)
    (receipt / "binary.json").write_text(json.dumps({"sha256": digest(binary), "library_sha256": digest(library)}))
    manifest = {str(p.relative_to(receipt)): digest(p) for p in receipt.rglob("*") if p.is_file()}
    (receipt / "manifest.json").write_text(json.dumps(manifest, indent=2))
    with tarfile.open("receipt.tar.gz", "w:gz") as bundle:
        bundle.add(receipt, arcname="receipt")
    print(json.dumps({"complete": True, "samples": 408, "receipt_sha256": digest("receipt.tar.gz"), "source_commit": commit, "arch": arch}))

if __name__ == "__main__":
    main()
