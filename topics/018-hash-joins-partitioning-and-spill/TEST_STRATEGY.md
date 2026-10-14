# Correctness strategy and invariant audit

Scope: `nested`, `hash`, `merge`, `partition`, `chunk`, `write_keys`, `spill` and
`SpillStats` over non-null u64 keys. Single-threaded; `spill` owns its scratch directory.

Contract and failure classes: exact inner-join pair counts with duplicate
multiplicity; bounded build rows per table under `chunk`/`spill`; progress through
depth and no-progress guards; exact `InvalidData`/`InvalidInput` rejection with
no spill files created; logical read/write accounting that excludes source creation.

Primary technique: exhaustive small-domain model agreement, 14,641 ordered pairs of
all 121 vectors of length 0..4 over 3 keys, every resident candidate and budgets
1/2/5 against `nested`.

Secondary techniques: literal extreme-key and million-pair cases; file-backed
examples with hand-derived counters; exact rejection kinds plus scratch inventory.
Each timed benchmark process checks its dataset against an independent BTreeMap
frequency oracle before timing.

Oracle: `nested` double loop for resident candidates and `spill`; hand-derived
literals for counters; BTreeMap frequency map in benchmarks. The oracles share no
hashing, sorting, bucketing or file code with the candidates.

Input/model/schedule domain and bounds: keys 0..2 exhaustively, 0 and `u64::MAX`,
contiguous and all-equal file inputs of 3 to 257 rows, budgets 1 to 7, depth 0 to 3.

Replay assets: deterministic literals in `src/lib.rs` tests, `tests/contracts.rs`
and `examples/demo.rs`; no seeds.

What a green run establishes: agreement on the enumerated domain and literals, exact
counter values for three file layouts, exact rejection kinds and an unchanged scratch
inventory. It is bounded evidence, not a proof, whole-memory bound, durability check,
cancellation/fault-injection test or production SQL conformance suite.

Known gaps and delegated skills: no generated large-key properties, no I/O fault
injection, no concurrency, no byte-budget enforcement, no cold-cache or storage timing.

Concrete tests to add: generated multisets over the full u64 domain against
`nested`; fault injection on partition file creation; multi-level spill trees whose
counters are derived from an independent file model.

## Invariant-test audit

Shared facts: comparators are exact equality on `u128`, `usize`, `u64` tuples,
`io::ErrorKind` and directory entry counts. Every mutation backstop was applied to a
scratch copy of the committed `src/lib.rs`, observed to fail the named test with the
quoted output, and discarded; repository source was restored byte for byte.

### `exhaustive_multisets` (src/lib.rs)
Claim and scope: for all 14,641 ordered pairs of vectors of length 0..4 over keys
0..2, `hash`, `merge`, `partition` and `chunk` with budgets 1/2/5 equal `nested`.
Plausible violation: lost duplicate multiplicity, mismatched build/probe routing,
run-cursor errors in `merge`, double counting across chunks.
Observation: four equalities per pair and budget.
Oracle and independence: `nested` double loop; shares no table, sort or bucket code.
Comparator: exact `u128` equality.
Smallest discriminating case: one-key vectors with repeated keys.
Mutation backstop: probe routing `bucket(key, 1, 15)` in `partition` fails
`left: 0 right: 1`.

### `extreme_keys_and_output_growth` (src/lib.rs)
Claim and scope: keys 0 and `u64::MAX` join to 5 pairs under `nested` and
`partition`; 1000 equal keys on both sides give 1,000,000 under `merge`.
Plausible violation: bucket mixing or masking that mishandles extreme keys; run
lengths added instead of multiplied.
Observation: three literal counts.
Oracle and independence: hand-derived literals.
Comparator: exact equality.
Smallest discriminating case: two equal runs of length 1000.
Mutation backstop: `+` for `*` on run lengths in `merge` fails
`left: 2000 right: 1000000`.

### `spill_progress_and_duplicates` (tests/contracts.rs)
Claim and scope: 257 rows over 23 keys against 131 rows over 19 keys with budget 7
and depth 3 equal `nested`, partition more than once and load at most 7 build rows.
Plausible violation: lost multiplicity at a file leaf, dropped child pairs, a leaf
that exceeds the row cap.
Observation: count equality, `partitions > 1`, `max_build_rows <= 7`.
Oracle and independence: `nested` over the resident copies of the written files.
Comparator: exact equality and bounds.
Smallest discriminating case: a two-level tree where several keys share a leaf.
Mutation backstop: `table.contains_key` for the frequency lookup in `leaf` fails
`left: 311 right: 1469`.

### `heavy_key_terminates_without_losing_pairs` (tests/contracts.rs)
Claim and scope: 31 and 29 copies of one key with budget 4 and depth 3 give 899
pairs after one partition, one fallback, 4 build rows, 60 writes and 323 reads.
Plausible violation: recursion past a child with no progress, duplicated or lost
pairs across the 8 build chunks, probe rescans uncounted.
Observation: count and all five counters.
Oracle and independence: hand-derived literals (31 × 29; 31 + 29 split reads and
writes; 31 build reads; 8 × 29 probe reads).
Comparator: exact equality.
Smallest discriminating case: one child holding every build row.
Mutation backstop: disabling the no-progress guard fails `left: 3 right: 1` on
`partitions`.

### `leaf_counts_reads_without_writes` (tests/contracts.rs)
Claim and scope: 3 build rows against 3 probe rows under budget 4 give 2 pairs, 6
reads, 0 writes, no partitions or fallbacks, and leave only the two inputs in scratch.
Plausible violation: source creation counted as writes; probe reads uncounted; files
created for an under-budget build.
Observation: count, `(reads, writes)`, `(partitions, fallbacks)`, directory entries.
Oracle and independence: hand-derived literals.
Comparator: exact equality.
Smallest discriminating case: a single leaf.
Mutation backstop: removing the probe read increment in `leaf` fails
`left: (3, 0) right: (6, 0)`.

### `heavy_key_with_empty_probe_bucket_skips_fallback` (tests/contracts.rs)
Claim and scope: 31 copies of key 0 against 29 copies of key 1 with budget 4 and
depth 3 give 0 pairs, one partition, zero fallbacks, 60 reads, 60 writes and
`max_build_rows == 0`.
Plausible violation: the stalled build child is sent to a chunked leaf against an
empty probe child, rereading the build and recording a fallback.
Observation: count and all five counters.
Oracle and independence: hand-derived literals; the two keys route to different
first-level children.
Comparator: exact equality.
Smallest discriminating case: one populated build child paired with an empty probe child.
Mutation backstop: dropping the `file_rows(y)? > 0` guard (the pre-fix source at
`e6dfe9c`) fails `left: 1 right: 0` on `fallbacks`.

### `malformed_and_empty_inputs` (tests/contracts.rs)
Claim and scope: a 7-byte file is rejected with `InvalidData`; budget 0 and depth 32
are rejected with `InvalidInput`; two empty inputs under depth 0 return 0; after all
calls scratch holds exactly the two input files.
Plausible violation: a truncated record decoded as a key or surfacing as a different
error kind; validation after partition files are created.
Observation: `ErrorKind` of each rejection, the empty result, directory entry count.
Oracle and independence: documented error kinds and the input inventory.
Comparator: exact equality.
Smallest discriminating case: a single partial key.
Mutation backstop: disabling the `bytes % 8 != 0` check fails with
`called Result::unwrap_err() on an Ok value: (0, SpillStats { reads: 0, ... })`,
since the 7-byte file then rounds down to zero rows and joins as empty.
