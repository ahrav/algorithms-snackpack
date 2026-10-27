# Correctness and invariant audit

Independent oracle: synchronous Bellman-Ford, previous-round copies and optional
`u128` labels. Candidates use minimum extraction and `u64` sentinel labels.
Compare complete vectors, including unreachable positions, before timing.

- Exhaust all 19,683 directed three-vertex graphs, including loops, with each
  edge absent, zero or one. Check every source and all six candidates.
- Check 160 deterministic multigraphs, all sources, against the oracle.
- Reject invalid endpoints, empty-graph queries and specialist domain failures.
- Check maximum-u32 path sums, duplicate decreases, disconnected vertices,
  zero cycles, weight-64 Dial admission and radix bit boundaries.
- Inspect stale removal in the running example: two records must be rejected.
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
