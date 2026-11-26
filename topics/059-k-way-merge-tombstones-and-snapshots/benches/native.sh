#!/usr/bin/env bash
# Run from the extraction root of the path-limited topic archive.
set -euo pipefail
export LC_ALL=C
topic=topics/059-k-way-merge-tombstones-and-snapshots
mkdir evidence
{
  hostname -f
  uname -a
  lscpu
  taskset -pc "$$"
  rustc -Vv
  python3 --version
  rustc --print cfg -C target-cpu=native
  printf '%s\n' 'rustc --edition=2024 -C opt-level=3 -C target-cpu=native'
} > evidence/environment.txt
find "$topic" -type f -print0 | sort -z | xargs -0 sha256sum > evidence/members.sha256
flags=(--edition=2024 -C opt-level=3 -C target-cpu=native)
rustc "${flags[@]}" --crate-name k_way_merge_tombstones_and_snapshots --crate-type rlib "$topic/src/lib.rs" -o evidence/libsnapshot.rlib
external=(--extern k_way_merge_tombstones_and_snapshots=evidence/libsnapshot.rlib)
rustc "${flags[@]}" "${external[@]}" --test "$topic/tests/contract.rs" -o evidence/contract
rustc "${flags[@]}" "${external[@]}" "$topic/examples/snapshot.rs" -o evidence/snapshot
./evidence/contract > evidence/contract.txt
./evidence/snapshot demo > evidence/demo.txt
sha256sum evidence/snapshot evidence/contract evidence/libsnapshot.rlib > evidence/binaries.sha256
objdump -dC evidence/snapshot | gzip -c > evidence/disassembly.txt.gz
allowed=$(taskset -pc "$$" | awk '{print $NF}')
cpu=${allowed%%[-,]*}
printf '%s\n' "$cpu" > evidence/pinned-cpu.txt
taskset -c "$cpu" python3 "$topic/benches/compare.py" evidence/snapshot evidence/raw.jsonl > evidence/summary.json
