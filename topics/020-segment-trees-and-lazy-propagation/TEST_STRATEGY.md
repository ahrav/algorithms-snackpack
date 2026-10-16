# Correctness strategy and invariant audit

Scope: `Tag`, `RangeSum` behind `Kind::{Scan, Prefix, Eager, Lazy}`, and
`OrderedTree<T>`, over u64 values on a fixed length. Single-threaded; no
allocation or timing claims are tested here.

Contract and failure classes: half-open affine range updates `x -> mul*x + add`
and range sums modulo 2^64; newer tags compose after older ones; empty updates
and queries are no-ops returning zero; panic before mutation on reversed or
out-of-range endpoints; `OrderedTree::query` folds `[l,r)` left to right under
an associative combine with neither commutativity nor inverses required.

Primary technique: exhaustive small-domain model agreement. Every array
`0..n` through length eight, for every `Kind`, every update interval, four
first tags and a whole-range add, then every query interval against the eager
model.

Secondary techniques: 2048 seeded overlapping updates for lengths 1/3/17/65
per `Kind` with wrapping values; literal temporal assign/add witnesses and the
README running example; exhaustive ordered folds over thirteen distinct
strings; rejection with state inspection; public empty-domain and temporal
contracts through `tests/contracts.rs`; the rendered example in
`examples/states.rs`; per-query comparison against an eager leaf array local to
the bench inside every benchmark process before its timing is admitted.

Oracle: an eager vector applies `mul*x+add` directly to each element and folds
each queried slice with wrapping addition. It uses no `Tag` method, tree
decomposition, push or composition. The bench oracle is the same shape and
shares no code with `Kind::Scan`.

Input/model/schedule domain and bounds: values `0..n` and `u64::MAX`; tags
add 3, assign 0, assign 7, `mul 2 add u64::MAX`, add 5, seeded assign/add;
lengths 0..=8 (exhaustive), 1/3/17/65 (seeded), 8 and 9 (literals), 13 strings;
eight declared benchmark cells with fixed seed 20.

Replay assets: deterministic enumerations and literals in `src/lib.rs` tests,
`tests/contracts.rs`, `examples/states.rs`, and the frozen seeds in
`src/lib.rs` (19) and `benches/portfolio.rs` (20); no random seeds are drawn
at run time.

What a green run establishes: agreement with the independent model on the
enumerated domains and literals, exact composition order, panics on the listed
invalid endpoints with unchanged state, order-preserving folds on every interval
of the string array, and query-level agreement of each candidate with the leaf
array on the declared streams. These are finite, bounded witnesses, not a proof.

Known gaps and delegated skills: no generated inputs beyond the seeded
schedules; no tests above length 65 except the benchmark streams; no exact
arithmetic rollback, concurrency, allocation failure injection, SQL window
semantics or production vectorized/exact-distinct/quantile execution;
`OrderedTree` is tested with one combine; rejection is tested for `update`
only, `query` bounds are covered by the exhaustive end-at-n cases and the
mutation below.

Concrete tests to add: generated long arrays with random update/query schedules
against the fold oracle; an `OrderedTree` rejection test for reversed endpoints;
a metamorphic check that `query(l, r) == query(0, r) - query(0, l)` modulo 2^64
across random endpoints; a non-string noncommutative combine.

Run all workspace gates listed in the task, including all-targets integration
checks. No tests merely assert private representation shape.

## Invariant audit

Mutation backstops below were each applied to `src/lib.rs`, run through
`cargo test -p segment-trees --all-targets`, and reverted; the named test is
the one that failed.

### exhaustive_small_ranges_and_tag_order

