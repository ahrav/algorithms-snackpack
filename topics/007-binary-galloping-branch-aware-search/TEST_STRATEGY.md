# Test strategy

Scope: immutable, sorted `u64` lower bounds. Failure classes: skipped first
duplicate, wrong insertion/end sentinel, empty-input indexing, bracket growth
or half-open boundary mistakes, size-invariant final off-by-one.
Primary technique: exact boundary cases and exhaustive finite-domain checks.
Secondary technique: seeded irregular/duplicate arrays and partition checks.
Oracle: independent full-slice count of values below the query, never a
candidate call. Comparator: exact returned index and both partition sides.
Domain: all 1001 nondecreasing arrays of length 0..10 over 0..3, six keys
each; 256 seeded arrays of lengths case*17; power-of-two neighbors through
1025 and all gap/end queries; integer extrema and the running example.
Replay: committed tests, fixed seeds and exact runner. A green run establishes
agreement in these bounds, not proof for all lengths or hardware. Unsorted
input is outside the contract; no unsafe or concurrent implementation exists.
Concrete tests to add: none required for the implemented contract.

# Invariant-test audit

- `empty_singletons_duplicates_and_extrema`: premature indexing or returning
  an equal-run member is rejected by zero-length and all-equal cases. Smallest
  cases: `[]`, `[4,4]` with key 4. Oracle count stays independent.
- `running_example_first_duplicate`: equality discarding the left half would
  return a later 4. Exact index and both partition sides reject it.
- `exhaustive_sorted_arrays`: missing final comparison or wrong midpoint
  update is visible on singleton/two-value cases; expected counts come from
  the full oracle, with no candidate helper.
- `power_of_two_brackets_and_tails`: truncated gallop bracket, off-by-one
  `hi`, or probing beyond len fails lengths 2,3,7,8,9 and above-range keys.
- `seeded_irregular_and_duplicate_runs`: assumes arithmetic spacing or
  unique values would disagree on fixed-seed irregular sorted values.

Mutation backstop: plausible violations above are discriminated by exact
index plus all-before/all-after assertions. No automated mutation score is
claimed. Saturation avoids arithmetic wrap; allocating near-usize::MAX
slices is not tested. Public API docs include a checked duplicate/end example.
