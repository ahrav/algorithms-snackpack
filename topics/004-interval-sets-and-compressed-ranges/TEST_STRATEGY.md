# Test strategy

Scope:

The immutable `PointOracle`, `FlatIntervalSet`, `PackedIntervalSet`,
`BoundaryEventSet`, and `BTreeIntervalSet` APIs: interval construction,
validation precedence, canonical projection, membership, cardinality, packed
run decoding, and union, intersection, and difference. The scope does not
include serialization, mutable updates after construction, threads, clocks,
files, sockets, unsafe code, or external services.

Contract and failure classes:

Each successful value denotes a finite set of `u32` points. Input intervals use
half-open `u64` endpoints in `0..=2^32`. Empty intervals are no-ops. Duplicate,
nested, overlapping, and adjacent inputs have Boolean set semantics. Canonical
output contains only nonempty, sorted, disjoint, non-adjacent intervals.

Construction rejects the first invalid interval. For each input, it checks the
start endpoint and then the end endpoint against `2^32` before checking reversed
order. It reports exact `OutOfDomain { index, endpoint }` or
`Reversed { index, start, end }` fields. Membership includes the start and
excludes the end. Cardinality is the sum of canonical run lengths in `u64`.
Set algebra returns the exact member set in canonical form.

Target failures include merging overlap but not adjacency, retaining empties,
losing the final point `u32::MAX`, narrow packed-end overflow, accepting a
reversed interval, reporting the wrong invalid endpoint or index, stopping tree
coalescing after one successor, processing equal-coordinate events separately,
binary-search endpoint errors, dropping a split prefix or tail in difference,
and circular agreement with a shared normalization helper.

Primary technique:

Differential comparison with `PointOracle`. The oracle inserts individual
points into a `BTreeSet<u32>`, validates endpoints independently, and groups
consecutive points without calling an optimized validator, sorter, merge helper,
packed decoder, event sweep, tree coalescer, membership search, or set-algebra
function. Each optimized representation is compared through exact canonical
intervals, cardinality, and point membership. Set algebra is also compared with
the oracle's independent point-set operations.

Secondary techniques:

Hand-derived examples and exact boundary cases pin the public contract.
Exhaustive short interval sequences cover the small finite domain named below.
A fixed-seed generated campaign samples larger valid cases. Permutation,
duplicate-injection, and empty-insertion checks are anchored metamorphic tests:
they compare both transformed and original results with a literal set, so a
constant implementation cannot pass. Direct public packed-run inspection pins
widened decoding. Direct canonical projection pins sorted, nonempty, disjoint,
and non-adjacent output. No fuzzing, Miri, Loom, Shuttle, Kani, async-time
control, snapshot test, or concurrency harness matches this safe, typed,
immutable contract.

Oracle:

Literal intervals, exact errors, exact member probes, and exact cardinalities
are the primary oracle for the running example and domain boundaries.
`PointOracle` is the independent differential oracle for tractable spans. Its
typed `union`, `intersection`, and `difference` use `BTreeSet` operations rather
than the optimized two-cursor functions. Canonical equality compares the exact
`Vec<Interval>` sequence. Tests do not sort, merge, or deduplicate candidate
output before comparing it. Packed representation checks use literal field and
decoded-end values.

Input/model/schedule domain and bounds:

- Literal cases include empty input, empty intervals at `0` and `2^32`, the
  running example, overlap, nesting, duplicate ranges, adjacency, one-point
  ranges, a bridge across several runs, and membership at both half-open
  endpoints.
- Error cases include a reversed interval, each endpoint above `2^32`, and an
  interval that is both reversed and out of domain. Exact index, endpoint, and
  validation precedence are checked.
- Terminal-domain cases include `[u32::MAX, 2^32)` and `[0, 2^32)`. The eager
  oracle does not expand the full-domain case.
