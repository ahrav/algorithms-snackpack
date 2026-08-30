# Prefix-scan test strategy

Scope:
The public `i64` inclusive and seeded-exclusive APIs, wrapping arithmetic,
ordered output, empty and singleton boundaries, invalid block and worker
configuration, partial blocks, and multiworker output composition.

Contract and failure classes:
Inclusive output at index `i` is the ordered wrapping sum through `input[i]`.
Seeded-exclusive output at index `i` is `init` followed by the ordered wrapping
sum of inputs before `i`. Output length equals input length. Empty input returns
empty output. `blocked_inclusive(input, 0)` returns
`ScanError::ZeroBlockSize`. `parallel_inclusive(input, 0)` returns
`ScanError::ZeroWorkers`, including for empty input. Both rejection paths leave
the borrowed input unchanged. Failure classes include wrong inclusion boundary,
ignored seed, ordinary or checked addition, reordered operands, missing or
double-counted chunk offsets, mishandled partial chunks, zero-parameter fallback,
and worker-count partition errors.

Primary technique:
Differential agreement against deliberately quadratic, independently written
reference functions. The bounded exhaustive domain enumerates every vector of
length `0..=7` over `{-2, -1, 0, 1, 2}` for the scalar candidates and every
valid blocked size through `len + 1`.

Secondary techniques:
Exact examples, empty and singleton boundaries, explicit overflow vectors,
direct recurrence checks, rejection checks, a partial-final-block regression,
and deterministic multiworker output cases. Source review, not a test
assertion, establishes the current thread topology.

Oracle:
`reference_inclusive` and `reference_exclusive` restart from the beginning of
each prefix. They share no correctness-critical helper with the optimized
candidates. Exact expected vectors provide a second oracle for the running
example, overflow, partial-block, and seeded-exclusive regressions. The direct
inclusive recurrence is a third, local semantic observation.

Input/model/schedule domain and bounds:
The exhaustive value model contains 97,656 vectors and covers lengths `0..=7`.
It does not enumerate the full `i64` domain. Overflow tests cover both signed
extremes. The multiworker tests use deterministic inputs, worker counts `3`,
`4`, and `64`, and effective worker counts both below and equal to input length.
The operating system chooses actual schedules. No scheduler interleaving is
enumerated or controlled. The tests do not observe thread count, phase count,
or whether a candidate delegates to a semantically equivalent scalar path.

Replay assets:
All cases are deterministic source-level fixtures in `tests/scan.rs`. The
exhaustive generator has no random seed. A failing assertion prints the input
and configuration needed to replay model disagreement.

What a green run establishes:
A green run establishes exact output agreement on the finite domains above and
exercises the configured rejection paths. It is bounded evidence. Source review
separately establishes that the current multiworker implementation caps the
effective worker count and uses two joined scoped-thread phases. The tests do
not prove those implementation-shape facts. A green run is not a proof over all
input lengths, all `i64` vectors, allocation failures, panicking workers, or all
operating-system schedules.

Known gaps and delegated skills:
No unsafe code exists, so Miri is not selected. Threads operate on owned chunk
vectors, join before phase transitions, and expose no caller-visible concurrent
state, so Loom and Shuttle would model a different runtime contract and are not
selected. The finite arithmetic contract does not justify Kani for this first
artifact because exhaustive small vectors, exact overflow cases, and an
independent model target the identified failures directly. Fuzzing is not
selected because the input is already typed and the main boundary is algebraic,
not a parser. `invariant-test-review` is applied below to every claim-bearing
test.

Concrete tests to add:
None before publication. The selected test set is implemented in
`tests/scan.rs`. Any minimized future failure must become a named deterministic
regression here before broadening a generator.

## Invariant-test review

### `exact_example_matches_inclusive_and_exclusive_contracts`

Claim and scope:
All candidates implement inclusive or seeded-exclusive semantics on one
nonuniform example, including a partial block and multiple worker chunks.

Plausible violation:
Exclusive output includes the current element, or a blocked or parallel
candidate omits the offset from an earlier chunk.

Observation:
The test compares every output element and the full output length.

Oracle and independence:
Literal expected vectors are independent of every implementation. Reference
functions are also checked against those literals.