Claim and scope: for every `Kind`, every array `0..n` through length eight,
every update interval and first tag, then a whole-range add, every query
interval equals the eager model.
Plausible violation: reversed affine composition, missing push before a
partial descent, stale parent sum after a child change, off-by-one node or
query bounds, checked instead of wrapping arithmetic.
Observation: every `(ql, qr)` after both updates, per `Kind`.
Oracle and independence: eager vector update and slice fold; no shared code.
Comparator: exact u64 equality.
Smallest discriminating case: length two, whole-range add after `assign 7` on
`[0,1)`, then `query(0, 1)` on `Lazy`.
Mutation backstop: reversed `Tag::after` composition fails; removing the push
in `fold` fails; removing the pull in `change` fails; `r < n` in the `query`
bound panics on end-at-n queries; `mul * value + add` with checked arithmetic
panics on `mul 2 add u64::MAX`.

### seeded_differential_overlaps_wrapping_and_nonpowers

Claim and scope: 2048 seeded overlapping assign/add updates on arrays of
`u64::MAX` at lengths 1/3/17/65 keep whole-range and just-updated range sums
equal to the model, for every `Kind`.
Plausible violation: wrapping errors at large values, stale sums on non-power
of two shapes, composition errors across many overlapping tags.
Observation: `query(0, n)` and `query(l, r)` after each update.
Oracle and independence: the same eager vector model.
Comparator: exact u64 equality.
Smallest discriminating case: length one, first update `assign seed`, then
`query(0, 1)`.
Mutation backstop: reversed composition, removed pull and checked arithmetic
each fail; the removed-push mutation does not reach this test, which is why
the exhaustive test owns that claim.

### newer_tag_after_older_is_not_commutative

Claim and scope: `add.after(assign)` assigns the sum while `assign.after(add)`
assigns; a `Lazy` tree over the README example reports 30, 10, 42 and 24 at the
documented steps.
Plausible violation: commutative or reversed composition, pushing a composed
tag in the wrong order, re-applying a consumed tag.
Observation: `Tag` equality and four literal sums.
Oracle and independence: hand-computed literals from the README walkthrough.
Comparator: exact `Tag` and u64 equality.
Smallest discriminating case: `Tag::adding(3).after(Tag::assigning(4)) ==
Tag::assigning(7)`.
Mutation backstop: reversed composition fails at the `Tag` assertion; removed
push and removed pull fail at the literal sums.

### ordered_fold_rejects_reversed_right_accumulator

Claim and scope: `OrderedTree::query(l, r)` over thirteen distinct strings equals
`a[l..r].concat()` for every interval.
Plausible violation: combining the right accumulator on the wrong side, folding
the left accumulator after the node, wrong padding identity.
Observation: every `(l, r)` including empty intervals.
Oracle and independence: slice concatenation; no tree code.
Comparator: exact `String` equality.
Smallest discriminating case: length two, `query(0, 2)` across the root split.
Mutation backstop: `combine(&right, &data[r])` instead of
`combine(&data[r], &right)` fails only this test.

### invalid_update_panics_before_mutation

Claim and scope: `update(0, 4, assign 9)` on a length-three tree panics for every
`Kind` and leaves `query(0, 3) == 6`.
Plausible violation: bounds checked after a partial write, clamping instead of
rejecting, panicking only for some kinds.
Observation: `catch_unwind` result and the subsequent whole-range sum.
Oracle and independence: literal sum of the constructor input.
Comparator: `is_err()` plus exact u64 equality.
Smallest discriminating case: `Lazy`, `update(0, 4, assign 9)` then `query(0, 3)`.
Mutation backstop: moving the bounds assertion after a clamped mutation fails
with `left: 27, right: 6`.

### empty_and_temporal_contracts

Claim and scope: for every `Kind`, an empty tree accepts `update(0, 0)` and
returns zero; assign 4 then add 3 over nine ones gives 42 on `[2,8)`, and
assigning zero on `[3,7)` gives 35 overall; an empty `OrderedTree` returns the
identity.
Plausible violation: sentinel allocation indexed on empty domains, assignment
not clearing pending adds, partial assignment leaking into neighbours.
Observation: three literal sums per `Kind` and one identity fold.
Oracle and independence: hand-computed literals.
Comparator: exact u64 equality.
Smallest discriminating case: `RangeSum::new(&[], k).query(0, 0)`.
Mutation backstop: covered by the composition and push mutations above through
the public API; no separate mutation was run for the empty-domain branch.
