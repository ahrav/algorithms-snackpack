# Test strategy

Scope:

The four pure batch functions in this crate. The tests cover range validation,
half-open update semantics, wrapping arithmetic, exact output order,
multiplicity, and agreement between the independent eager model and the three
boundary candidates.

Contract and failure classes:

For valid `RangeUpdate` values, add each `delta` to `[start, end)` with wrapping
`i64` arithmetic and return a new vector with the input's exact length and
order. Empty ranges and zero deltas are valid no-ops. For an invalid batch,
return the first input-order `RangeError` where `start > end` or
`end > input.len()`. Target failures include inclusive-end writes, rejection of
valid boundary ranges, wrong overflow behavior, the wrong invalid-update index,
lost repeated events, wrong difference direction, omitted end events,
order-dependent addition, and output reordering.

Primary technique:

Differential testing against `apply_reference`. The reference model directly
loops over every covered element. It shares no correctness-critical helper with
the boundary candidates.

Secondary techniques:

Literal boundary and regression examples; exhaustive enumeration over a small
finite domain; a deterministic 10,000-case generated campaign; and an anchored
metamorphic permutation test over all 24 orders of four valid updates. No
parser, unsafe code, shared mutable state, threads, clocks, files, or external
services enter the contract, so byte fuzzing, Miri, Loom, Shuttle, Kani, async
time control, test doubles, and integration harnesses do not target a stated
failure class here.

Oracle:

Use hand-derived literal vectors and exact `RangeError` values for finite
boundaries. Use `apply_reference` for valid generated cases. The reference
implementation validates directly, clones the input, and eagerly updates each
element. It does not call the dense, in-place, event, scan, or comparator logic.
Every comparison checks `Result<Vec<i64>, RangeError>` or the full ordered
`Vec<i64>`. No sort, set, multiset, or approximate comparator can erase a
contract-relevant difference.

Input/model/schedule domain and bounds:

- Exhaustive: lengths `0..=4`; every base vector over `{-1, 0, 1}`; every valid
  `0 <= start <= end <= len`; deltas `{-1, 0, 1}`; update-sequence lengths 0, 1,
  and 2. The test checks exactly 196,261 cases.
- Generated: LCG seed `2026083002`; exactly 10,000 cases; length `0..=128`;
  update count `0..=64`; valid endpoints across the full per-case range; base
  values and deltas drawn from `i64::MIN`, `i64::MAX`, `-1`, `0`, `1`, and
  arbitrary `i64` bit patterns.
- Metamorphic: all 24 permutations of one fixed four-update valid batch.
- Schedule domain: none. The functions are sequential and pure.

Replay assets:

The literal cases, exhaustive domain, generator code, seed `2026083002`, case
index, and full failing input in assertion diagnostics are checked in. Any
generated failure must be minimized into a named literal regression before the
fix is accepted.

What a green run establishes:

Every literal assertion passed. Every optimized candidate matched the
independent reference for all 196,261 exhaustive cases and the 10,000 seeded
cases explored. Every implementation returned the exact result for all 24
declared permutations. No checked property failed in those domains. This is
bounded evidence, not a proof for every vector or update sequence.

Known gaps and delegated skills:

The exhaustive domain has at most two updates and small values. The seeded
campaign samples larger valid inputs but does not exhaust them. Invalid batches
are covered by three exact examples, not every invalid shape. The pure return type
cannot distinguish full prevalidation from an implementation that mutates a
private temporary and later returns the same error; source review and benchmark
work counts cover that internal sequencing claim. Performance belongs to
`BENCHMARK.md`. A targeted mutation run would be useful after publication, but
mutation execution and survivor triage belong to the mutation-testing
operations skill.

Concrete tests to add:

- `literal_half_open_boundaries_produce_exact_sequence`
- `valid_len_boundary_empty_ranges_and_zero_deltas_are_no_ops`
- `overflow_uses_wrapping_i64_arithmetic`
- `first_invalid_update_is_reported_exactly`
- `duplicate_and_canceling_endpoints_preserve_multiplicity`
- `permuting_valid_updates_preserves_exact_output`
- `exhaustive_small_domain_matches_independent_reference`
- `seeded_differential_cases_match_independent_reference`

## Invariant test review

### `literal_half_open_boundaries_produce_exact_sequence`

Claim and scope:

For the fixed four-element input and four valid updates, every implementation
uses half-open endpoints and returns exactly `[0, 18, 28, 45]`.

Plausible violation:

An implementation treats `end` as inclusive, omits the stop boundary at index
3, or applies the empty `[2, 2)` update.

Observation:

The complete `Result<Vec<i64>, RangeError>` from each of the four public
functions.

Oracle and independence:

The expected vector is a hand-derived literal. It does not call any candidate,
scan helper, event combiner, or shared range helper.

Comparator:

Exact `Result` equality and exact sequence equality. Position and value both
matter.