Comparator:
Exact `Vec<i64>` equality preserves length, order, multiplicity, and bit
patterns.

Smallest discriminating case:
The first element separates inclusive from exclusive. The third chunk separates
one preceding offset from the sum of all preceding offsets.

Mutation backstop:
Moving the exclusive write after the addition or applying only the immediately
preceding chunk total changes a literal position and fails this test.

### `empty_and_singleton_boundaries_match_the_models`

Claim and scope:
Every valid candidate returns empty for empty input and handles one element
without an out-of-bounds read or spurious addition.

Plausible violation:
The candidate emits an identity for empty input, drops a singleton, or adds the
singleton into seeded-exclusive output.

Observation:
The test observes full returned vectors for empty and singleton inputs.

Oracle and independence:
The mathematical boundary values are literal. No optimized candidate supplies
an expected value.

Comparator:
Exact emptiness and exact vector equality.

Smallest discriminating case:
Lengths zero and one are the smallest cases that reach these boundaries.

Mutation backstop:
Unconditionally pushing an accumulator or omitting the first inclusive output
fails one of the literal checks.

### `every_candidate_uses_wrapping_add_at_overflow_boundaries`

Claim and scope:
Every candidate uses ordered `i64::wrapping_add` when a prefix crosses both
signed boundaries.

Plausible violation:
Ordinary debug addition panics, checked addition rejects, saturating addition
sticks at an endpoint, or a reassociation changes an intermediate result for a
different operator.

Observation:
The test observes the complete inclusive and exclusive bit-pattern sequences.

Oracle and independence:
The expected vectors are calculated literally from the wrapping contract and
checked against both reference and optimized implementations.

Comparator:
Exact signed values, which are exact `i64` bit-pattern comparisons.

Smallest discriminating case:
`[i64::MAX, 1]` distinguishes wrapping from checked, ordinary-debug, and
saturating addition. The longer fixture crosses both boundaries and returns.

Mutation backstop:
Replacing any hot-path `wrapping_add` with `saturating_add` changes the second
prefix. Replacing it with `+` fails under the test profile when overflow occurs.

### `exclusive_nonzero_init_is_included_and_wraps`

Claim and scope:
Both exclusive candidates place the exact seed at output zero and carry that
seed through later wrapping prefixes.

Plausible violation:
The implementation silently uses zero, treats `init` as an input identity, or
adds the current element before writing.

Observation:
The complete seeded output vector is compared.

Oracle and independence:
A literal vector is the primary oracle; the quadratic reference is checked
against the same vector.

Comparator:
Exact vector equality.

Smallest discriminating case:
A nonzero seed and one input distinguish seeded exclusive output from the
zero-seeded and inclusive variants. The chosen seed also forces wrapping.

Mutation backstop:
Initializing the optimized accumulator to zero or writing after addition fails
the first or second expected position.

### `zero_configuration_is_rejected_without_changing_input`

Claim and scope:
Both configurable APIs return their exact error for zero configuration, for
nonempty and empty input, while the borrowed input value remains unchanged.

Plausible violation:
Zero silently selects a fallback, returns the wrong error, divides by zero, or
checks empty input before rejecting zero workers.

Observation:
The test observes the exact `Result` and compares the nonempty input with a
pre-call clone after each rejection.

Oracle and independence:
The expected enum variants are literal contract values. Safe immutable borrowing
makes input mutation structurally unavailable to this implementation.

Comparator:
Exact `Result` equality and exact `Vec<i64>` equality.

Smallest discriminating case:
An empty input with zero workers distinguishes validation order. A one-element
input would be sufficient for input preservation; the fixture uses three
different values.

Mutation backstop:
Changing either early return to `Ok` or swapping error variants fails directly.
Checking empty input before worker validation fails the empty-input assertion.

### `partial_final_block_receives_the_complete_preceding_offset`

Claim and scope:
A short final block receives the reduction of every earlier block total.

Plausible violation:
The final block receives no carry, only the immediately preceding block total,
or a carry that includes its own total.

Observation:
The test compares every output in a seven-element, three-block result.

Oracle and independence:
The expected arithmetic progression is literal and does not call the reference
or optimized code.

Comparator:
Exact vector equality.