- Exhaustive cases enumerate every valid sequence of zero through three
  intervals whose endpoints lie in `0..=6`. Every optimized representation is
  compared with the independent point model. The interval catalog contains all
  `0 <= start <= end <= 6` pairs, including empties. Ordered sequences retain
  duplicates and distinct arrival orders. Reversed and out-of-domain inputs are
  covered by exact tests instead of this valid-input enumeration.
- The generated campaign uses seeds `0`, `1`, `0x05eed004`,
  `0x9e3779b97f4a7c15`, and `u64::MAX`, with 64 cases per seed and 320 cases in
  total. Each case contains 48 generated valid intervals, five exact
  duplicates, and four empty anchors, for at most 57 inputs. Endpoints lie in
  `0..=256`, and membership probes cover `0..=257`. Each failure reports the
  seed, case index, and full input.
- Schedule domain: none. All operations are sequential and immutable.

The exhaustive test proves only its finite enumeration. The generated campaign
is sampled evidence and does not exhaust the `u32` domain.

Replay assets:

All literal regressions, the exhaustive enumerator, the fixed generator seed,
case count, and full failing inputs live in `tests/interval_sets.rs`. A generated
failure must be minimized into a named literal regression before its fix is
accepted. The test must print the seed and case index so the exact stream can be
replayed without depending on a changing default RNG.

What a green run establishes:

A green run establishes exact behavior for the named literal and terminal
boundary cases, complete agreement for the finite domain-six enumeration, and
agreement with the independent model for the recorded fixed-seed stream. It
also establishes the audited canonicalization, set-algebra, event-grouping,
tree-bridge, and packed-decoding claims at their stated inputs. This is bounded
evidence. It is not a proof for every `u32` interval collection, every possible
allocation failure, hostile serialized data, future mutation histories, thread
schedules, or performance claims.

Known gaps and delegated skills:

The point oracle cannot expand `[0, 2^32)`, so terminal full-domain behavior
uses direct canonical and arithmetic assertions instead of differential
agreement. Large generated domains are sampled. Allocation failure is not
injected. There is no decoder boundary to fuzz, no unsafe code for Miri, and no
concurrency contract for Loom or Shuttle. Serialized canonical-form validation
would need raw-byte fuzzing if added. A mutable API would need operation-sequence
model testing after every transition. Runtime, allocation, memory traffic,
crossover, and machine-mechanism claims belong to `BENCHMARK.md` and its
retained external evidence.

Concrete tests to add:

- `running_example_canonicalizes_to_two_exact_runs_in_every_representation`
- `error_precedence_reports_first_out_of_domain_endpoint_before_reversed_order`
- `empty_intervals_at_zero_and_domain_end_are_no_ops`
- `packed_full_domain_run_decodes_endpoint_with_widened_arithmetic`
- `bridge_interval_coalesces_predecessor_and_all_adjacent_successors`
- `grouped_boundary_events_preserve_coverage_across_canceling_adjacent_events`
- `exhaustive_sequences_of_up_to_three_intervals_match_independent_point_model_on_domain_six`
- `seeded_differential_cases_match_independent_point_model`
- `permutation_duplicates_and_empty_intervals_preserve_anchored_set`
- `union_intersection_and_difference_match_independent_point_algebra`
- `set_algebra_identities_hold_for_canonical_projection`
- `operations_preserve_domain_end_without_narrowing`
- `membership_observes_inclusive_start_and_exclusive_end`

The invariant-test audit below records what each claim-bearing test observes and
which wrong implementation it must reject.

## Invariant-test audit

### `running_example_canonicalizes_to_two_exact_runs_in_every_representation`

Claim and scope:

All five representations build the documented six-interval example as exactly
`[1,12)` and `[14,16)`, with cardinality 13 and correct membership at covered
points, both half-open boundaries, the gap, and `u32::MAX`.

Plausible violation:

A normalizer uses `<` instead of `<=` and leaves `[1,9)` adjacent to `[9,12)`,
retains `[6,6)`, drops `[14,16)`, or counts a boundary twice.

Observation:

The test reads each exact canonical vector, checks the canonical predicate,
reads cardinality, and probes membership at `0, 1, 3, 6, 11, 12, 13, 14, 15,
16`, and `u32::MAX`.

