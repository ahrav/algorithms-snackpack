# Two pointers and sliding windows

## What this lesson is trying to teach

Two pointers turn some nested searches into one monotone scan. A sliding window
is the same idea applied to a contiguous range: extend one boundary, repair the
other boundary, then use the repaired window.

The speedup is not automatic. It depends on a monotone fact that makes discarded
positions stay discarded. For this crate, every input value is nonnegative. If
a window sum is too large, extending the window cannot make it smaller. Moving
the left boundary right cannot make it larger. That is the fact that supports
the linear scan.

This lesson counts bounded-sum subarrays. It also shows a sorted pair search, a
fixed-width window, a prefix-sum alternative, and the cases where the sliding
argument stops being valid.

## If you remember only these things

- State the window as a half-open range `[left, end)`. Its length is
  `end - left`.
- Name the monotone fact before using two pointers. Here it comes from
  nonnegative values.
- After repairing the window ending at `end`, every suffix that starts at or
  after `left` is valid. That contributes `end - left` subarrays at once.
- An element enters the direct window once and leaves at most once. The scan is
  `O(n)` and uses no heap allocation.
- Zeros are valid and important. They create equal prefix sums and may require
  several left moves before the sum changes.
- Use wide arithmetic for both the sum and the answer. This crate accepts
  `u64` values and a `u128` budget, and returns an exact `u128` count.
- A green correctness suite does not pick the fastest implementation. The
  benchmark uses a frozen workload family and reports unresolved results when
  the interval crosses the practical boundary.

## Contract

The main API is:

```rust
pub fn count_bounded_subarrays(values: &[u64], budget: u128) -> u128
```

It returns the number of nonempty contiguous half-open ranges `[start, end)`
such that:

```text
0 <= start < end <= values.len()
sum(values[start..end]) <= budget
```

Each endpoint pair identifies one subarray. Equal values at different positions
still produce distinct subarrays. Input order is part of the value being
counted. The function does not sort, deduplicate, or mutate the input.

`u64` input makes nonnegativity a type-level precondition. Sums are accumulated
in `u128`, so summing any slice addressable by a target whose pointer width is
at most 64 bits cannot overflow. The largest answer is `n(n + 1) / 2`. That also
fits in `u128` for every such `usize` value. The empty slice returns zero. A
budget of zero counts every nonempty all-zero subarray and excludes any range
that contains a positive value.

The crate also exposes five named implementations for comparison:

```rust
pub fn recompute_reference(values: &[u64], budget: u128) -> u128;
pub fn quadratic_early_exit(values: &[u64], budget: u128) -> u128;
pub fn direct_sliding(values: &[u64], budget: u128) -> u128;
pub fn oversized_reset_sliding(values: &[u64], budget: u128) -> u128;
pub fn prefix_binary_search(values: &[u64], budget: u128) -> u128;
```

All five have the same `(&[u64], u128) -> u128` contract. The public
`count_bounded_subarrays` function selects `direct_sliding`.

## Three forms of the pattern

A sorted pair search has one pointer at each end. Suppose the sorted input is
`[0, 1, 2, 2, 3]` and the target sum is `4`.

# Visual 1 of 5: sorted pair search

| Step | Left index | Right index | Left value | Right value | Sum | Decision |
|---:|---:|---:|---:|---:|---:|---|
| 1 | 0 | 4 | 0 | 3 | 3 | Sum is small, so advance left |
| 2 | 1 | 4 | 1 | 3 | 4 | Return indices `(1, 4)` |

Advancing the left pointer cannot decrease the sum, and retreating the right
pointer cannot increase it. Sorting supplies the monotone order. This is a
different contract from the subarray problem because sorting an array would
destroy contiguity and original positions.

A fixed-width window keeps its length constant. For `values = [2, 0, 3, 2, 1]`
and width `3`, subtract the value that leaves and add the value that enters.

