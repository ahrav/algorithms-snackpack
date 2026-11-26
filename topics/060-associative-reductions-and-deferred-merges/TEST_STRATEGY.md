# Test strategy

- **Scope.** Fixed-cut, single-stream exact sum/count and wrapping ordered affine composition.
- **Contract and failures.** Check empty input, invalid cuts, cut ordinals, signed limits and legal regrouping.
- **Techniques.** Fixed boundaries and seeded prefix sweeps, followed by cross-host replay.
- **Oracle.** Slice the original rows and sum in i128. Affine checks call the operations directly in chronological order.
- **Domain and bounds.** 128 deterministic vectors, sizes 65 through 192, with eight cut spacings. This samples the i32 input domain. The size proof covers legal sums.
- **Replay assets.** Run the integration tests and example, or use the Linux native runner.
- **Green meaning.** Sampled exact prefix agreement, boundary validation and fixed affine contract checks.
- **Gaps.** Persistent file formats, snapshots acquired after compaction and reset/delete handling require a storage engine model.
- **Revisit.** Compare a prefix-materialized policy, checkpoint release, read-visible resets, scoped byte accounting and deterministic floating-point trees.

| Claim and scope | Plausible violation | Observation and independent comparator | Smallest discriminating case | Mutation backstop |
| --- | --- | --- | --- | --- |
| Every retained cut survives | A reader includes later records | Exact pair at each known cut | Snapshot 2 sees 60/4 before 80 arrives | Select latest compact state for an old snapshot |
| Invalid cuts reject and empty is valid | Duplicate cuts pass validation | Exact Option variant and empty read | Cuts [2, 2] for two rows | Accept equal adjacent cuts |
| Generated prefixes agree | Chunk crosses a cut or loses a tail | Exact i128 raw-prefix comparison | Non-multiple-of-32 interval | Remove an interval's final partial chunk |
| Affine regrouping preserves order | Compose in reverse order | Direct chronological application and fixed 23/26 | Double and add 3 on input 10 | Reverse composition |
