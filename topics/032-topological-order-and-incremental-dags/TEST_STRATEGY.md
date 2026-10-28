# Test strategy and invariant audit

Scope: simple DAG admission, valid orders, wrapping-u64 updates, atomic input rejection
and edge insertion. Primary technique: exact examples and independent differential
state transitions. Secondary: exhaustive64 reverse-ranked four-vertex DAGs,48 two-edit
batches per candidate,1024 possible edge insertions, and16,384-vertex reverse path.
Oracle: independent raw-edge predecessor memoization with u128 accumulation and no
candidate ordering/update helpers. Replay: cargo tests, standalone run.py, and
scripts/test_run.py driving run.py against stub compiler and benchmark processes.
A green run establishes bounded agreement, not proof for all graphs or histories.
Concurrency, dynamic dependencies, deletion API, effects, local order repair and
allocation-failure recovery remain outside scope. No additional test is queued for
implemented behavior at publication.

- Exact example/shared consumer: a consumer processed before both edited parents
  yields a stale value. Compare complete vectors after duplicate-seed batches.
- Input rejection: validating after applying the first edit changes bases before an
  error. Require exact Error::Vertex and unchanged bases/values, then run a valid edit.
- Cyclic insertion: publishing the edge before validation corrupts graph state.
  Require exact Error::Cycle and graph equality; recompute after rejected insertion.
- Backward valid insertion: blindly rejecting reversed rank loses legal edges.
  Check a reordered valid permutation and independently evaluated values.
- Empty/noop/wrapping/cancellation: eager propagation changes work counts; checked
  addition fails the admitted numeric law; equality of one input cannot stand for
  equal output. Check modular values, cancellation, and zero propagation on no-op.
- Exhaustive updates: numeric-ID evaluation and lost repeated scheduling produce
  disagreement. Compare every vector after each of9216 candidate transitions.
- Long path: recursive DFS can exhaust the call stack. Exercise explicit frames,
  validate every edge and compare16,384 values to the independent oracle.
- Exhaustive insertion: test every endpoint pair on every small DAG. Independent
  Floyd-Warshall reachability determines cycle rejection; retained graph equality
  catches side effects; success requires complete order validity.

Each observation directly distinguishes its named violation. Mutation testing was
not run; the smallest cases above are the recorded mutation backstops.