# Visual 2 of 5: fixed-width rolling sum

| Window | Members | Previous sum | Update | New sum |
|---|---|---:|---|---:|
| `[0, 3)` | `[2, 0, 3]` | none | `2 + 0 + 3` | 5 |
| `[1, 4)` | `[0, 3, 2]` | 5 | subtract `2`, add `2` | 5 |
| `[2, 5)` | `[3, 2, 1]` | 5 | subtract `0`, add `1` | 6 |

The width determines both boundaries. No repair loop is needed.

A variable-width window moves the right boundary once per input value and
moves the left boundary only when the constraint fails. The next section uses
this form.

## Running example

Use:

```text
values = [2, 0, 3, 2, 1]
budget = 5
```

The table uses an exclusive `end`, so adding `values[end - 1]` creates the
candidate window. The repair loop removes values from the left while the sum is
greater than `5`.

# Visual 3 of 5: direct variable-width scan

| End | Added value | Sum after add | Repair actions | Left after repair | Valid starts | Contribution | Running total |
|---:|---:|---:|---|---:|---|---:|---:|
| 1 | 2 | 2 | none | 0 | `0` | 1 | 1 |
| 2 | 0 | 2 | none | 0 | `0, 1` | 2 | 3 |
| 3 | 3 | 5 | none | 0 | `0, 1, 2` | 3 | 6 |
| 4 | 2 | 7 | remove `values[0] = 2` | 1 | `1, 2, 3` | 3 | 9 |
| 5 | 1 | 6 | remove `values[1] = 0`, then `values[2] = 3` | 3 | `3, 4` | 2 | 11 |

The answer is `1 + 2 + 3 + 3 + 2 = 11`.

The last row is a useful zero test. Removing `values[1]` moves the pointer but
does not change the sum. The repair loop must continue. Code that assumes every
left move decreases the sum will retain an invalid window.

For one repaired end boundary, every start in `left..end` is valid. Removing a
nonnegative prefix from a valid window cannot increase its sum. Starts before
`left` are invalid by the minimal-left invariant. The valid windows ending at
that boundary therefore contribute exactly:

```text
end - left
```

Summing that contribution over all end boundaries counts every nonempty valid
subarray once.

## The same example through prefix sums

Let `P[0] = 0` and `P[i + 1] = P[i] + values[i]`. Then the sum of `[start,end)`
is `P[end] - P[start]`. A window is valid exactly when:

```text
P[start] >= P[end] - budget
```

Because values are nonnegative, `P` is nondecreasing. For each `end`, binary
search finds the first earlier prefix that is at least the threshold.

# Visual 4 of 5: prefix boundaries for the running example

| Boundary index | Prefix sum | At `end = 5`, relation to threshold `8 - 5 = 3` | Start status |
|---:|---:|---|---|
| 0 | 0 | below 3 | invalid |
| 1 | 2 | below 3 | invalid |
| 2 | 2 | below 3 | invalid |
| 3 | 5 | first value at least 3 | valid |
| 4 | 7 | at least 3 | valid |
| 5 | 8 | current end, not a start candidate | excluded |

The first valid prefix index is `3`, so starts `3` and `4` contribute two
windows ending at `5`. Equal prefix values at indices `1` and `2` come from the
zero. The search must be a lower bound for the first value at least the
threshold. Searching only for equality is wrong.

## Why the direct scan is correct

After repair for each exclusive boundary `end`, maintain these invariants:

1. `sum` equals the mathematical sum of `values[left..end]`.
2. `sum <= budget`.
3. Every start smaller than `left` gives a sum greater than `budget` for the
   same `end`.
4. Every start in `left..end` gives a sum at most `budget`.
5. `left` and `end` never move backward.

Adding `values[end - 1]` preserves the exact-sum invariant. If the sum is too
large, removing `values[left]` and advancing `left` also preserves it. The loop
stops because each move shortens the window. It cannot move past `end`: the
empty sum is zero and every budget is nonnegative.

