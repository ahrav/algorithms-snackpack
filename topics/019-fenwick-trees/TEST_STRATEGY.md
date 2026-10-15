# Correctness strategy and invariant audit

Scope: `Fenwick`, `Counts`, `RangeAdd` and the benchmark comparators `Scan`,
`Prefix`, `Segment` behind the `Index` trait, over u64 values on a fixed length.
Single-threaded; no allocation or timing claims are tested here.

Contract and failure classes: half-open prefix and range sums modulo 2^64 after
interleaved point additions; panic before mutation on reversed or out-of-range
endpoints for `Fenwick` and `RangeAdd`; `Counts` admits only nonnegative counts
with a checked u64 total, rejects overflowing construction or replacement without
state change, and returns the index holding a zero-based rank or `None` at or
above the total; `Index` comparators honor the sum contract for caller-validated
endpoints.

Primary technique: exhaustive small-domain model agreement. Every ternary array
through length seven checks every range before and after each modular negative
point update, for `Fenwick` and for each comparator.

Secondary techniques: exhaustive rank selection on nonnegative arrays through
length six; differential range-add over every boundary pair through length
seventeen; literal wrap, maximum-total and rejection witnesses; the rendered
README example as an integration test; per-query comparison against `Scan`
inside every benchmark process before its timing is admitted.

Oracle: a fold over the logical leaf array for sums; a linear cumulative scan
for rank; a mutated plain array for range additions; `Scan` for benchmark
streams. The oracles share no bucket, lowbit, prefix-repair or segment-walk code
with the candidates. `Scan` itself is checked against the leaf fold.

Input/model/schedule domain and bounds: values in {0,1,2} with updates of
modular -3, lengths 0..=7 (sums) and 0..=6 (ranks); deltas +7 and modular -3
over lengths 0..=17 (range add); literals `u64::MAX`, 0 and 1 for wrap and
overflow; eight declared benchmark streams with a fixed xorshift seed.

Replay assets: deterministic enumerations and literals in `src/lib.rs` tests,
`tests/contracts.rs`, and the frozen seed in `benches/portfolio.rs`; no random
seeds are drawn at run time.

What a green run establishes: agreement with the independent models on the
enumerated domains and literals, exact panics on the listed invalid boundaries,
unchanged `Counts` state after rejected replacement, and query-level agreement
of each candidate with `Scan` on the declared streams. These are finite,
bounded witnesses.

Known gaps and delegated skills: no generated inputs beyond the enumerations;
no tests for lengths above seventeen; comparators accept invalid endpoints by
contract and are not tested for rejection; selection and range-add timing are
unmeasured; allocation counts are untested.

Concrete tests to add: generated long arrays with random update/query schedules
against the fold oracle; a `Counts::select` test at ranks near `u64::MAX` with
many leaves; metamorphic checks that `range(l, r) == prefix(r) - prefix(l)`
across random endpoints.

## Invariant audit

### exhaustive_prefix_ranges_and_updates

Claim and scope: `Fenwick::range` equals the leaf sum for every range of every
ternary array through length seven, before and after each modular update.
Plausible violation: wrong endpoint convention, wrong lowbit walk direction,
incorrect linear parent forwarding, broken range cancellation.
Observation: every `(l, r)` after every update pass.
Oracle and independence: fold over the mutated leaf array; no shared code.
Comparator: exact u64 equality.
Smallest discriminating case: length one, value 1, `range(0, 1)` after one update.
Mutation backstop: flipping the parent-forward bound or the walk direction fails.

### comparators_agree_with_leaf_sums

Claim and scope: `Scan`, `Prefix` and `Segment` satisfy the same sum contract on
the same domain.
Plausible violation: off-by-one suffix repair, wrong segment edge handling,
padding leak into sums.
Observation: every `(l, r)` after every update pass, per comparator.
Oracle and independence: the leaf fold; comparators share no code with it.
Comparator: exact u64 equality.
Smallest discriminating case: length two, `range(1, 2)` after updating index 0.
Mutation backstop: inverting the segment right-edge parity test or shifting the
prefix repair start by one fails.

### selection_zeros_and_atomic_rejections

Claim and scope: `Counts::select` returns the index containing each zero-based
rank below the total and `None` at the total; rejected replacements leave state
unchanged.
Plausible violation: `<` instead of `<=` skipping zero leaves, wrong step
initialization, rank not decremented, mutation before overflow check.
Observation: every rank from 0 through the total for every array; selections
after rejected and accepted replacements.
Oracle and independence: linear cumulative scan with `total > rank`.
Comparator: `Option<usize>` equality and `bool` return values.
Smallest discriminating case: array `[0, 1]`, rank 0 must select index 1.
Mutation backstop: changing `<=` to `<` in `select` fails on zero leaves.

### range_add_differential

Claim and scope: `RangeAdd::range` equals the sum of a plain array receiving the
same range additions, for every boundary pair through length seventeen.
Plausible violation: wrong boundary event sign, missing suffix event at `n`,
wrong index weight in the second tree.
Observation: every `(s, e)` after every `(l, r)` addition.
Oracle and independence: a mutated `Vec<u64>` with per-element adds.
Comparator: exact u64 equality.
Smallest discriminating case: length one, `add(0, 1, 7)`, `range(0, 1)`.
Mutation backstop: dropping the `wrapping_neg` on the end event fails.

### invalid_bounds_and_wrap

Claim and scope: `Fenwick` wraps modulo 2^64 and panics before mutating on an
out-of-range index, reversed range, or oversized prefix endpoint.
Plausible violation: assertion placed after the first write; missing assertion.
Observation: `catch_unwind` outcome, then a sum read after the failed call.
Oracle and independence: literal expected sums.
Comparator: exact u64 equality and `is_err()`.
Smallest discriminating case: `add(3, 7)` on length three, then `range(0, 3)`.
Mutation backstop: moving `assert!` below the write changes `range(0, 3)`.

### running_example

Claim and scope: the README walkthrough values hold through the public API.
Plausible violation: documentation drift from the implementation.
Observation: `prefix(7)`, `range(2, 7)`, `select(8)` and `RangeAdd::prefix(7)`.
Oracle and independence: hand-derived literals in the README.
Comparator: exact equality.
Smallest discriminating case: the example itself.
Mutation backstop: any change to the walkthrough values fails the test.
