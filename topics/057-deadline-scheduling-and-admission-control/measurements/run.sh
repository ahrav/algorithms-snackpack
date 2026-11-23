#!/usr/bin/env bash
# Execute the frozen example and independent contract tests on a Linux host.
set -euo pipefail
cd "$(dirname "$0")/.."
export LC_ALL=C
printf 'hostname: '
hostname
uname -smr
lscpu
printf 'available CPUs: '
nproc
rustc -Vv
printf 'build flags: %s\n' '--edition=2024 -C opt-level=2 -C target-cpu=native'
rustc --print cfg -C target-cpu=native
sha256sum src/lib.rs examples/deadline.rs tests/contracts.rs measurements/run.sh
mkdir -p replay
crate=deadline_scheduling_and_admission_control
flags=(--edition=2024 -C opt-level=2 -C target-cpu=native)
rustc "${flags[@]}" --crate-name "$crate" --crate-type rlib src/lib.rs -o "replay/lib$crate.rlib"
external=(--extern "$crate=replay/lib$crate.rlib")
rustc "${flags[@]}" "${external[@]}" examples/deadline.rs -o replay/deadline
rustc "${flags[@]}" "${external[@]}" --test tests/contracts.rs -o replay/contracts
sha256sum replay/deadline replay/contracts
replay/deadline
replay/contracts --test-threads=1