When repair stops, invariant 2 holds. The last removed start, and every earlier
start, was invalid when considered. Adding more nonnegative values on the left
cannot make that larger window valid, which establishes invariant 3. Removing
a nonnegative prefix from the repaired window cannot increase its sum, which
establishes invariant 4.

Each subarray has one exclusive end. At that end, the contribution counts its
start exactly once. That proves the final count.

The right boundary visits `n` values. The left boundary advances at most `n`
times over the whole run. The direct scan therefore takes `O(n)` time and
`O(1)` extra space. It performs no heap allocation.

## Candidates and derived costs

# Visual 5 of 5: candidate comparison before measurement

| Candidate | Derived time | Extra storage | Role or caveat |
|---|---|---|---|
| Recompute reference | `O(n^3)` | `O(1)` | independent oracle; re-sums every range |
| Quadratic early exit | `O(n^2)` worst case | `O(1)` | cheap immediate rejection; quadratic when all fit |
| Direct sliding | `O(n)` | `O(1)` | default; proof needs nonnegative values |
| Oversized reset | `O(n)` | `O(1)` | tests frequent separators; adds a branch |
| Prefix binary search | `O(n log n)` | `O(n)` | clear algebra; pays allocation and searches |

These bounds and storage classes are derived. They do not establish which path
is fastest on a host.

For the five-element running input, the number of nonempty candidate windows is:

```text
W = n(n + 1) / 2 = 5 * 6 / 2 = 15
```

The reference re-sums a total of:

```text
T = n(n + 1)(n + 2) / 6 = 5 * 6 * 7 / 6 = 35
```

input elements across those windows. The direct trace performs five right
additions and three left removals, or eight value updates. On a 64-bit target,
the 40-byte input payload plus one 16-byte sum and one 8-byte left cursor is 64
logical bytes. The prefix vector alone stores six `u128` values, or 96 payload
bytes. These counts exclude slice metadata, vector headers, spare capacity,
alignment, allocator metadata, instructions, cache traffic, and compiler
choices.

## Why signed values break this proof

Consider:

```text
values = [4, -3]
budget = 2
```

After reading `4`, the direct repair discards start `0` because the sum is too
large. After reading `-3`, the discarded window `[0,2)` would have sum `1` and
would be valid. A pointer that only moves right cannot recover it.

The counterexample does not mean every signed-input problem is quadratic. It
means this proof and this implementation do not apply. Signed bounded-sum
counts usually need a different prefix-sum data structure, offline ordering, or
another problem-specific argument.

## Other failure modes

- Sorting the input before a subarray scan changes positions and contiguity.
- Using `< budget` instead of `<= budget` drops boundary-equal windows.
- Counting the repaired window once instead of adding `end - left` misses valid
  suffixes.
- Counting the empty window adds one result per boundary.
- Repairing at most once fails when several left values must leave, especially
  when the first removed values are zero.
- Accumulating sums in `u64` can wrap even though every element is a `u64`.
- Returning `usize` makes the answer width depend on the target.
- Subtracting before proving the leaving value belongs to the current sum can
  underflow or corrupt the invariant.
- Treating a variable-width constraint as a fixed-width window solves a
  different problem.
- Treating byte offsets as characters or grapheme clusters breaks text-window
  contracts.
- Reporting one benchmark cell as a universal winner ignores workload and
  machine dependence.

## Production adaptations

The invariant stays useful, but production contracts often need more state.

- For a fixed-width slice view, Rust's `slice::windows` expresses overlapping
  immutable windows directly. A rolling aggregate can avoid recomputing each
  view when the aggregate has an inverse.
- A minimum or maximum over a moving window needs a monotone deque. A sum needs
  one scalar because subtraction removes the outgoing value exactly.
- A constraint such as "at most `k` distinct keys" needs a frequency map and a
  distinct counter. The two boundaries stay monotone, but the repair predicate
  changes.
