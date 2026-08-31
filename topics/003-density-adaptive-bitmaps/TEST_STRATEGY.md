# Test strategy

Scope:

The immutable `ReferenceSet`, `SortedSet`, `DenseBitSet`, and
`AdaptiveBitmap` APIs: construction, duplicate normalization, universe
validation, exact errors, membership, cardinality, ascending iteration,
same-type intersection cardinality, logical payload accounting, and adaptive
container selection. No mutation, serialization, unsafe code, threads, clocks,
files, or external services enter the public contract.

Contract and failure classes:

Each successful value denotes a finite set of unique `u32` members in
`[0, universe_exclusive)`, where `universe_exclusive <= 2^32`. Input order and
duplicates do not change that set. Iteration is strictly increasing and
complete. Membership is false outside the declared universe. Same-type
intersection returns the exact shared-member count and rejects unequal
universes with the exact `UniverseMismatch` fields. Construction rejects an
oversized universe and the first out-of-universe input with the exact error
fields. Dense allocation failure reports the required logical bytes.

Adaptive containers use `Array` through cardinality 4,096 and `Bitmap` above
it, then replace that baseline with `Run` only when `2 + 4r` is strictly
smaller than the baseline payload. Target failures include deduplicating only
adjacent inputs, unsorted iteration, a bit shift at word or chunk boundaries,
counting dense padding bits, an array/bitmap off-by-one, choosing a run on a
payload tie, dropping or merging a run endpoint, skipping an occupied high
key, comparing unequal universes, asymmetric intersection, and circular model
agreement.

Primary technique:

Differential comparison with `ReferenceSet`. The reference model uses a simple
ordered-set representation and shares no dense-word, sorted-merge, high-key,
container-selection, run-building, or intersection kernel with the three
optimized candidates. Every comparison observes exact iteration, length,
membership, or intersection cardinality rather than a checksum.

Secondary techniques:

Hand-derived examples; exact error and boundary cases; direct inspection of
`ContainerSummary`; exhaustive pairs of small finite sets; a deterministic
generated immutable campaign; and anchored intersection algebra properties.
Payload equations are checked at cases that discriminate strict inequality
from ties. Byte fuzzing, Miri, Loom, Shuttle, async-time control, and
state-machine mutation testing do not target this immutable, typed, safe API.

Oracle:

Literal sorted member vectors, exact counts, and exact `BitmapError` values are
the primary oracle for boundaries and the running example. `ReferenceSet` is
the independent model for broader cases. Exhaustive masks independently define
membership over their finite universe, so agreement cannot pass merely because
all implementations share one wrong iterator. Adaptive representation claims
use hand-derived cardinality, run count, and payload equations, not an
optimized candidate's own choice as the expected value.

Input/model/schedule domain and bounds:

- Literal domains include universes `0`, `4`, `10`, `16`, `65_536`, `65_537`,
  `131_073`, and the sparse/adaptive `2^32` boundary; word boundaries `63/64`;
  chunk boundaries `65_535/65_536`; a run ending at low value `65_535`; and
  adaptive cardinalities `4_095/4_096/4_097`.
- Exhaustive domain: every ordered pair of subsets for every universe size
  `0..=8`, exactly `sum(4^u, u=0..8) = 87_381` pairs. Membership is checked at
  every in-universe value and two out-of-universe probes.
- Generated domain: exactly 10,000 immutable pairs from seed `2026083103`,
  universes drawn from the fixed list through `131_072`, shuffled inputs with
  injected duplicates, and up to 97 generated input records per side. Coverage
  anchors require every declared universe, empty input, duplicates, and values
  in a nonzero high-16 chunk. Threshold and terminal-run boundaries remain
  literal tests rather than sampled claims.
- Schedule domain: none. All operations are sequential and immutable.

The generated campaign is sampled evidence. It does not enumerate every
`u32` set. The full `2^32` semantic boundary is exercised only by
`ReferenceSet`, `SortedSet`, and `AdaptiveBitmap`; the suite deliberately does
not attempt a 512 MiB `DenseBitSet` payload.

Replay assets:

All literal cases, exhaustive loops, generator code, seed `2026083103`, case
index, and full failing inputs live in `tests/bitmap_sets.rs`. Generator
coverage anchors and the exact 87,381-pair count fail if the intended finite
domain silently shrinks. Any generated failure must be minimized into a named
literal regression before its fix is accepted.

What a green run establishes:

A green run establishes exact behavior for every literal and exhaustive case,
and agreement with the independent model for the fixed 10,000-case stream. It
also establishes the inspected adaptive choices and logical payloads at the
declared boundary cases. This is bounded evidence. It is not a proof for all
possible sets, allocation failures, machines, serialized inputs, mutation
histories, or performance claims.

Known gaps and delegated skills:

`DenseAllocation` cannot be forced portably without allocator injection, so
the suite checks its arithmetic indirectly but does not claim deterministic
coverage of an operating-system allocation failure. Universe `2^32` is checked
on representations that do not require a 512 MiB fixed array; dense success at
that size is machine-budget dependent. The generated campaign samples rather
than exhausts large universes. Resident memory, allocation count, construction
cost, and operation crossovers belong to `BENCHMARK.md`. A future serializer
would require raw-byte fuzzing and checked-decoder tests. A future mutable API
would require transition-by-transition model comparison and threshold-churn
sequences.

Concrete tests to add:

- `running_example_matches_all_candidates`
- `zero_universe_and_out_of_universe_contract`
- `iteration_is_sorted_unique_and_duplicates_are_idempotent`
- `word_and_chunk_boundaries_match_reference`
- `adaptive_thresholds_and_strict_run_selection_are_exact`
- `intersection_rejects_universe_mismatch`
- `intersection_len_matches_independent_reference`
- `exhaustive_small_domain_matches_reference`
- `seeded_immutable_cases_match_reference`
- `intersection_laws_hold_for_all_candidates`

## Invariant-test audit

### `running_example_matches_all_candidates`

Claim and scope:

All four candidates construct the documented universe-16 `A` and `B`, iterate
their exact members, and return intersection `{2, 3, 10, 14}` with cardinality
four.

Plausible violation:

A sorted cursor skips a match, a dense `AND` uses the wrong bit, duplicate
normalization changes a member, or one candidate reports the wrong count.

Observation:

Exact lengths, sorted iterators, literal intersection members derived through
the reference view, and each same-type `intersection_len` result.

Oracle and independence:

Both operand vectors and `[2, 3, 10, 14]` are hand-derived literals.

Comparator:

Exact `Vec<u32>` and `Result<usize, BitmapError>` equality. No sorting or
deduplication is applied to candidate output.

Smallest discriminating case:

The shared values include adjacent `2,3` and separated `10,14`; nonmembers lie
between them, so neither a range approximation nor one aggregate bit can pass.

Mutation backstop:

Advance both sorted cursors on inequality, use `OR` in the dense kernel, or
skip one match. The literal member vector or count must fail.

### `zero_universe_and_out_of_universe_contract`

Claim and scope:

Every candidate accepts `(0, &[])`, reports empty behavior, returns false for
membership, rejects universe greater than `2^32`, and reports the first invalid
input's index, value, and universe exactly. The three sparse/adaptive
representations also accept members `0` and `u32::MAX` at universe `2^32`.

Plausible violation:

Zero universe allocates or indexes a word, `value == U` is accepted, validation
reports the last invalid value, or conversion truncates an oversized `u64`
universe.

Observation:

The complete construction result, empty iterator, length, membership result,
and exact `BitmapError` fields. Full-universe success excludes `DenseBitSet`.

Oracle and independence:

Expected errors are literal enum values derived from the half-open-universe
contract. They do not call a candidate validator.

Comparator:

Exact `Result` and enum-field equality; `is_err()` alone is forbidden.

Smallest discriminating case:

`try_new(0, &[])` is the smallest valid universe. `try_new(4, &[3, 4, 9])`
places the first invalid value at input index 1. `2^32 + 1` distinguishes
checked universe validation from truncation.

Mutation backstop:

Change `>= U` to `> U`, continue validation after the first failure, cast `U`
to `u32` before checking, or special-case empty input before universe
validation. An exact assertion must fail.

### `iteration_is_sorted_unique_and_duplicates_are_idempotent`

Claim and scope:

For shuffled repeated input, every candidate denotes exactly
`{0, 1, 4, 7, 9}`, has length five, and iterates `[0, 1, 4, 7, 9]` once each.

Plausible violation:

Only adjacent duplicates are removed, insertion order leaks into iteration, or
length counts input records rather than members.

Observation:

Exact length, every relevant membership result, and the complete iterator.

Oracle and independence:

`[0, 1, 4, 7, 9]` is a literal expected vector derived without any candidate
helper.

Comparator:

Exact iterator sequence equality. Converting candidate output to a set would
erase both duplicate and ordering defects and is forbidden.