Oracle and independence:

The input, two expected runs, cardinality derived from those runs, and probes
come directly from the documented example. No candidate computes the expected
vector.

Comparator:

Exact `Vec<Interval>`, `u64`, and Boolean equality. The test does not normalize
candidate output before comparison.

Smallest discriminating case:

The boundary at `9` distinguishes overlap-only merging from canonical adjacency
merging. The empty `[6,6)` and real gap at points `12` and `13` distinguish
empty removal from gap removal.

Mutation backstop:

Change the merge predicate to `<`, stop after the first component, retain empty
ranges, or change `start <= point && point < end` at either boundary. The exact
vector, cardinality, or named probe fails.

Final verdict:

PASS (INFO). The test reaches every claimed feature of the documented example
and observes the complete local contract.

### `error_precedence_reports_first_out_of_domain_endpoint_before_reversed_order`

Claim and scope:

Every constructor reports the first invalid input. Within that interval, it
checks the start endpoint, then the end endpoint, then reversed order. It
returns the exact error fields.

Plausible violation:

An implementation checks reversal before domain, checks the end before the
start when both endpoints are invalid, reports the last invalid interval, or
returns a generic error without the rejected endpoint.

Observation:

The test compares the complete `Result` from all five constructors with literal
`OutOfDomain` and `Reversed` variants.

Oracle and independence:

Expected variants, indices, endpoints, and start/end fields are literal values
derived from the validation-order contract. No constructor supplies them.

Comparator:

Exact error variant and field equality. `is_err()` and display text are not used
as proxies.

Smallest discriminating case:

`[DOMAIN_END + 7, DOMAIN_END + 9)` distinguishes start-before-end checking.
`[DOMAIN_END + 7, 3)` distinguishes domain-before-reversed checking. A second
input interval pins the reported index.

Mutation backstop:

Swap endpoint checks, move the reversed check first, continue after an error,
or replace `index` with zero. One exact result fails for every constructor.

Final verdict:

PASS (INFO). Earlier ambiguous cases were replaced with inputs that discriminate
both endpoint and failure-class precedence.

### `empty_intervals_at_zero_and_domain_end_are_no_ops`

Claim and scope:

Empty intervals at zero, an interior coordinate, and the exclusive domain end
all construct the empty set in every representation.

Plausible violation:

A constructor rejects `[DOMAIN_END,DOMAIN_END)`, treats an empty interval as a
one-point range, narrows `DOMAIN_END` to zero, or reports nonzero cardinality.

Observation:

The test reads the exact empty canonical projection, cardinality zero, and
membership at `0`, `5`, and `u32::MAX`.

Oracle and independence:

The expected set is the literal empty vector. Half-open semantics directly make
all three inputs empty.

Comparator:

Exact empty sequence, exact `u64` cardinality, and Boolean nonmembership.

Smallest discriminating case:

`[0,0)` pins ordinary emptiness. `[DOMAIN_END,DOMAIN_END)` pins valid terminal
endpoint handling without expanding the point domain.

Mutation backstop:

Remove the empty filter, change empty to `start > end`, or cast an endpoint to
`u32` before validation. A projection, cardinality, or membership assertion
fails.

Final verdict:

PASS (INFO). The test observes both the semantic result and terminal-boundary
acceptance.

### `packed_full_domain_run_decodes_endpoint_with_widened_arithmetic`

Claim and scope:

The packed representation encodes `[0,2^32)` as start zero and
`length_minus_one == u32::MAX`, decodes the exclusive end in `u64`, reports
cardinality `2^32`, and preserves the terminal one-point run.

Plausible violation:

Encoding length directly cannot represent `2^32`. Adding start, encoded length,
and one in `u32` wraps. Converting the exclusive end to `u32` loses `2^32`.

Observation:

The test reads both packed fields, decoded intervals, total cardinality, and
membership at `0` and `u32::MAX`.

Oracle and independence:

The expected encoded fields and decoded endpoints come from the literal
`length_minus_one` equation. No production decode creates the expected values.