- Event-time stream windows need watermarks, allowed lateness, triggers,
  eviction, and state-retention rules. A finite in-memory slice has none of
  those concerns.
- General sliding aggregation depends on algebra. Invertible aggregates can
  subtract outgoing contributions. Associative but non-invertible aggregates
  need structures such as two-stack, DABA, or finger-tree based methods.
- Rust string windows must choose bytes, Unicode scalar values, or grapheme
  clusters. `str` indexes bytes, and extended grapheme boundaries require the
  Unicode segmentation rules or a suitable library.
- Unbounded streams need an explicit memory bound and a policy for expired or
  late data. `O(window width)` can still be too much when the width is driven by
  time rather than a small element count.

## Selection guide

- Use the direct sliding scan for this exact nonnegative bounded-sum count.
- Use the oversized-reset variant only when the workload contains enough
  values larger than the budget to justify its extra branch. Measure that
  claim.
- Use the prefix form when prefix values serve other queries, random-access end
  queries matter, or the algebra is easier to audit than mutable window state.
- Use the quadratic path as a small-input comparator or when immediate
  rejection dominates and simplicity matters more than worst-case scaling.
- Keep the recompute path as an independent test oracle on bounded inputs, not
  as a production candidate.
- Choose a different algorithm when values can be negative or the constraint is
  not monotone under boundary movement.

## Correctness evidence

The test suite compares every optimized candidate with the recompute reference.
It includes the running example, empty and zero-heavy inputs, oversized values,
wide-sum edges, reversal, budget monotonicity, positive scaling, and the
all-fit triangular count.

The exhaustive domain covers every array of length `0..=6`, every element in
`0..=3`, and every budget in `0..=19`. That is exactly `109,220` array-budget
cases. A fixed-seed generated campaign extends coverage to larger arrays. Its
assignment seed is `0x0000_0005_5A17_D0C5`. Generated evidence is sampled, not
exhaustive.

[TEST_STRATEGY.md](TEST_STRATEGY.md) defines the oracle boundary, generators,
comparators, invariant audit, and what a green run does not establish.

## Benchmark decision

Five candidates can win for different reasons. The frozen family contains 12
candidate-workload contrasts selected before decision-grade collection. It
includes tiny reference cases, all-fit quadratic stress, immediate rejection,
oversized resets, zero-heavy prefixes, and large direct scans.

The exact family is:

- Reference: `n8_zero_heavy_budget0` and `n8_all_fit`.
- Quadratic: `n8_all_fit`, `n64_immediate_reject`,
  `n4096_immediate_reject`, and `n4096_all_fit`.
- Reset: `n65536_all_fit`, `n65536_oversized_every64`, and
  `n65536_half_oversized_alternating_zero`.
- Prefix: `n64_zero_heavy_budget0`, `n4096_uniform_moderate`, and
  `n65536_all_fit`.

Every item compares that candidate with direct sliding on the same fixed
fixture. The root assignment seed is `0x0000_0005_5A17_D0C5`. Each fresh
process verifies the exact count, performs one untimed warmup, and reports the
median of three timed batches. Each scheduled batch contains 64 candidate
calls. Fixture generation, canaries, warmup, and work accounting stay outside
the timer. The focused manual command below overrides `inner` to one.

Each contrast uses four pilot blocks, exactly two `ABBA` and two `BAAB`, only to
estimate uncertainty. The main estimate uses 12 new blocks, exactly six of each
template. Pilot observations never enter the main estimate. The practical
ratio is `1.05`. Bonferroni assigns each of the 12 family members alpha
`0.05 / 12 = 0.004166666666666667`. Each tail receives
`0.0020833333333333333`.

For positive position median `t`, one complete block contributes:

