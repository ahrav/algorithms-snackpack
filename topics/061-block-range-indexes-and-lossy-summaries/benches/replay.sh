#!/usr/bin/env bash
set -euo pipefail
# Run on an extracted, hash-verified topic archive with the root manifest.
topic=topics/061-block-range-indexes-and-lossy-summaries
export PATH="$HOME/.cargo/bin:$PATH"
export RUSTFLAGS="-C target-cpu=native"
export CARGO_TARGET_DIR="$PWD/target"
mkdir evidence
{
    hostname -f
    uname -a
    lscpu
    nproc
    rustc -Vv
    cargo -V
    rustc $RUSTFLAGS --print cfg
    printf 'RUSTFLAGS=%s\n' "$RUSTFLAGS"
    grep Cpus_allowed_list /proc/self/status
} > evidence/environment.txt
# Retain the root toolchain and shared lint configuration. Restrict membership
# to the single topic within this replay archive.
python3 - "$topic" <<'PY'
from pathlib import Path
import re
import sys
p = Path("Cargo.toml")
s = p.read_text().split("[workspace]", 1)[1]
s = re.sub(r'members = \[.*?\]', 'members = ["' + sys.argv[1] + '"]', s, count=1, flags=re.S)
p.write_text("[workspace]" + s)
PY
cargo test --release --manifest-path "$topic/Cargo.toml" > evidence/test.log 2>&1
cargo run --release --manifest-path "$topic/Cargo.toml" --example demo > evidence/demo.log 2>&1
python3 "$topic/benches/run.py" evidence/process
binary="$(cat evidence/process/binary-path)"
sha256sum "$binary" > evidence/binary.sha256
cp "$binary" evidence/scan.bin
objdump -Cd "$binary" > evidence/scan.asm
