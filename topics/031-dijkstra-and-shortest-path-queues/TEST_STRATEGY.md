# Correctness and invariant audit

Independent oracle: synchronous Bellman-Ford, previous-round copies and optional
`u128` labels. Candidates use minimum extraction and `u64` sentinel labels.
Compare complete vectors, including unreachable positions, before timing.

- Exhaust all 19,683 directed three-vertex graphs, including loops, with each
  edge absent, zero or one. Check every source and all six candidates.
- Check 160 deterministic multigraphs, all sources, against the oracle, once
  with weights `0..=64` and once with weights `{0,1}`. The binary pass runs the
  0/1 deque on graphs with parallel edges, unreachable vertices and distances
  above two, and Dial with two buckets so its cursor wraps repeatedly.
- Bound 0/1 deque improvements below `2V`: a settled level `c` pushes each
  vertex at most once with `c+1` and once with `c`. A deque with swapped ends,
  or a plain FIFO, still returns exact distances through stale rejection but
  exceeds this bound on the binary multigraphs.
- Reject invalid endpoints (source and target), vertex counts above `u32::MAX`,
  empty-graph queries for every candidate and the oracle, and specialist
  domain failures at the first rejected weight: `65` for Dial, `2` for the
  deque, and the running example's weight `20` for the deque. Admit `64`/`1`.
- Check maximum-u32 path sums, duplicate decreases, disconnected vertices,
  zero cycles, weight-64 Dial admission and radix bit boundaries.
- Inspect stale removal in the running example: two records must be rejected.
  The example binary asserts the same count; contract tests check five edges.
- Bound indexed queue records by vertex count and require zero stale entries.
- Test indexed position maps after swaps/decreases; a stale position maps a
  later decrease to the wrong vertex and fails exact extraction checks.
- Test full-width radix keys, including `1<<63` and `u64::MAX`, directly inside
  the queue module. A 64-bucket array or incorrect equal-key handling fails.

These checks reject discovery-time finalization, non-strict equal enqueue,
incorrect queue direction, sentinel/width mistakes and lost decrease-key state.
They provide bounded evidence, not a proof over every admitted graph. The
nonnegative minimum-extraction argument and admitted sum bound supply the
algorithmic invariants. Allocation exhaustion follows Rust's allocator behavior.