Smallest discriminating case:

The existing `[0, 1) += -15` update separates position 0 from position 1. The
`[1, 3)` update separates index 2 from the excluded index 3. The `[2, 2)`
update anchors empty-range behavior.

Mutation backstop:

Changing a range loop to include `end`, deleting a right-boundary subtraction,
or emitting boundaries for an empty range must fail at the exact-vector
assertion. Finding: INFO. The test reaches the claimed behavior and observes it
directly.

### `valid_len_boundary_empty_ranges_and_zero_deltas_are_no_ops`

Claim and scope:

For a two-element input and an empty input, ranges with `start == end`, including
`len..len`, are valid no-ops, and a zero delta over the full range is a valid
no-op.

Plausible violation:

Validation rejects `start == end` or `end == len`; boundary code records only
one side of an empty update; or the empty input path indexes position 0.

Observation:

The exact success result from all four functions for the nonempty and empty
inputs.

Oracle and independence:

The unchanged input vectors are direct consequences of the no-op contract and
are written as literals. No implementation supplies the expectation.

Comparator:

Exact `Result` and ordered-vector equality.

Smallest discriminating case:

`RangeUpdate { start: 0, end: 0, delta: 1 }` on `[]` distinguishes a valid
empty range from an out-of-bounds access. `2..2` on a length-two input checks
the upper boundary.

Mutation backstop:

Replacing `start > end` with `start >= end`, replacing `end > len` with
`end >= len`, or adding a start boundary without suppressing the matching empty
end must fail. Finding: INFO. The examples check both empty-input and `len`
boundaries.

### `overflow_uses_wrapping_i64_arithmetic`

Claim and scope:

Every implementation performs all base, boundary, difference, event, and scan
arithmetic modulo `2^64` for two fixed edge-value cases.

Plausible violation:

A candidate uses checked, saturating, or debug-overflowing arithmetic, or
negates `i64::MIN` with ordinary unary negation.

Observation:

Both exact returned vectors and the absence of an overflow panic for all four
functions.

Oracle and independence:

The expected vectors `[i64::MIN, 0, i64::MIN, i64::MAX]` and
`[i64::MIN, 0]` are derived by modular addition and stored directly. They do
not reuse candidate arithmetic.

Comparator:

Exact ordered `i64` sequence equality.

Smallest discriminating case:

`i64::MAX + 1` must become `i64::MIN`. The direct `[0, 1) += i64::MIN`
case forces every boundary candidate to create the interior stop value through
wrapping subtraction. The overlapping case also forces wrapped intermediate
boundary and scan values, not only a wrapped final base addition.

Mutation backstop:

Replace one `wrapping_add` or `wrapping_sub`, including
`0_i64.wrapping_sub(delta)`, with checked, saturating, or ordinary arithmetic.
The test must panic or return a different literal vector. Finding: INFO. The
case checks both positive overflow and the `i64::MIN` boundary.

### `first_invalid_update_is_reported_exactly`

Claim and scope:

For three fixed invalid batches over a length-three input, every implementation
returns the first invalid update in input order with the exact index, endpoints,
and length. The caller input remains unchanged.

Plausible violation:

A candidate reports the last invalid update, checks only `end > len`, groups
errors by predicate instead of input order, panics, or returns a partial success
vector.

Observation:

The complete `RangeError` and the caller-owned input after the call.

Oracle and independence:

All three expected errors are literal structs derived from the public
validation predicate. The input-preservation expectation is a literal array.

Comparator:

Exact `RangeError` field equality. Error kind alone or `is_err()` would be too
weak.

Smallest discriminating case:

The first batch places a valid update before a reversed range and an additional
out-of-bounds range after it. The second batch makes `len..len` valid before a
later `end > len` error. The third puts an out-of-bounds range before a reversed
range, so a validator cannot group failures by predicate instead of input
order.

Mutation backstop:

Reverse validation order, continue after finding an error, change either
comparison boundary, or return a generic error. The exact struct assertion must
fail. Finding: INFO for the public error claim. The test cannot observe whether
a discarded private result was partially computed before the error.

### `duplicate_and_canceling_endpoints_preserve_multiplicity`

Claim and scope:

All four implementations preserve repeated boundary contributions and wrapped
cancellation for the fixed six-update batch.

Plausible violation:

The event candidate stores events in a set, a map overwrites one equal-position
event, or boundary aggregation uses ordinary arithmetic and mishandles two
`i64::MIN` additions.

Observation:

The exact returned sequence `[5, 6, 10, 11, 12]` from every implementation.

Oracle and independence:

The expected sequence is hand-derived. Two `+7` updates and one `-14` update
cancel only when both `+7` records remain. Two full-range `i64::MIN` updates
cancel modulo `2^64`.

Comparator:

Exact sequence equality. A set or multiset comparator is forbidden because
position is contractual.

Smallest discriminating case:

The duplicate `[1, 4) += 7` entries paired with `[1, 4) += -14` distinguish
multiplicity from deduplication. The final `[2, 5) += 3` prevents a degenerate
all-input result from passing.

Mutation backstop:

Call `dedup`, overwrite rather than add an equal-position event, drop either
duplicate, or replace wrapped boundary addition. The literal output must
change. Finding: INFO. The case rejects the smallest serious sorted-event data
loss fault.

### `permuting_valid_updates_preserves_exact_output`

Claim and scope:

For the fixed valid four-update batch, all 24 update orders return exactly
`[2, 1, 6, 4, -1]` under every implementation.

Plausible violation:

A candidate assigns rather than adds a delta, retains only the first or last
event at a boundary, or applies an order-sensitive overwrite during
aggregation.

Observation:

The exact output for each candidate and each generated permutation, plus 24
successful leaves and 24 distinct update tuples in an independent set.

Oracle and independence:

The expected vector is a hand-derived literal. The relation is anchored to that
literal, so a constant or no-op implementation cannot pass by permutation
invariance alone.

Comparator:

Exact ordered-vector equality for every permutation.

Smallest discriminating case:

Overlapping positive and negative updates share positions, while the empty
update checks that moving a no-op does not change the result. Enumerating all 24
orders exposes first-wins and last-wins behavior.

Mutation backstop:

Replace one accumulation with assignment, skip every event after the first at a
position, or remove the swap restoration in the permutation generator. The
literal output, leaf count, or distinct-set assertion must fail. Finding: INFO.
The anchored metamorphic oracle rejects degenerate permutation-invariant
implementations.

### `exhaustive_small_domain_matches_independent_reference`

Claim and scope:

For all 196,261 declared small valid cases, each boundary candidate returns the
same full ordered vector as the eager reference.

Plausible violation:

A candidate uses the wrong right boundary, scans in the wrong direction, loses
one of two overlapping updates, mishandles length zero, or handles only positive
deltas.

Observation:

Each optimized candidate's complete success result for every enumerated base
vector and update sequence.

Oracle and independence:

`apply_reference` directly updates each covered element. It shares no helper
with the difference, dense-sidecar, or sorted-event implementations. The
generator independently enumerates valid ranges and values.

Comparator:

Exact ordered `Vec<i64>` equality after every case. For each length, the test
checks base-vector length, value membership, uniqueness, and cardinality. It
also checks update-option cardinality, tuple uniqueness, endpoint bounds, and
delta membership. Those checks establish that neither finite universe was
silently reduced before the final case-count assertion.

Smallest discriminating case:

Length two with `[0, 1) += 1` distinguishes half-open from inclusive behavior;
two updates over one position distinguish accumulation from replacement. Both
occur in the enumerated domain.
Mutation backstop:

Delete an end-boundary update, reverse a difference subtraction, start a scan
at the wrong index, skip the second update, collapse decoded bases to zero,
duplicate one generated option, or replace endpoint pairs while preserving the
loop count. The model comparison, universe checks, or the `196_261` count must
fail. Finding: INFO. The oracle is structurally independent and the domain is
explicit.

### `seeded_differential_cases_match_independent_reference`

Claim and scope:

For exactly 10,000 deterministic valid cases from seed `2026083002`, every
length from 0 through 128, and every update count from 0 through 64, each
boundary candidate returns the eager reference's full ordered vector. Coverage
anchors require empty ranges, `end == len`, overlapping nonempty ranges,
`i64::MIN`, `i64::MAX`, and values outside the fixed edge-value set.

Plausible violation:

A candidate fails only on large overlap, endpoints at `len`, empty input,
arbitrary bit patterns, `i64::MIN`, `i64::MAX`, or a longer repeated update
sequence that the exhaustive domain does not contain.

Observation:

The complete candidate result for each case, with candidate name, case index,
seed, input, and updates in the failure diagnostic.

Oracle and independence:

`apply_reference` is the independent eager model. The LCG and typed valid-range
generator do not call optimized candidate helpers.

Comparator:

Exact ordered `Vec<i64>` equality. No normalization occurs. Independent sets
and flags check the declared generated-stream coverage anchors.

Smallest discriminating case:

The campaign mixes fixed edge values with arbitrary `i64` bit patterns. The
endpoint generator can draw any valid half-open range. The coverage anchors
confirm empty ranges, upper-bound endpoints, and overlap occurred in this fixed
stream. A failure must be minimized into a literal regression so the smallest
observed case becomes durable.

Mutation backstop:

Collapse the generator to empty cases, cap update processing below 64, remove
wrapping on one optimized path, skip `end == len`, or collapse equal-position
events. The coverage anchors or model comparison must expose the mutation when
its triggering case is sampled, and the minimized case then becomes a
permanent exact test. Finding: INFO with a sampled-evidence limit. A green
campaign is not an exhaustive proof.
