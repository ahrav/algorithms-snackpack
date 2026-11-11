Scope: insertion-only u64 batches, <=1,000,000 observations; fixed-size sketches.
Contract and failure classes: exact baseline answers; Count-Min upper counts;
HLL register update/merge; rank tolerance; empty input; incompatible seed; batch limit.
Primary technique: independent Python full-workload reconstruction before accepting
each process, plus exhaustive Rust ternary streams of lengths0..8 (9,841 streams).
Secondary techniques: ordering/duplicate/extreme-value probes and partition merge.
Oracle: Python Counter/set/sorted, bit-length HLL model and formula; Rust filter
counts and sorted rank intervals. Oracles share neither candidate update helpers
nor quantile interval maintenance. Input generation shares a specified mixer only.
Input/model/schedule domain and bounds: six fixed cells,6 timing pairs/cell,16 HLL
seeds/cell,4 contract groups; deterministic integer hash model and <=1m items.
Replay assets: src/lib.rs, main.rs, run.py; frozen hashes in external receipt.
What a green run establishes: bounded mechanical/model agreement and observed error.
Known gaps and delegated skills: probability coverage for mixer, production merges,
windows, serialized compatibility, concurrency, original GK space theorem, peak RSS.
Concrete tests to add: future weighted/window/serialization APIs require new contracts.

Invariant-test audit:
- Claim: Count-Min upper estimates. Violation: increment one row only/use max or
wrong seed. Observe all counters and all4096 queries against independent Python
counts; ternary minimum case [0,0,1]. Backstop: mismatched counter/min is rejected.
- Claim: HLL duplicate and compatible merge semantics. Violation: sum registers,
wrong bit split, rehash duplicates differently. Compare complete register vectors
and independently computed estimate. Case [4,4,7]; incompatible seed fails unchanged.
- Claim: rank tolerance and endpoints. Violation: omit gap transfer, use value
error, delete min, off-by-one rank. Sorted-data rank intervals are independent;
N<=8 requires exact output. Larger sorted/reverse/duplicate/random probes observe
all101 queries and Python checks cumulative stored intervals.
- Claim: empty/limit/error behavior. Violation: synthesize quantile0 for empty,
accept unknown label, overflow state. Direct exact error checks and empty vectors;
1,000,001 observations rejects before mutation. Batch APIs own all mutable state.