Smallest discriminating case:

`[9, 1, 9, 4, 1, 7, 4, 0, 9]` contains nonadjacent duplicates and a
non-sorted first-occurrence order.

Mutation backstop:

Remove the sort, remove deduplication, or set `len` from input length. The
sequence or cardinality assertion must fail.

### `word_and_chunk_boundaries_match_reference`

Claim and scope:

Values around `63/64` and `65_535/65_536` retain exact membership and ascending
iteration in every candidate. The adaptive directory omits an empty high key
between occupied keys zero and two.

Plausible violation:

A word index uses 63-bit groups, a shift uses the global value, the low 16 bits
wrap without incrementing the key, or the adaptive directory emits an empty
middle container.

Observation:

Exact iteration, membership hits and misses at every neighboring boundary,
logical dense payload bytes for universe 131,073, and the exact adaptive key
sequence for contiguous and sparse directories.

Oracle and independence:

The boundary member vector is literal and `ReferenceSet` has no dense-word or
high/low split.

Comparator:

Exact sequence, Boolean membership, and cardinality equality.

Smallest discriminating case:

`63` and `64` cross one word; `65_535` and `65_536` cross one adaptive key.
Universe `131_073` makes the final dense word partial and creates high key two.

Mutation backstop:

Use `/ 63`, shift by the unmasked value, omit the high-key carry, or materialize
an empty high-key container. One neighboring assertion must fail.

### `adaptive_thresholds_and_strict_run_selection_are_exact`

Claim and scope:

Isolated cardinalities 4,095 and 4,096 select `Array`; 4,097 selects `Bitmap`.
One run of three values ties its six-byte array baseline and remains `Array`;
one run of four values selects six-byte `Run` over an eight-byte array. A
four-value run ending at low value 65,535 preserves that terminal endpoint.

Plausible violation:

The threshold is `< 4096`, the 4,097th value is lost during conversion, a run
tie incorrectly wins, run size omits its two-byte run-count field, or adding
one past a terminal `u16` run endpoint overflows.

Observation:

Exact `ContainerKind`, cardinality, run count, payload bytes, iterator, and
membership from `container_summaries()` and the set API.

Oracle and independence:

Expected choices come from the literal rules `2c`, `8192`, and `2 + 4r` with a
strict comparison. Isolated even values make every value a separate run and
prevent accidental run selection.

Comparator:

Exact public summary fields and exact members. Semantic agreement alone is too
weak because it cannot detect the claimed representation boundary.

Smallest discriminating case:

Cardinalities 4,096 and 4,097 separate the baseline containers. Three versus
four consecutive values separate a payload tie from a strict run win. The
terminal `65_532..=65_535` run is the smallest selected run in this test that
reaches the largest low value.

Mutation backstop:

Change either threshold comparison, use `<=` for run selection, omit the run
header, increment a `u16` endpoint before widening, or drop a value during
array-to-bitmap construction. A summary or member assertion must fail.

### `intersection_rejects_universe_mismatch`

Claim and scope:

Every same-type intersection rejects left universe 8 and right universe 9 with
the exact ordered `UniverseMismatch` fields.

Plausible violation:

Intersection silently uses the shorter universe, compares only payload shape,
or swaps error fields.

Observation:

The complete left-to-right `Result` for every candidate type.

Oracle and independence:

The expected enum values are literal. No intersection implementation supplies
the expected universes.

Comparator:

Exact error variant and fields in both directions.

Smallest discriminating case:

Operands with shared members but universes 8 and 9 distinguish semantic
membership agreement from the required universe check.

Mutation backstop:

Compare only members or payload sizes, compare dense word counts instead of
universes, or normalize field order. An exact assertion fails.

### `intersection_len_matches_independent_reference`

Claim and scope:

For fixed array-, bitmap-, and run-container pairs, plus disjoint, equal,
multi-chunk, and unmatched-key cases, every optimized candidate returns the
same exact intersection count as `ReferenceSet`.

Plausible violation:

A two-pointer merge advances the wrong side, a dense kernel counts an operand
instead of `AND`, a run endpoint is inclusive twice, or an unmatched adaptive
key is treated as present.

Observation:

Each complete `Result<usize, BitmapError>` and the exact operand iterators used
to diagnose disagreement.

Oracle and independence:

`ReferenceSet` traverses its independent ordered model and shares no optimized
intersection helper. Literal counts anchor at least one case of every declared
shape.

Comparator:

Exact `usize` equality, plus exact operand member equality before intersection.
A hash or approximate count is forbidden.

