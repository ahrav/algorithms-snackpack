#!/usr/bin/env bash
set -euo pipefail

sha256sum --check SHA256SUMS
test "$(uname -s)" = Linux
test "$(uname -m)" = "$1"
mkdir -p result
{
    hostname
    uname -a
    lscpu
    nproc
    taskset -pc "$$"
    rustc -vV
    rustc --print cfg -C target-cpu=native
    cat SOURCE_COMMIT
} > result/host.txt

topic=topics/056-cycle-detection-and-deadlock-resolution
flags=(--edition=2024 -D warnings -C opt-level=2 -C overflow-checks=yes -C target-cpu=native)
printf '%s\n' "${flags[*]}" > result/flags.txt
crate=cycle_detection_and_deadlock_resolution
rustc "${flags[@]}" --crate-type lib --crate-name "$crate" \
    "$topic/src/lib.rs" -o "result/lib${crate}.rlib"
rustc "${flags[@]}" --test "$topic/tests/contracts.rs" \
    --extern "$crate=result/lib${crate}.rlib" -o result/contracts
rustc "${flags[@]}" "$topic/examples/deadlock.rs" \
    --extern "$crate=result/lib${crate}.rlib" -o result/deadlock

result/contracts --nocapture --test-threads=1 | tee result/contracts.txt
result/deadlock | tee result/deadlock.txt
sha256sum result/{host.txt,flags.txt,lib*.rlib,contracts,deadlock,contracts.txt,deadlock.txt} \
    > RESULT_SHA256SUMS