Comparator:

Exact scalar, interval, cardinality, and Boolean equality.

Smallest discriminating case:

The full-domain run forces encoded length `u32::MAX` and decoded end `2^32`.
`[u32::MAX,2^32)` separately forces widened addition with a nonzero start.

Mutation backstop:

Remove either widening conversion, encode raw length, or omit the final `+ 1`.
The field, decode, cardinality, or terminal membership assertion fails.

Final verdict:

PASS (INFO). The test directly observes the representation fact and its semantic
consequences.

### `bridge_interval_coalesces_predecessor_and_all_adjacent_successors`

Claim and scope:

The tree constructor absorbs one predecessor and every successor connected by
the bridge `[3,10)`, leaving one run `[1,11)` with cardinality 10.

Plausible violation:

Tree insertion checks only the predecessor, absorbs only the first successor,
uses strict overlap instead of connectivity, or forgets to remove an absorbed
node.

Observation:

The test reads tree run count, the full canonical projection, and cardinality.

Oracle and independence:

`[1,11)` and cardinality 10 are hand-derived from the four literal ranges. No
tree traversal or coalescing helper computes the expectation.

Comparator:

Exact run count, exact interval sequence, and exact cardinality.

Smallest discriminating case:

Existing runs `[1,3)`, `[5,7)`, and `[9,11)` plus bridge `[3,10)` require one
predecessor merge and two successive successor merges.

Mutation backstop:

Delete the successor loop, break after one successor, or use `>` where
adjacency needs `>=`. More than one run remains or cardinality changes.

Final verdict:

PASS (INFO). The setup reaches the claimed multi-node coalescing path and the
observation detects partial cleanup.

### `grouped_boundary_events_preserve_coverage_across_canceling_adjacent_events`

Claim and scope:

Grouped events keep coverage active where starts and ends share coordinates,
even with duplicate coverage. The result is exactly `[1,7)`.

Plausible violation:

An event sweep handles one event at a time, emits a false boundary between an
end and a start, collapses depth to a Boolean and closes too early, or fails to
aggregate duplicate deltas.

Observation:

The test reads the exact canonical projection, cardinality six, and membership
at the canceling coordinates `3` and `5`.

Oracle and independence:

The expected run and membership facts are literal consequences of the five
input intervals. No event list or sweep counter computes them.

Comparator:

Exact interval sequence, exact cardinality, and Boolean membership.

Smallest discriminating case:

At coordinates `3` and `5`, at least one end and one start cancel while coverage
remains positive. Duplicated `[1,3)` and `[3,5)` also require depth above one.

Mutation backstop:

Emit after each event, order ends before starts without grouping, or clamp depth
to one. The output splits, cardinality changes, or a boundary probe fails.

Final verdict:

PASS (INFO). The exact result detects the false-gap failure instead of checking
only final depth or absence of panic.

### `exhaustive_sequences_of_up_to_three_intervals_match_independent_point_model_on_domain_six`

Claim and scope:

For every ordered sequence of zero through three valid intervals with endpoints
in `0..=6`, every optimized representation agrees with the point model on
canonical intervals, cardinality, and membership through point `7`.

Plausible violation:

An implementation mishandles order, duplicates, empties, nesting, overlap,
adjacency, singleton ranges, or a gap within this finite domain.

Observation:

For every enumerated input, the helper reads the full canonical projection,
checks its canonical predicate, reads cardinality, and probes membership at
every point in `0..=7`.

Oracle and independence:

`PointOracle` enumerates individual points in a `BTreeSet` and shares no
correctness-critical validator, sort, merge, event, tree, packed, or algebra
helper with the optimized candidates.

Comparator:

Exact canonical sequence, exact cardinality, and pointwise Boolean equality.
Candidate output is not normalized before comparison.

Smallest discriminating case:

The catalog includes empties, one-point ranges, `[0,1)` followed by `[1,2)`,
duplicates, nested pairs, reversed arrival order, and three-range bridges.

Mutation backstop:

Change adjacency handling, omit sorting, remove deduplication, stop tree merging,
or weaken one membership comparison. At least one complete enumerated sequence
fails at the intended assertion.

Final verdict:

PASS (INFO). The enumeration is systematic for the declared finite catalog. It
makes no claim beyond sequence length three or endpoint six.

### `seeded_differential_cases_match_independent_point_model`

Claim and scope:

All optimized candidates agree with the independent model for 320 fixed-seed
cases. Each case contains 48 generated intervals plus injected duplicates and
empties with endpoints in `0..=256`.

Plausible violation:

A defect needs more than three intervals, a longer merge chain, an arbitrary
arrival order, repeated duplicates, or a larger bounded span before it changes
the result.

Observation:

Each case compares exact canonical projection, canonical shape, cardinality,
and membership at every point in `0..=257`. Failure messages include the full
input, hexadecimal seed, and case index.

Oracle and independence:

The eager `BTreeSet` point model is independent of all optimized construction
paths. Fixed seeds and the in-test LCG make the input stream reproducible.

Comparator:

Exact interval sequence, exact `u64` cardinality, and pointwise Boolean
equality. Checksums and candidate-side normalization are absent.

Smallest discriminating case:

A one-point membership or one-endpoint projection difference is the smallest
observable failure. Named literal tests retain the minimized adjacency,
empty-range, bridge, packed-end, and grouped-event faults.

Mutation backstop:

Drop one absorbed run, skip one event, alter an endpoint comparison, or count
duplicate coverage. A generated input that exercises the mutation fails with a
replay identity; the exhaustive and literal tests remain the deterministic
backstop for their smaller fault classes.

Final verdict:

PASS (INFO). This is replayable sampled evidence over the fixed stream, not an
exhaustive or proof claim.

### `permutation_duplicates_and_empty_intervals_preserve_anchored_set`

Claim and scope:

Reordering input, duplicating every original interval, and adding empty
intervals at zero and `2^32` do not change any representation's exact set.

Plausible violation:

Construction depends on arrival order, removes only adjacent duplicates,
counts duplicate coverage, or treats an empty terminal interval as data.

Observation:

The original and transformed inputs are each compared with the same exact
canonical vector, cardinality, canonical predicate, and membership probes.

Oracle and independence:

`[2,10)` and `[20,21)` are hand-derived anchors. The transformed result is not
computed from the original candidate output.

Comparator:

Exact interval sequence, exact cardinality, and Boolean membership. The test
does not convert observed output to a set.

Smallest discriminating case:

The transformed input contains nonadjacent duplicates, a changed arrival order,
an interior overlap, and both ordinary and terminal empty intervals.

Mutation backstop:

Remove sorting, deduplicate only neighboring input records, add lengths before
canonicalization, or retain empty intervals. The anchored exact result fails.

Final verdict:

PASS (INFO). The literal anchor prevents a constant or mutually wrong pair of
metamorphic executions from satisfying the relation.

### `union_intersection_and_difference_match_independent_point_algebra`

Claim and scope:

For all 25 left/right representation pairs, union, intersection, and left
difference equal the independent point-set operations and return exact canonical
runs.

Plausible violation:

Intersection advances the wrong cursor, union fails to merge cross-input
adjacency, difference loses an uncovered prefix or tail, or an operand exposes
a malformed projection that an intermediate normalizer hides.

Observation:

The test first compares every operand projection with its point oracle. It then
passes each candidate directly to the generic operation and reads the exact
result vector for all representation pairs.

Oracle and independence:

Expected values come from `BTreeSet::union`, `BTreeSet::intersection`, and
`BTreeSet::difference` through typed `PointOracle` methods. The optimized
two-cursor operations do not supply expected values.

Comparator:

Exact canonical `Vec<Interval>` equality. No Flat re-canonicalization sits
between a candidate projection and the operation under test.

Smallest discriminating case:

The literals include partial overlap, containment, gaps, a right interval that
cuts a left run, and terminal pieces on both sides. Difference must emit
`[1,3)`, `[10,12)`, `[13,14)`, and `[21,22)`.

