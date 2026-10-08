# Contract and invariant checks

Independent model: BTreeMap buckets keyed by unsigned order key, retaining full records in arrival
order. Stable candidates must match it exactly. Unstable candidates must be ordered and preserve
the exact multiset of (key,id), including repeated payload IDs. Candidate critical helpers are not
shared with the oracle.

Seven checks cover 9,841 exhaustive vectors over three keys through length eight; seeded workload
shapes at empty/singleton, insertion/quick cutoffs, run lengths and 4,096; strict descending reversal
with adjacent equals; zero/u64::MAX/prefix extrema; atomic invalid-prefix rejection; shuffled IDs;
and the invalid arbitrary-segment concatenation counterexample.

Plausible violations rejected: lost or duplicated scatter records, reversed equal-key order,
using payload ID as tie breaker, off-by-one heap/run boundaries, strict versus inclusive comparison,
partial mutation before rejecting groups and concatenating sorted chunks without a boundary proof.

These bounded tests do not prove all lengths, allocation failure behavior, generic comparator panic
safety or global-order encodings for signed/float/string keys. Those contracts are outside this API.

Run `cargo test -p sorting_lab --all-targets`; workspace --lib/--examples alone does not run
integration contracts, so all-targets is an additional gate. No unsafe code is used.
