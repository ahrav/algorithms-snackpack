# Test strategy

Scope:

The six immutable functions `count_bounded_subarrays`, `recompute_reference`,
`quadratic_early_exit`, `direct_sliding`, `oversized_reset_sliding`, and
`prefix_binary_search`. Every function accepts `&[u64]` and a `u128` budget and
returns the exact `u128` number of nonempty contiguous half-open ranges whose
mathematical sum is at most that budget.

The scope does not include signed inputs, mutation, serialization, unsafe code,
threads, clocks, files, sockets, streaming-time semantics, or external
services.

Contract and failure classes:

Input order is significant. Equal values at different indices still identify
distinct endpoint pairs. Empty input returns zero. Zero values can create many
valid ranges at budget zero and duplicate prefix sums. A value larger than the
budget cannot appear in a valid range but can separate valid runs on each side.
Sums and counts use `u128`, including when a pair of `u64::MAX` values exceeds
`u64::MAX`.

Target failures include counting empty ranges, using `< budget` instead of
`<= budget`, adding one result per right endpoint instead of all valid suffixes,
repairing only once, stopping repair after removing a zero, failing to reset
past an oversized value, letting an oversized value contaminate the next run,
leaving the left boundary behind after a long exact-budget prefix, overflowing
a `u64` sum, narrowing the answer to `usize`, searching duplicate prefix values
with equality instead of a lower bound, including the current end as a prefix
start, and sharing optimized state with the reference.

Primary technique:

Differential comparison with `recompute_reference`. The reference recomputes
the sum of every candidate slice from scratch. It does not keep a running sum,
stop early, call another candidate, build prefixes, use `partition_point`, move
a persistent left cursor, or use an oversized reset. That separation makes it a
useful correctness oracle on bounded inputs.

Secondary techniques:

Literal examples pin the public contract and wide arithmetic. An exact finite
enumeration checks every small array-budget pair in the declared domain. A
fixed-seed campaign samples larger arrays and includes `u64::MAX`. Anchored
metamorphic tests check reversal, budget monotonicity, positive scaling, and the
all-fit triangular formula. The reversal and scaling tests compare against a
reference result from the untransformed input, so a constant candidate cannot
pass merely by preserving its own wrong output.

No byte fuzzing, Miri, Loom, Shuttle, Kani, async-time control, snapshot test,
or concurrent schedule harness matches this safe, typed, immutable API. A
deterministic generated campaign is retained because larger value and length
combinations still matter.

Oracle:

Literal counts are the primary oracle for the running example, empty and zero
cases, oversized separators, and the wide-sum edge. The triangular equation is
the independent oracle when the budget covers the full input sum.

`recompute_reference` is the oracle for exhaustive, generated, reversal, and
scaling checks. It shares the public types and mathematical contract with the
optimized paths but no optimized traversal helper. Comparators use exact
`u128` equality. Checksums, tolerances, sorted projections, and candidate-made
normalization do not enter correctness tests.

Input/model/schedule domain and bounds:

- Literal cases include `[]`, one and six zeros, zero-separated positive
  values, the running input `[2, 0, 3, 2, 1]`, one and several oversized
  values, eleven ones followed by `u64::MAX` and zero, and two `u64::MAX`
  values.
- The exhaustive domain contains every array length in `0..=6`, every value in
  `0..=3`, and every budget in `0..=19`. There are
  `(1 + 4 + 4^2 + ... + 4^6) * 20 = 109,220` array-budget cases.
- The generated campaign uses seeds `0`, `1`,
  `0x0000_0005_5A17_D0C5`, `0x9E37_79B9_7F4A_7C15`, and `u64::MAX`.
  It runs 128 cases per seed, or 640 cases. Length is in `0..=24`. Ordinary
  values are in `0..=32`; deterministic positions can inject `u64::MAX`.
  Budgets include zero, small sampled values, values through 512,
  `u64::MAX`, and `u128::MAX`.
- Metamorphic fixtures include empty, zero, mixed, oversized, and wide-sum
  arrays. Budget probes cross exact sums and the `u64` boundary.
- Schedule domain: none. All calls are sequential and immutable.

The exhaustive test proves only the named finite enumeration. The generated
campaign is sampled evidence and does not exhaust all `u64` arrays or `u128`
budgets.

Replay assets:

All literal regressions, the base-four exhaustive enumerator, generator code,
seed family, case count, and full failing inputs live in
`tests/bounded_subarrays.rs`. Generated assertions print the candidate name,
seed, case index, values, and budget. Any generated failure must be minimized
into a named literal regression before its fix is accepted.