Mutation backstop:

Advance both intersection cursors, omit union coalescing, discard a difference
prefix, stop before its tail, or weaken an operand projection. An exact oracle
comparison fails.

Final verdict:

PASS (INFO). Earlier masking Flat conversions were removed. The test now
observes both operand projections and direct generic-operation output.

### `set_algebra_identities_hold_for_canonical_projection`

Claim and scope:

For the fixed `a`, `b`, and `c`, union and intersection are commutative and
associative, self-difference is empty, and difference plus intersection
partitions `a`.

Plausible violation:

A tie rule makes union order-dependent, cursor advancement makes grouping
matter, self-difference retains a boundary fragment, or difference and
intersection omit part of `a`.

Observation:

The test reads and compares complete canonical projections from both sides of
each law. The partition law also compares with the literal operand projection.

Oracle and independence:

The oracle is metamorphic. Commuted and regrouped executions must agree. The
partition identity is anchored to `a`, so a constant empty implementation
cannot pass every assertion.

Comparator:

Exact canonical interval sequence. Order is part of the canonical contract and
is not sorted away in the test.

Smallest discriminating case:

The three operands contain overlapping, adjacent-after-union, disjoint, and
cross-cutting runs. The `difference(a,b)` and `intersection(a,b)` pair must
reconstruct all of `a` without overlap.

Mutation backstop:

Bias equal starts to one operand, advance the wrong end cursor, retain a
self-difference fragment, or drop one partition piece. A law or anchor fails.

Final verdict:

PASS (INFO). The test supports these algebraic relations for the fixed operands.
The independent algebra test carries the broader correctness oracle.

### `operations_preserve_domain_end_without_narrowing`

Claim and scope:

Union, intersection, and difference preserve the exclusive endpoint `2^32`
when a wide left set and packed right set overlap near the domain end.

Plausible violation:

An operation narrows an endpoint to `u32`, wraps `2^32` to zero, subtracts from
the wrong width, or drops the terminal tail during cursor advancement.

Observation:

The test reads the exact canonical output of all three operations.

Oracle and independence:

The three expected intervals are hand-derived literals from
`[2^32-10,2^32)` and `[2^32-4,2^32)`.

Comparator:

Exact interval sequence equality with `u64` endpoints.

Smallest discriminating case:

Both operands end at `DOMAIN_END`, while their starts differ. Difference must
retain `[DOMAIN_END-10,DOMAIN_END-4)` and the other operations must retain the
terminal endpoint.

Mutation backstop:

Cast either end to `u32`, compute a packed end without widening, or advance past
the remaining left prefix. One literal endpoint changes.

Final verdict:

PASS (INFO). The test does not rely on the eager oracle and directly covers the
full-domain endpoint that the oracle cannot expand.

### `membership_observes_inclusive_start_and_exclusive_end`

Claim and scope:

Every representation includes each interval start, excludes each interval end,
excludes gaps, and includes `u32::MAX` in the terminal one-point run.

Plausible violation:

Membership uses `start < point`, `point <= end`, selects the wrong predecessor,
or narrows the terminal exclusive end.

Observation:

The test directly reads Boolean membership at zero, ordinary starts and ends,
one gap, the last point before an end, and `u32::MAX`.

Oracle and independence:

Every expected Boolean is a literal consequence of `[0,1)`, `[5,8)`, and
`[u32::MAX,2^32)`. No candidate membership helper supplies expectations.

Comparator:

Exact Boolean equality at each named point. Cardinality or nonempty status is
not used as a proxy.

Smallest discriminating case:

`[0,1)` distinguishes inclusive start zero from exclusive end one. `[5,8)`
adds an interior gap and nonzero start. The terminal singleton pins the maximum
point.

Mutation backstop:

Change either boundary comparison, use the insertion point instead of its
predecessor, or decode the terminal end in `u32`. A named probe fails.

Final verdict:

PASS (INFO). The test reaches all boundary classes named in its claim and checks
them directly.
