# Evidence boundary

Raw evidence: /Users/ahrav/.codex/automations/algorithms-daily-curriculum/evidence/topic-024/20261020.
Frozen source manifests, raw process JSONL, schedules, host metadata, binary
hashes, launch/test outputs and transfer receipts remain external. Summaries
retain medians, all process ranges, paired ratio ranges and A/A controls.
Initial/preteach and original committed campaigns each ran 288 fresh
processes per host. A benchmark no-argument smoke default was repaired after
source a417c0a; timed explicit-argument behavior was unchanged. Final source
16d77dbde6d54ed62644395ddf6c6f226b7e1651 was independently
transferred/verified/recompiled. Use committed-final/x86 and
committed-isolated/arm, 288 processes each. The prior final Arm campaign
overlapped plugin tests and is excluded/preserved; the isolated replacement
ran after all own remote diagnostics/tests stopped. Final linked binary hashes
match. All responses, tests, source hashes and checksums passed. All campaigns
remain external as independent observations, not pooled replicates or tuned
repeats. The compact identity receipt is `EVIDENCE_RECEIPT.json`; its archive
hash, size and member count are unset because the raw root was not reachable
from the review host.

Arm: dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com, aarch64, 64 available
CPUs, Linux 6.12.111-136.204.amzn2023, Rust 1.98.1 LLVM 22.1.8. lscpu reports
ARM model 1; exact CPU fields retained, no marketing-model inference. xxl
resolved at runtime to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com, x86_64,
192 available CPUs, Xeon Platinum 8488C, Rust 1.99.0 LLVM 23.1.1, same kernel
release family. Native opt3. No core/frequency pinning, cold-cache, tail, PMU,
RSS or ISA-only conclusion.

Final fastest correct selections: radix tiny/shared/wide/prefix256 on both;
trie early miss on both; flat broad output and wide build on both. Shared build
resolves flat on both.
All six paired ratios must exceed 1.03 against every rival. Descriptive, not
statistical significance. Early x86 broad output and original-commit x86 wide
build were unresolved; do not conceal this campaign sensitivity. No extra runs
were made to seek resolution. Final selectors are committed-*.json.

Custom shared capacity: flat 557056, trie 789120, radix 1245986 bytes. BTree
private capacity is unreported. Separate 24-process final allocation
diagnostics and actual linked Radix::locate/Index::get inspection are retained
externally. Exact lookup 64 queries allocated 0 in all candidates. Shared build
allocation calls: flat 8195, tree 8940, trie 68, radix 8263; realloc calls 11,
0, 208, 218 respectively. Cumulative requested bytes differ from live capacity
(radix 4029250 bytes). Shared prefix 64 results allocate 64 vectors (tree also
64 lower-bound keys) and reallocate; wide prefix queries match one value each
and allocate 64 vectors without reallocation. Instrumented timings are
excluded. See allocation.json for complete fields.
Regenerate the 12 rows for one host from the topic directory:

```bash
OUT=../../target/alloc-probe; mkdir -p "$OUT"
rustc --edition=2024 --crate-type=rlib --crate-name topic024_tries_radix_trees_and_art \
  -C opt-level=3 -C target-cpu=native src/lib.rs -o "$OUT/libtopic024.rlib"
rustc --edition=2024 -C opt-level=3 -C target-cpu=native scripts/alloc_probe.rs \
  --extern topic024_tries_radix_trees_and_art="$OUT/libtopic024.rlib" -o "$OUT/alloc_probe"
for shape in tiny shared wide; do for c in flat tree trie radix; do "$OUT/alloc_probe" $c $shape; done; done
```

Each line of output is one allocation.json row. The committed rows used the
host default toolchains named above (Arm rustc 1.98.1, x86 rustc 1.99.0).
The tiny rows (32 shared-prefix keys) were regenerated on both hosts with the
recorded toolchains after the probe mapped `tiny` to the shared workload; the
earlier tiny rows had measured 32 bare eight-byte keys. Tiny build allocation
calls: flat 34, tree 37, trie 37, radix 38, identical on both hosts. Shared and
wide rows reproduced the committed values exactly.
Linked code has scalar byte/index access paths and comparison calls; no PMU,
cache-miss, branch-miss or instruction-count mechanism is inferred.