What a green run establishes:

A green run establishes exact output for every literal and exhaustive case,
agreement with the independent recompute oracle for the fixed 640-case stream,
and the named metamorphic relations for their fixed fixtures. It also
establishes that the default entry point agrees with the named candidates in
those domains.

This is bounded evidence. It is not a proof for every possible slice, target,
compiler, allocation outcome, or signed-value extension. It does not establish
runtime, allocation count, instruction shape, cache behavior, or a universal
candidate ranking.

Known gaps and delegated evidence:

The prefix path allocates, but ordinary tests cannot force allocation failure
portably. The type excludes signed inputs, so the signed counterexample is a
documented contract boundary rather than a runtime test. There is no
serialization or unsafe code to fuzz, no mutable state machine to model, and no
concurrent schedule to explore. Runtime and workload crossover belong to
`BENCHMARK.md`. Static disassembly and dynamic profile attempts are benchmark
evidence, not correctness oracles.

Concrete tests:

- `running_example_counts_eleven_half_open_subarrays`
- `empty_zero_only_and_duplicate_prefix_boundaries_follow_the_contract`
- `oversized_values_split_independent_valid_runs`
- `maximum_value_separator_after_positive_prefix_resets_state`
- `u128_accumulation_preserves_mathematical_sums_above_u64_max`
- `exhaustive_lengths_zero_through_six_match_on_frozen_small_domain`
- `seeded_differential_cases_match_the_independent_reference`
- `reversing_values_preserves_the_number_of_valid_ranges`
- `increasing_the_budget_never_decreases_the_count`
- `scaling_values_and_budget_by_a_positive_factor_preserves_the_count`
- `a_budget_covering_the_total_sum_counts_the_triangular_number`

## Invariant-test audit

### `running_example_counts_eleven_half_open_subarrays`

Claim and scope:

Every implementation counts exactly 11 valid nonempty half-open subarrays of
`[2, 0, 3, 2, 1]` at budget `5`.

Plausible violation:

The implementation counts only the repaired window, uses a strict bound, drops
a suffix that starts after zero, or counts an empty range.

Observation:

The test observes the exact `u128` result from the default and all five named
implementations.

Oracle and independence:

Eleven is hand-derived from the documented end-by-end contribution sequence
`1, 2, 3, 3, 2`. No production function supplies the expected value.

Comparator:

Exact scalar equality. A tolerance or checksum would be weaker and is not used.

Smallest discriminating case:

This five-value example combines a zero, an exact-budget window, one repair,
and a later two-step repair whose first removal is zero.

Mutation backstop:

Replace `<=` with `<`, add one per end, or change the repair loop to one `if`.
The literal count fails.

Final verdict:

PASS (INFO). The test observes the complete public result for the lesson's
running example.

### `empty_zero_only_and_duplicate_prefix_boundaries_follow_the_contract`

Claim and scope:

Empty input returns zero. Six zeros at budget zero produce 21 nonempty ranges.
For `[0, 0, 1, 0]` at budget zero, the two zero runs produce four ranges.

Plausible violation:

The implementation counts empty ranges, treats zero removal as repair
completion, searches only one equal prefix, or includes a positive value at
budget zero.

Observation:

Exact results are read for empty input at zero and maximum budgets, one zero,
six zeros, and a split zero fixture.

Oracle and independence:

The expected counts `0`, `1`, `21`, and `4` come from literal enumeration and
the triangular formula, not from a candidate.

Comparator:

Exact `u128` equality for every implementation.

Smallest discriminating case:

`[0]` separates empty from nonempty counting. `[0, 0, 1, 0]` combines duplicate
prefix sums with a positive separator.

Mutation backstop:

Count `end - left + 1`, use equality search in prefixes, or accept a positive
value at budget zero. A literal result fails.

Final verdict:

PASS (INFO). The test observes empty, zero-only, and duplicate-prefix behavior.

### `oversized_values_split_independent_valid_runs`

Claim and scope:

A value larger than the budget belongs to no valid range and separates valid
ranges on each side. Repeated oversized values do not contaminate later state.

Plausible violation:

The reset path leaves its sum or left boundary stale, skips the first value
after a separator, or counts the oversized singleton.

Observation:

The test reads exact counts for `[2, 100, 0, 3]`, alternating `u64::MAX` and
zero at budget zero, and an all-oversized input.

Oracle and independence:

Expected values `4`, `2`, and `0` are literal endpoint counts. They do not come
from the reset candidate.

Comparator:

Exact result equality across all implementations.

Smallest discriminating case:

One valid value on each side of an oversized separator detects both stale-state
and skipped-successor faults. Alternation repeats the transition.

Mutation backstop:

Reset only `sum`, reset only `left`, or advance `left` one extra position. An
exact count fails.

Final verdict:

PASS (INFO). The test directly observes separator and reset semantics.

### `maximum_value_separator_after_positive_prefix_resets_state`

Claim and scope:

For eleven copies of `1`, followed by `u64::MAX` and `0`, at budget `11`,
every implementation returns exactly `67`. The maximum value separates the
all-fit positive prefix from the trailing zero.

Plausible violation:

The direct path stops repair before removing the maximum value. The reset path
clears its sum but leaves its left boundary behind. Either fault lets ranges
that cross the separator contaminate the trailing-zero contribution.

Observation:

The test reads the exact `u128` result from the default and all five named
implementations for the literal input and budget.

Oracle and independence:

The eleven-one prefix contributes `11 * 12 / 2 = 66` ranges. Every range that
contains `u64::MAX` exceeds budget `11`. The trailing zero contributes one
range. The independent endpoint equation is therefore `66 + 1 = 67`.

Comparator:

Exact scalar equality. The test does not derive its expected value from a
candidate or checksum.

Smallest discriminating case:

`[1, u64::MAX, 0]` at budget `1` is the semantic core. Eleven ones make the
prefix exactly fill budget `11` and force repeated left-edge removals before
the direct scan can pass the separator. That longer repair is the point of this
regression.

Mutation backstop:

Reset `sum` without setting `left = right + 1`, remove only one left value, or
stop repair when the old prefix sum reaches zero while `u64::MAX` remains. The
literal count differs from `67`.

Final verdict:

PASS (INFO). The test directly observes complete state reset after a long
positive prefix and a maximum-value separator.

### `u128_accumulation_preserves_mathematical_sums_above_u64_max`

Claim and scope:

Two `u64::MAX` values are accumulated without narrowing. At budget
`u64::MAX`, only the two singletons fit. At budget `2 * u64::MAX`, all three
nonempty ranges fit.

Plausible violation:

An intermediate `u64` sum wraps, saturates, or panics. A wide budget is narrowed
before comparison.

Observation:

The test reads exact counts `2` and `3` at the two literal budgets from all
implementations.

Oracle and independence:

The expected mathematical sums and triangular count are literal `u128`
equations.

Comparator:

Exact `u128` equality.

Smallest discriminating case:

Two maximum `u64` values are the smallest array whose total exceeds `u64::MAX`.

Mutation backstop:

Change a running or prefix sum to `u64`, or cast the budget down. The pair's
classification fails or debug arithmetic traps.

Final verdict:

PASS (INFO). The test observes both sides of the wide-sum boundary.

### `exhaustive_lengths_zero_through_six_match_on_frozen_small_domain`

Claim and scope:

Every optimized implementation matches the independent reference for all
109,220 cases in the declared finite domain.

Plausible violation:

A boundary fault appears only after several zeros, repeated repairs, an exact
budget sum, or one specific short input order.

Observation:

The test observes exact output for each candidate in every case and asserts the
final case count is exactly `109_220`.

Oracle and independence:

`recompute_reference` re-sums every range and shares no traversal helper with
the candidates. The base-four enumeration is independent of candidate output.

Comparator:

Exact scalar equality. The assertion includes values and budget on failure.

Smallest discriminating case:

The enumerator includes all smaller counterexamples automatically. Length six
allows several extend-and-repair transitions and zero runs.

Mutation backstop:

Shrink a loop bound, remove one base-four digit, alter a boundary comparison,
or count the wrong suffix width. Either an output assertion or the exact domain
count fails.

Final verdict:

PASS (INFO). The test has a bounded, exact claim and a cardinality guard against
silent domain shrinkage.

### `seeded_differential_cases_match_the_independent_reference`

Claim and scope:

All optimized implementations match the reference on the fixed 640-case stream
with lengths through 24, ordinary values through 32, injected `u64::MAX`, and
the declared budget classes.

Plausible violation:

A defect needs a longer repair sequence, a mixture of ordinary and maximum
values, or a wide budget not reached by the exhaustive domain.

Observation:

Every candidate's exact count is checked for every generated case.

Oracle and independence:

The independent recompute path supplies the expected count. The generator does
not consult any candidate and uses five fixed seeds.

Comparator:

Exact scalar equality. Failures report candidate, seed, case index, full input,
and budget.

Smallest discriminating case:

The campaign is not minimal by construction. Its purpose is broader sampled
coverage. Any failure must become a minimized literal regression.

