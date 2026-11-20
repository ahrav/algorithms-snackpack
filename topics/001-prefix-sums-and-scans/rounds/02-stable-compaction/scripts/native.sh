#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
rustc_bin=${RUSTC:-$HOME/.cargo/bin/rustc}
result=${1:?result path required}
mkdir "$result"
result=$(cd "$result" && pwd)
args=(--edition=2021 -C opt-level=3 -C codegen-units=1 -C target-cpu=native)
cpu=$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')
{
    hostname
    uname -a
    lscpu
    python3 -c 'import os; print("Available CPUs:", sorted(os.sched_getaffinity(0)))'
    printf 'Pinned CPU: %s\n' "$cpu"
    printf 'Rust flags: %s\n' "${args[*]}"
    "$rustc_bin" -vV
    "$rustc_bin" -C target-cpu=native --print cfg | sort
    python3 --version
} > "$result/environment.txt"
"$rustc_bin" "${args[@]}" --crate-type=lib --crate-name=stable_flag_compaction src/lib.rs -o "$result/libstable_flag_compaction.rlib"
for unit in examples/compact.rs benches/compact.rs tests/contract.rs; do
    library="stable_flag_compaction=$result/libstable_flag_compaction.rlib"
    case "$unit" in
        tests/*) "$rustc_bin" "${args[@]}" --test "$unit" --extern "$library" -o "$result/contract";;
        examples/*) "$rustc_bin" "${args[@]}" "$unit" --extern "$library" -o "$result/example";;
        benches/*) "$rustc_bin" "${args[@]}" "$unit" --extern "$library" -o "$result/bench";;
    esac
done
"$result/contract" > "$result/contract.txt"
"$result/example" > "$result/example.txt"
if command -v objdump >/dev/null; then
    objdump -Cd "$result/bench" | awk '/^[[:xdigit:]]+ <stable_flag_compaction::(cursor|ranks|tiles|runs)>:$/ {emit=1} emit {print} /^$/ {emit=0}' > "$result/kernels.asm"
fi
sha256sum "$result/bench" "$result/example" "$result/contract" > "$result/binaries.sha256"
taskset -c "$cpu" python3 scripts/measure.py "$result/bench" "$result"
