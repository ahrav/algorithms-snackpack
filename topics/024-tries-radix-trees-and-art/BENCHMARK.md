# Frozen dictionary experiment

benchmark_required: true

Four candidates: sorted vector (`flat`), standard `BTreeMap` (`tree`), byte trie
(`trie`), and immutable scalar adaptive radix model (`radix`). No online-update,
concurrent ART, kernel XArray or production-database timing claim.

| Cell | Input and operation | Repeats per subsample |
|---|---|---:|
| tiny | 32 shared-prefix keys, exact lookup | 400 |
| shared | 8,192 shared-prefix keys, exact lookup | 40 |
| wide | 8,192 keys, 256-way first-byte fanout | 40 |
| miss | 8,192 shared keys, first-byte misses | 40 |
| prefix256 | 8,192 shared keys, 256 output values per query | 40 |
| prefix_all | 8,192 shared keys, all values per query | 1 |
| build_shared | shuffled shared keys, construction/query/drop | 1 |
| build_wide | shuffled wide keys, construction/query/drop | 1 |

Shared keys contain `tenant/region/catalog/items/` plus an eight-byte big-endian
ordinal. Wide keys prepend ordinal modulo 256 to the eight-byte ordinal.
Deterministic Fisher-Yates input permutation uses seed 24020. Each batch has
64 queries selected by `(i * 127) % n`. Exact batches alternate hits and
append-0xff misses, except `miss` which changes the first byte to 0xff.
Prefix256 removes the final ordinal byte. Prefix_all uses a seven-byte prefix.
Outputs contain values in key order, not cloned full keys.

## Units, assignment, controls and stopping

Six fresh processes per candidate/cell, arranged in three mirrored order pairs.
Rotate the six labels (four candidates plus identical treeA/treeB controls) by
pair. Each pair reverses the same order. Rotate cells by pair too. Total:
8 cells * 6 labels * 6 blocks = 288 fresh processes per host per campaign.
One complete warmup batch; three timed subsamples reduced to one process median.
Subsamples and inner repeats are not independent observations.

Steady timing excludes construction/final index destruction. Prefix allocation,
complete output materialization, checksum folding and output destruction are
included. Build timing includes construction, 64 queries, folding and index
destruction. Input/query generation, oracle and process startup are excluded.
The correctness index is released before build timing. Assertions/checksums are
included consistently in each candidate's timed operation.

Select the lowest process median only if every rival/best ratio exceeds 1.03
in all six paired blocks. Otherwise report unresolved. Retain all process ranges
and A/A ratio ranges. This is descriptive evidence, not a confidence interval.
No pinning, frequency controls, explicit cache flush, PMU or tail inference.
Rust/toolchain versions differ between hosts. Stop after the fixed schedule;
source replays do not tune workloads to obtain a winner.

## Correctness and source identity

Check each prefix response against an independent latest-write HashMap plus
filter/sort oracle before warmup. Exact queries use a reverse input scan.
Reject a mismatch before selecting timing. Verify hostname, architecture, source
hashes, runner, binary hash, kernel, CPU, toolchain, native features and available
CPUs. Raw observations, schedules and attempts remain outside Git. Keep only
final summaries, allocation aggregates and compact identity receipts here.

Initial/preteach campaigns precede integration. An original committed campaign
is retained separately. A later no-argument startup repair leaves explicit-arg
timed behavior unchanged, but requires an exact final-source replay. Exclude
any host campaign overlapping owned diagnostic/test work; preserve it and use
one isolated exact-source replacement. Do not pool these as extra replicates.

## Separate storage and allocation diagnostics

`scripts/alloc_probe.rs` forwards System allocator operations through counters.
Its 12 fresh processes per host report alloc, realloc, dealloc and cumulative
requested bytes for construction, 64 exact queries, 64 prefix queries and drop.
No instrumented timing enters selection. Unsafe forwarding exists only in this
standalone diagnostic, with safety comments; the candidate library forbids unsafe.
Counts include requests, not live memory or RSS. Allocation failure is not
injected. The library's custom capacity calculations exclude allocator headers
and private BTreeMap internals. Tree capacity zero means unreported.

Inspect Radix::locate and Index::get in the actual final linked binary. Scalar
byte/index loads and comparison calls describe compiled code shape, not measured
cache, branch or instruction-count causality. See measurements/README.md.
