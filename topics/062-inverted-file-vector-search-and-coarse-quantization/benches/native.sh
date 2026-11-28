#!/usr/bin/env bash
set -euo pipefail
export LC_ALL=C
mkdir -p evidence bin
{
    date -u +%FT%TZ
    hostname
    uname -a
    lscpu
    nproc
    rustc +1.93.0 -vV
    rustc +1.93.0 --print cfg -C target-cpu=native
    python3 --version
    printf '%s\n' 'edition=2024 opt-level=3 target-cpu=native'
} > evidence/host.txt
find src examples benches tests -type f | sort | xargs sha256sum > evidence/source.sha256
flags=(--edition=2024 -C opt-level=3 -C target-cpu=native)
crate=inverted_file_vector_search_and_coarse_quantization
rustc +1.93.0 "${flags[@]}" --crate-name "$crate" --crate-type rlib src/lib.rs -o "bin/lib$crate.rlib"
rustc +1.93.0 "${flags[@]}" --crate-name "$crate" --crate-type rlib --emit asm src/lib.rs -o evidence/library.s
linked=(--extern "$crate=bin/lib$crate.rlib")
rustc +1.93.0 "${flags[@]}" "${linked[@]}" --test tests/contracts.rs -o bin/contracts
rustc +1.93.0 "${flags[@]}" "${linked[@]}" examples/demo.rs -o bin/demo
rustc +1.93.0 "${flags[@]}" "${linked[@]}" benches/search.rs -o bin/search
sha256sum bin/* > evidence/image.sha256
./bin/contracts | tee evidence/contracts.log
./bin/demo | tee evidence/demo.log
python3 benches/run.py "$PWD/bin/search" evidence | tee evidence/run.log
