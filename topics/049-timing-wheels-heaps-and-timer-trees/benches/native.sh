#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir native
{ hostname; uname -a; lscpu; nproc; rustc -Vv; rustc --print cfg -C target-cpu=native; taskset -pc $$; } > native/environment.txt
printf '%s\n' 'rustc --edition=2024 -C opt-level=3 -C target-cpu=native; taskset -c 0; 16 replays/process; 6 blocks' > native/build.txt
sha256sum src/lib.rs examples/replay.rs tests/contracts.rs benches/run.py benches/native.sh > native/inputs.sha256
rustc --edition=2024 --crate-type rlib --crate-name topic_049_timers -C opt-level=3 -C target-cpu=native src/lib.rs -o native/libtopic_049_timers.rlib
rustc --edition=2024 -C opt-level=3 -C target-cpu=native examples/replay.rs --extern topic_049_timers=native/libtopic_049_timers.rlib -o native/replay
rustc --edition=2024 --test tests/contracts.rs --extern topic_049_timers=native/libtopic_049_timers.rlib -o native/contracts
native/contracts > native/contracts.txt
rustc --edition=2024 --test src/lib.rs -o native/unit-tests
native/unit-tests > native/unit-tests.txt
rustc --edition=2024 --crate-type rlib --crate-name topic_049_timers -C opt-level=3 -C target-cpu=native --emit=asm src/lib.rs -o native/library.s
python3 benches/run.py --binary native/replay --out native/results > native/summary.json
find native -type f ! -name members.sha256 -print0 | sort -z | xargs -0 sha256sum > native/members.sha256
