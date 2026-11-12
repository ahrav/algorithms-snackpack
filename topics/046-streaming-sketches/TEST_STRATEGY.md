Scope: insertion-only u64 batches, <=1,000,000 observations; fixed-size sketches.
Contract and failure classes: exact baseline answers; Count-Min upper counts;
HLL register update/merge; rank tolerance; empty input; incompatible seed; batch limit.
Primary technique: independent Python full-workload reconstruction before accepting
each process, plus exhaustive Rust ternary streams of lengths0..8 (9,841 streams).
tests/contracts.rs rebuilds Count-Min counters, HLL registers and estimates from the
layout at the public boundary. test_run.py drives run.py with stub processes.
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
In cargo test, count_min_counters_and_row_minimum_match_independent_table compares
all1280 counters and4096 minima for [0,0,1] at seed3 and4096 uniform keys at seed7.
- Claim: HLL duplicate and compatible merge semantics. Violation: sum registers,
wrong bit split, rehash duplicates differently. Compare complete register vectors
and independently computed estimate. Case [4,4,7]; incompatible seed fails unchanged.
evaluate merges sketches of [4] and [4,7], so key4 has equal nonzero ranks on both
sides; hll_merge_keeps_register_maxima_and_estimates_from_them compares all1024
registers, the linear-counting estimate, and the raw estimate for16384 keys.
- Claim: rank tolerance and endpoints. Violation: omit gap transfer, use value
error, delete min, off-by-one rank. Sorted-data rank intervals are independent;
N<=8 requires exact output. Larger sorted/reverse/duplicate/random probes observe
all101 queries and Python checks cumulative stored intervals. For N4096,
compressed_rank_summary_keeps_101_observed_values_and_exact_endpoints requires101
observed values, exact minimum and maximum, and gaps summing to N. Python requires
every quantile and stored entry to occur in the input. Exact quantiles for
[4,7,4,9,4] are75 fours,25 sevens and one nine: index floor(4q/100) of [4,4,4,7,9].
- Claim: empty/limit/error behavior. Violation: synthesize quantile0 for empty,
accept unknown label, overflow state. Direct exact error checks and empty vectors;
1,000,001 observations rejects before mutation. Batch APIs own all mutable state.
empty_input_answers_are_exact_for_each_contract requires one zero per frequency
query, empty quantile and cardinality values, and zeroed fixed-size state.
- Claim: every attempt is retained and only checked output is marked PASS.
Violation: drop a timed-out, malformed or oracle-rejected attempt, accept output
with assertions disabled, mix two campaigns in raw/. test_run.py reads the raw
record after each failure shape, requires startup refusal under -O, -OO and
PYTHONOPTIMIZE and over earlier outputs, and compares all360 invocations of a
complete stub campaign with the declared schedule and the364-member manifest.
