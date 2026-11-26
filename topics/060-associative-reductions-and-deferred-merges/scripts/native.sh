#!/usr/bin/env bash
set -euo pipefail
source_dir=$1
results=$2
mkdir -p "$results"
results=$(cd "$results" && pwd)
cd "$source_dir"
export CARGO_TARGET_DIR="$results/target"
export RUSTFLAGS="-C target-cpu=native"
cpu=$(awk '/^Cpus_allowed_list:/ { print $2 }' /proc/self/status | cut -d, -f1 | cut -d- -f1)
{
    hostname
    uname -a
    lscpu
    nproc
    taskset -pc "$$"
    printf 'selected CPU=%s\nRUSTFLAGS=%s\n' "$cpu" "$RUSTFLAGS"
    rustc -vV
    cargo -V
    python3 -V
    rustc -C target-cpu=native --print cfg
} > "$results/environment.txt"
cargo test --offline --test contracts > "$results/test.log" 2>&1
cargo run --offline --release --example contracts > "$results/example.txt" 2> "$results/example-build.log"
cargo bench --offline --bench reduction --no-run --message-format=json > "$results/build.jsonl" 2> "$results/build.log"
binary=$(python3 -c 'import json,sys; rows=map(json.loads,sys.stdin); print(next(r["executable"] for r in rows if r.get("executable") and r["target"]["name"]=="reduction"))' < "$results/build.jsonl")
sha256sum "$binary" > "$results/binary.sha256"
nm -C "$binary" > "$results/symbols.txt"
objdump -d -C "$binary" > "$results/disassembly.txt"
taskset -c "$cpu" python3 scripts/measure.py "$binary" "$results/processes.jsonl"