Smallest discriminating case:

`{1, 64, 65_536, 131_072}` intersected with
`{0, 64, 65_536, 65_537}` has two matches across word and chunk boundaries,
with high key two present only on the left.

Mutation backstop:

Advance both sorted cursors on inequality, use `OR` in the dense kernel,
extend a run end by one, or emit a count for an unmatched key. A literal or
reference count fails.

### `exhaustive_small_domain_matches_reference`

Claim and scope:

For exactly 87,381 subset pairs across universes `0..=8`, all optimized
candidates match the independent model for members, length, every membership
probe, and intersection cardinality.

Plausible violation:

A defect depends on empty input, one particular low bit, disjointness, equality,
subset direction, or the last valid universe value.

Observation:

Every candidate is compared immediately for every pair. Independent counters
check universe count, subset count, pair count, and probe coverage.

Oracle and independence:

The two subset masks directly define expected membership and intersection via
bit arithmetic. `ReferenceSet` is checked against those masks before serving
as the broader candidate oracle.

Comparator:

Exact member vector, length, Boolean membership, and intersection count. The
test performs no canonicalization of candidate output.

Smallest discriminating case:

Universes grow in order, so empty, singleton, disjoint, equal, and partial
overlap counterexamples occur before larger supersets.

Mutation backstop:

Skip universe zero, omit the highest subset mask, generate one operand from the
other, duplicate a pair while preserving a loop count, or corrupt one bit
kernel. Domain cardinality, uniqueness anchors, or semantic comparison fails.

### `seeded_immutable_cases_match_reference`

Claim and scope:

For exactly 10,000 cases from seed `2026083103`, every optimized candidate
matches the reference on construction result, exact members, membership probes,
and intersection count, with the declared large-shape coverage anchors.

Plausible violation:

A failure requires shuffled duplicates, a nonzero high-16 key, unequal small
operand sizes, or a larger partial final dense word absent from the exhaustive
domain. Cardinality 4,096, long runs, and fragmented runs are covered by named
literal tests because this generator emits at most 97 input records per side.

Observation:

The full result for each candidate plus the fixed seed and case index.
Independent flags and sets verify every declared coverage anchor. The exact
input vectors are reconstructed from that seed and case index.

Oracle and independence:

`ReferenceSet` is constructed separately. The LCG, shape generator, and
coverage accounting call no optimized candidate helper.

Comparator:

Exact `Result`, iterator, membership, and cardinality equality. Failure output
contains the complete replay case.

Smallest discriminating case:

The generator is one deterministic pseudorandom stream, so the first failing
case index is the smallest replay point in that stream. Exact threshold and
terminal-run boundaries stay in named literal tests. Any sampled failure must
be minimized into a new literal test.

Mutation backstop:

Collapse the generator to one universe, suppress duplicates, remove the
nonzero high-key universe, or bypass one candidate. Coverage anchors, case
count, candidate count, or model agreement must fail. A green sampled run
remains bounded evidence.

### `intersection_laws_hold_for_all_candidates`

Claim and scope:

For fixed same-universe operands, intersection cardinality is
commutative, idempotent, bounded by the smaller cardinality, zero against the
empty set, and equal to cardinality for equal sets in every candidate type.

Plausible violation:

Intersection advances asymmetrically, double-counts duplicates, includes
padding, substitutes union for intersection, or mishandles the empty operand.

Observation:

Both operand orders, self-intersections, empty intersections, and operand
lengths.

Oracle and independence:

The laws are direct consequences of finite-set intersection. The helper takes
only public lengths and intersection results; it does not use a candidate's
internal output as the expected value.

Comparator:

Exact equality and integer inequalities. This test claims only laws observable
through `intersection_len`; it makes no materialized-intersection or union
claim.

Smallest discriminating case:

The executable operands share `{2, 3, 64, 90}`, differ on both sides, include a
duplicate input, and cross a word boundary. A separate empty operand anchors
the zero boundary.

Mutation backstop:

Advance only one cursor on equality, count input records rather than members,
use `OR`, or return a nonzero count against empty. At least one directional,
bound, or literal assertion fails.

## Review verdict

Every planned claim-bearing test has a reachable plausible fault, a direct
observation, an independent or literal oracle, a semantic comparator that
preserves the contract, a discriminating case, and a stated mutation backstop.
No test claims mutable behavior, serialized compatibility, allocation-failure
coverage, or performance. A completed green run will remain bounded evidence
over the domains recorded above.
