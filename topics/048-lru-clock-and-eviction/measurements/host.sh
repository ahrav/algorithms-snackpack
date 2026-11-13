#!/bin/sh
set -eu
# Run from the topic root after verifying the frozen-source manifest.
rustc --edition=2024 --crate-name topic048_eviction --crate-type rlib -C opt-level=3 -C target-cpu=native src/lib.rs -o libtopic048_eviction.rlib
rustc --edition=2024 -C opt-level=3 -C target-cpu=native examples/measure.rs --extern topic048_eviction=libtopic048_eviction.rlib -o measure
rustc --edition=2024 --test tests/contracts.rs --extern topic048_eviction=libtopic048_eviction.rlib -o contracts
./contracts
rustdoc --edition=2024 --test src/lib.rs --extern topic048_eviction=libtopic048_eviction.rlib
rustc --edition=2024 --crate-name topic048_eviction --crate-type rlib -C opt-level=3 -C target-cpu=native --emit=asm src/lib.rs -o implementation.s
sha256sum measure libtopic048_eviction.rlib contracts implementation.s > images.sha256
cpu=$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')
taskset -c "$cpu" python3 measurements/run.py ./measure raw