Smallest discriminating case:
Three blocks are required to distinguish the preceding block from all preceding
blocks. A short third block also exercises `n % block_size != 0`.

Mutation backstop:
Resetting `carry` to each block total or skipping the carry update changes the
first value of the third block.

### `exhaustive_model_agreement_for_lengths_zero_through_seven`

Claim and scope:
Scalar inclusive, scalar seeded-exclusive with seed zero, one-worker parallel,
and blocked inclusive agree with independent models throughout the stated
finite domain and blocked-size range.

Plausible violation:
Any candidate has a value-dependent error, an off-by-one prefix boundary, a
block-size boundary error, or a wrong empty/singleton path within the domain.

Observation:
Every generated case compares the full candidate output immediately and prints
the input and configuration on failure.

Oracle and independence:
Quadratic reference functions restart each prefix and share no helper with the
optimized loops. The generator only supplies inputs.

Comparator:
Exact vector equality without sorting, normalization, or ignored fields.

Smallest discriminating case:
The enumeration contains each shorter counterexample before longer vectors.
The block-size loop includes one, exact length, and greater than length.

Mutation backstop:
Dropping the final input, using the current element in exclusive output, or
starting a later block at zero produces a generated counterexample. The test
does not claim exhaustive multiworker schedule coverage; that path has a
separate test.

### `parallel_multi_chunk_output_matches_model`

Claim and scope:
The public multiworker configuration returns the exact ordered prefix output on
an uneven 257-element input.

Plausible violation:
Chunks are reassembled out of order, offset repair starts at the wrong chunk,
or a later chunk omits an earlier total.

Observation:
The test compares the full 257-element returned output. It does not observe
thread creation, joins, phase count, or whether the implementation delegates to
a scalar path.

Oracle and independence:
The quadratic reference runs separately and shares no worker, partition, or
offset helper.

Comparator:
Exact vector equality in original order.

Smallest discriminating case:
Three chunks can distinguish one preceding total from all preceding totals.
The larger fixture also makes all four chunks nontrivial and the division
uneven.

Mutation backstop:
Reversing repaired chunks, skipping one carry update, or applying the first
chunk's zero offset to all chunks changes the reference comparison. Replacing
the implementation with a correct scalar scan survives, so source review owns
the thread-topology claim.

### `inclusive_recurrence_holds_at_every_output_boundary`

Claim and scope:
Each optimized inclusive output begins with `input[0]` and each later output is
the preceding output plus the next input under wrapping arithmetic.

Plausible violation:
A candidate can match an aggregate checksum while one interior boundary is
wrong, duplicated, or shifted.

Observation:
The test checks the first value and every adjacent output/input transition.

Oracle and independence:
The recurrence is the public contract itself and does not call the quadratic
model. This observation is independent of the differential tests.

Comparator:
Exact value equality at each boundary.

Smallest discriminating case:
Two inputs expose a wrong recurrence. The fixture crosses signed boundaries and
multiple chunk boundaries.

Mutation backstop:
Skipping, duplicating, or shifting one input makes the corresponding adjacent
assertion fail even if a final total happens to match.

### `workers_greater_than_input_length_preserve_results`

Claim and scope:
Requesting more workers than input elements still returns exact ordered
prefixes.

Plausible violation:
Partitioning creates empty chunks, divides incorrectly, drops values, or starts
more logical chunks than output positions.

Observation:
The full returned result is compared. The test does not observe the number of
threads or chunks.

Oracle and independence:
The quadratic reference does not use worker counts or chunk partitioning.

Comparator:
Exact vector equality.

Smallest discriminating case:
Three inputs with 64 requested workers establish the public output behavior for
`workers > len`.

Mutation backstop:
Any partition implementation that drops, duplicates, or reorders an input fails
the reference comparison. A correct implementation that creates extra empty
workers survives, so source review, not this test, establishes
`min(workers, input.len())`.

## Review verdict

Each named claim reaches its intended path and observes the promised state.
The oracles are literal expectations, an independently structured quadratic
model, or the direct recurrence. No comparator canonicalizes away order,
length, multiplicity, or bit-pattern differences. No unresolved invariant-test
blocker remains in this recorded design. The completed test run is still
bounded evidence over the domains named above.