```text
ABBA: d = ((log t_B2 + log t_B3) - (log t_A1 + log t_A4)) / 2
BAAB: d = ((log t_B1 + log t_B4) - (log t_A2 + log t_A3)) / 2
```

Exponentiating the mean main-block contrast gives the candidate-to-direct
geometric mean ratio. The fixed horizon does not grow after results are seen.
An upper simultaneous bound below `1 / 1.05` supports a practical candidate
win. A lower bound above `1.05` supports a practical direct win. Every other
interval, including equality at a boundary, is unresolved.
Invalid positions remain in raw evidence, invalidate their block, and are not
retried automatically. A separate 12-block identical-artifact A/A campaign
checks label and schedule mechanics; it does not calibrate a false-positive
rate.

[BENCHMARK.md](BENCHMARK.md) freezes workloads, timing boundaries, randomization,
invalid-attempt handling, intervals, A/A checks, artifact identity, and the
measured-versus-inferred boundary.

## Run

```bash
cargo test -p topic-005-two-pointers-and-sliding-windows
cargo bench -p topic-005-two-pointers-and-sliding-windows --no-run
cargo bench --package topic-005-two-pointers-and-sliding-windows \
  --bench bounded_subarrays -- \
  --candidate direct \
  --workload n8_zero_heavy_budget0 \
  --phase count \
  --seed 22986346693 \
  --warmup 0 \
  --samples 1 \
  --inner 1
```

The focused bench command is a harness check. It verifies the candidate against
the exact expected count before timing one descriptive sample. It is not a
decision-grade benchmark result.

## Limits and evidence boundary

This crate counts finite in-memory slices of nonnegative integers. It does not
define signed-input behavior, text segmentation, streaming time semantics,
concurrent mutation, serialization, allocation-failure injection, or an
unbounded-state policy. It contains no unsafe code.

The API contract, invariant proof, asymptotic bounds, candidate-window counts,
and logical payload equations are derived. Test results are observed only for
their declared literal, exhaustive, and generated domains. Source inspection
and linked documentation establish that named production systems expose the
described interfaces, not that this local implementation has their operational
properties. Timing and machine-mechanism claims require the frozen benchmark
and apply only to its exact source, linked image, workload, host, compiler,
order, and run window.

## Primary sources

- [Rust slice methods, including `windows` and `partition_point`](https://doc.rust-lang.org/std/primitive.slice.html)
- [Rust string representation and indexing](https://doc.rust-lang.org/std/primitive.str.html)
- [Rust integer overflow behavior](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow)
- [Unicode Standard Annex 29: Unicode Text Segmentation](https://unicode.org/reports/tr29/)
- [Basin, Klaedtke, and Zălinescu, *The Monotone Sliding Window Aggregation Problem*](https://people.inf.ethz.ch/basin/pubs/ipl15.pdf)
- [Tangwongsan, Hirzel, and Schneider, *Low-Latency Sliding-Window Aggregation in Worst-Case Constant Time*](https://arxiv.org/html/2009.13768v1)
- [Tangwongsan, Hirzel, and Schneider, *Optimal and General Out-of-Order Sliding-Window Aggregation*](https://www.vldb.org/pvldb/vol12/p1167-tangwongsan.pdf)
- [Tangwongsan, Hirzel, and Schneider, *General Incremental Sliding-Window Aggregation*](https://www.vldb.org/pvldb/vol8/p702-tangwongsan.pdf)
- [IBM sliding-window aggregator implementations](https://github.com/IBM/sliding-window-aggregators)
- [Apache Flink window documentation](https://nightlies.apache.org/flink/flink-docs-release-2.3/docs/dev/datastream/operators/windows/)
- [NumPy `sliding_window_view`](https://numpy.org/doc/stable/reference/generated/numpy.lib.stride_tricks.sliding_window_view.html)
- [SciPy `maximum_filter1d`](https://docs.scipy.org/doc/scipy/reference/generated/scipy.ndimage.maximum_filter1d.html)
