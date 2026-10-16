# Correctness and invariant audit

Independent model: an eager vector applies mul*x+add directly and independently
folds each queried slice with wrapping addition. It does not reuse Tag methods,
tree splitting, pushing or composition. Every benchmark query and final full
state also checks against that model before timing; checksums agree across arms.

Five library groups cover exhaustive bounded update/query intervals for lengths
0..8 and four first tags, followed by a whole-range update; 2048 seeded overlapping
updates for lengths1/3/17/65 per candidate with wrapping values; explicit temporal
assign/add witnesses and the running example; every ordered fold over thirteen
distinct strings; and invalid update rejection before mutation. Public integration
checks empty domains, whole/partial chronological updates and assignment to zero.
The example checks before18/after30/query10. These are bounded checks, not a proof.

Discriminating invariants: reversed affine composition fails assignment-then-add;
missing push fails partial query after whole update; stale pull fails overlapping
updates; off-by-one bounds fail exhaustive empty/end-at-n queries; reversed right
accumulator fails distinct-string concatenation; checked sum arithmetic instead
of wrapping fails maximum-value cases. Bounds are validated before mutations.
Exact arithmetic rollback, concurrency, allocation failure injection, SQL window
semantics and production vectorized/holistic execution are not implemented.

Run all workspace gates listed in the task, including all-targets integration
checks. No tests merely assert private representation shape.
