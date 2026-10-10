# Correctness strategy and invariant audit

Scope:
`Table::new`, `insert`, `remove` and `get` under `Filter::{Unfiltered, Scalar, Word}`, the
`matching_mask` word filter, `hash` under `HashShape::{Mixed, Clustered, ConstantTag}`, and
narrow/wide `Key` equality. Single-threaded, fixed capacity, no resize.

Contract and failure classes:
`get` returns the exact stored value for a present key and `None` for an absent one under every
filter; `insert` replaces an existing key once and reports the old value, reuses a tombstone only
after the probe proves the key absent, and returns `Err(value)` when no slot is reusable; `remove`
leaves `DELETED`, never `EMPTY`; probing checks every candidate in a group before honoring `EMPTY`
and terminates on a table with no empty slot. Failure classes: borrow across byte lanes in the word
mask, early return at `EMPTY` before a later lane, duplicate key through a tombstone, unbounded
probe, erasure that hides a displaced survivor, tag match accepted without key comparison.

Primary technique:
Exact boundary examples on 16- and 32-slot tables plus a seeded insert/remove history on a 128-slot
table compared with `BTreeMap` after every transition over the whole 80-key domain.

Secondary techniques:
Exhaustive 32,768 byte/tag mask pairs; all 256 erasure subsets of eight colliding keys; constant-tag
and clustered hash controls; the checked example `examples/metadata.rs`; every benchmark process
verifies each query against `BTreeMap` and checks a batch checksum before timing.

Oracle:
`std::collections::BTreeMap<Key, u64>` or literal expected values. The oracle shares no probe, tag
or mask helper with `Table`.

Input/model/schedule domain and bounds:
Capacities 16, 32 and 128; key ids below 80 in the seeded history; 4,000 steps per configuration
across narrow/wide equality and three hash shapes (24,000 steps); removes on every third draw of a
fixed linear congruential sequence seeded with 1. Tombstones are never reclaimed, so the history
reaches the saturated regime where inserts reuse tombstones within the first few hundred steps.

Replay assets:
All fixtures are deterministic source-level values. Seeded history assertions print step, key,
equality width, hash shape and filter; no seed is read from the environment.

What a green run establishes:
Exact oracle agreement on the enumerated domains and after each seeded transition, the listed
rejections, and oracle agreement inside every measured benchmark process. Tests are bounded
evidence; they are not a proof for every key/prehash combination or a production table revision.
Uniform synthetic keys do not establish collision-attack resistance. No unsafe code is present.

Known gaps and delegated skills:
The `Table::new` capacity panic has no test. Narrow equality is exercised only through the seeded
history. No fuzzing (typed integer input), no Miri (no unsafe), no Loom (no shared state).

Concrete tests to add:
A `#[should_panic]` test for non-power-of-two or sub-16 capacity; a narrow-equality boundary
example with two keys that differ only outside the last word.

## Invariant-test audit

Shared facts: the comparator is exact equality on `Option<u64>`, `Result<Option<u64>, u64>` or
`u32` counts; every mutation backstop below was applied to the current source and observed to fail
the named test, then reverted.

### `every_byte_pair_mask_is_exact` (src/lib.rs)
Claim and scope: `matching_mask` sets exactly the lanes whose byte equals the tag, for every tag
0..=127 and byte 0..=255 in lane 0, plus an alternating 0/1 witness.
Plausible violation: a subtract-one trick borrows across lanes and reports an extra match.
Observation: bit `8i+7` of the mask for every lane `i`; `count_ones() == 7` on the witness.
Oracle and independence: `b == tag` per lane, computed without the mask.
Smallest discriminating case: lane 0 byte and tag pair with lane 15 fixed at 255.
Mutation backstop: `(x.wrapping_sub(ones) & !x) & high` fails with `left: 1 right: 0`.

### `candidate_after_empty_is_checked` (src/lib.rs)
Claim and scope: a group is scanned for every candidate lane before `EMPTY` ends the probe.
Plausible violation: returning absent when an empty lane precedes the matching lane.
Observation: `get(Key 9).value == Some(42)` for all three filters.
Oracle and independence: literal value placed directly in lane 15 of a fabricated group.
Smallest discriminating case: one tag in lane 15 after fifteen `EMPTY` lanes.
Mutation backstop: returning when the first `EMPTY` lane precedes the first tag lane fails.

### `wrap_tombstone_duplicate_and_full_cycle` (src/lib.rs)
Claim and scope: a 32-slot table with one shared hash fills across the group wrap, rejects a
33rd key, replaces key 31 in place after key 0 is erased, then reuses the tombstone for key 100.
Plausible violation: insert reuses the first tombstone before the probe proves the key absent, or
the probe stops after one group and loses wrapped keys.
Observation: `insert` results `Ok(None)`, `Err(100)`, `Ok(Some(31))`, `Ok(None)`; `get` values.
Oracle and independence: literal values.
Smallest discriminating case: 32 inserts with hash 1, erase 0, reinsert 31.
Mutation backstop: breaking the insert scan at the first reusable lane fails; bounding the `get`
probe to one group fails.

### `all_tags_collide_still_exact` (src/lib.rs)
Claim and scope: with every tag zero, present keys return their value and absent keys return
`None` under every filter.
Plausible violation: a tag match is accepted without key comparison.
Observation: `get(id).value == (id < 24).then_some(id)` for ids 0..48.
Oracle and independence: literal expectation from the insert set.
Smallest discriminating case: 24 constant-tag keys, query ids 0..48.
Mutation backstop: accepting a tag match without `same` fails for the first absent id.

### `seeded_operation_histories_against_ordered_map` (src/lib.rs)
Claim and scope: 4,000 seeded insert/remove steps per configuration agree with `BTreeMap` on every
return value and, after every transition, on `get` for all 80 modeled keys under all three filters.
Plausible violation: an insert or remove corrupts a slot other than the operated one.
Observation: per-step return value; per-step `get` over ids 0..80, present and absent.
Oracle and independence: `BTreeMap` driven by the same deterministic sequence.
Smallest discriminating case: the first step whose full-domain comparison diverges; messages name
step, key, width, shape and filter.
Mutation backstop: `remove` also tombstoning the sibling lane (`lane ^ 1`) fails at
`step 30 ... key 62`; the previous operated-key-only assertion caught it at step 81.

### `every_erasure_subset_preserves_survivors` (tests/contracts.rs)
Claim and scope: for every subset of eight erasures among 24 colliding keys, each survivor is
found and each erased or never-inserted key is absent, under every filter.
Plausible violation: an erasure writes `EMPTY` and hides keys displaced past it.
Observation: `get(id).value` for ids 0..32 against the subset mask.
Oracle and independence: literal expectation computed from the subset bits.
Smallest discriminating case: erasing key 0 alone hides the wrapped keys 16..24.
Mutation backstop: writing `EMPTY` instead of `DELETED` in `remove` fails.

Five library tests plus one external test pass locally. The initial harness invocation without
arguments failed during cargo --all-targets; the benchmark now returns normally when invoked without
explicit workload/candidate arguments. Literal/style and mathematically bounded conversion lint
findings were repaired before commit. Initial and exact committed campaigns remain separate and
their input hashes are retained. Both Linux exact-source contracts remain pending shared expired
Midway authentication.