Mutation backstop:

Narrow sums, mishandle maximum values, or corrupt left state after a long
repair. A fixed generated case is expected to fail, while literal and
exhaustive tests remain the deterministic backstop if it does not.

Final verdict:

PASS (INFO). The claim is limited to a replayable sampled stream and does not
present generation as exhaustive proof.

### `reversing_values_preserves_the_number_of_valid_ranges`

Claim and scope:

Reversing a finite array preserves the number of contiguous ranges with sum at
most the same budget because `[start,end)` maps bijectively to the reversed
range `[n - end,n - start)` with the same members and sum.

Plausible violation:

A candidate depends on left-to-right value order beyond the monotone sum
contract or mishandles a boundary at one end.

Observation:

The reference result for the original input is compared with the reference and
all optimized results for the reversed input.

Oracle and independence:

The bijection proves the relation. Anchoring to the original reference result
prevents a constant transformed output from passing on its own agreement.

Comparator:

Exact `u128` equality for empty, running, mixed, and maximum-value fixtures.

Smallest discriminating case:

The asymmetric mixed fixtures make reversal visibly change scan order while
preserving the endpoint-pair count.

Mutation backstop:

Drop the first or last contribution, retain stale left state, or use an
order-dependent reset boundary. At least one asymmetric fixture fails.

Final verdict:

PASS (INFO). The relation has a clear bijection and an independent anchor.

### `increasing_the_budget_never_decreases_the_count`

Claim and scope:

For a fixed input, increasing the budget across the declared ordered probe list
never decreases any implementation's count.

Plausible violation:

A comparison is reversed, arithmetic narrows at a boundary, or reset behavior
changes non-monotonically.

Observation:

The test retains the previous exact count and compares it with the next result
for each candidate and fixture.

Oracle and independence:

Set inclusion is the oracle: every range valid at budget `b` remains valid at
any larger budget. This relational test does not use a candidate as an expected
absolute count.

Comparator:

Exact nondecreasing order with `observed >= previous`.

Smallest discriminating case:

The probes include zero, exact small boundaries, `u64::MAX`, twice that value,
and `u128::MAX` over empty, mixed, and maximum-value arrays.

Mutation backstop:

Reverse the budget predicate or narrow a wide budget. The observed sequence can
decrease. A constant wrong implementation could pass this relation, so the
differential tests provide the absolute backstop.

Final verdict:

PASS (INFO). The test claims only monotonicity and the suite separately anchors
absolute correctness.

### `scaling_values_and_budget_by_a_positive_factor_preserves_the_count`

Claim and scope:

Multiplying every value and the budget by positive factor nine preserves the
valid endpoint pairs for the fixed mixed fixture.

Plausible violation:

Arithmetic narrows, scaling changes an exact-boundary comparison, or a
candidate depends on raw magnitude rather than the ordered sum predicate.

Observation:

Each implementation's scaled result is compared with the reference result from
the unscaled input.

Oracle and independence:

For positive `c`, `c * sum <= c * budget` is equivalent to
`sum <= budget`. The unscaled recompute result anchors the expected count.

Comparator:

Exact `u128` equality.

Smallest discriminating case:

The fixture contains zeros, exact and inexact partial sums, and separated large
values. Factor nine changes every positive magnitude without overflowing
`u64`.

Mutation backstop:

Use a strict bound, narrow the running sum, or compare an unscaled intermediate
with the scaled budget. The anchored result fails.

Final verdict:

PASS (INFO). The test checks one precise algebraic relation and does not claim
all scale factors.

### `a_budget_covering_the_total_sum_counts_the_triangular_number`

Claim and scope:

When the budget equals the exact total sum of a nonnegative input, every
nonempty contiguous range is valid, so the answer is `n(n + 1) / 2`.

Plausible violation:

The implementation omits suffixes, uses a strict comparison, over-repairs, or
narrows a total above `u64::MAX`.

Observation:

Every implementation's result is compared with the hand-computed triangular
number for empty, one-value, ordinary, and maximum-value fixtures.

Oracle and independence:

The triangular equation counts endpoint pairs directly. The total is computed
as a `u128` fold over the input, independent of candidate traversal.

Comparator:

Exact `u128` equality.

Smallest discriminating case:

The four-value wide fixture includes two `u64::MAX` values, zero, and one. It
checks both the all-fit relation and wide accumulation.

Mutation backstop:

Add one per end, use `< budget`, or accumulate the total or window in `u64`.
The triangular result fails.

Final verdict:

PASS (INFO). The oracle is an independent endpoint-pair equation and covers a
wide-sum case.
