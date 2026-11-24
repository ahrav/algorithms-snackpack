#!/usr/bin/env bash
# Replay the source-matched contracts and timing campaign on Linux.
set -euo pipefail
cd "$(dirname "$0")/.."
export LC_ALL=C
hostname
uname -smr
lscpu
nproc
rustc -Vv
python3 --version
flags=(--edition=2024 -C opt-level=3 -C target-cpu=native)
printf 'build flags: %s\n' "${flags[*]}"
rustc --print cfg -C target-cpu=native
sha256sum src/lib.rs examples/{queues,bench_queues}.rs tests/contracts.rs measurements/{run.sh,campaign.py}
mkdir -p replay
crate=deficit_round_robin_and_fair_queueing
rustc "${flags[@]}" --crate-name "$crate" --crate-type rlib src/lib.rs -o "replay/lib$crate.rlib"
external=(--extern "$crate=replay/lib$crate.rlib")
rustc "${flags[@]}" "${external[@]}" examples/queues.rs -o replay/queues
rustc "${flags[@]}" "${external[@]}" --test tests/contracts.rs -o replay/contracts
rustc "${flags[@]}" "${external[@]}" examples/bench_queues.rs -o replay/bench_queues
rustc "${flags[@]}" "${external[@]}" --emit=asm examples/bench_queues.rs -o replay/bench_queues.s
sha256sum replay/{queues,contracts,bench_queues,bench_queues.s}
replay/queues
replay/contracts --test-threads=1
python3 measurements/campaign.py > replay/processes.jsonl
sha256sum replay/processes.jsonl
