# Cycle detection, topological constraints, and deadlock resolution

A wait-for edge is a prerequisite only when its target must release. A cycle
among mandatory waits proves deadlock under the snapshot contract. A pool with
interchangeable units needs a count of available units by resource class.
Check the effects of a rollback before comparing its cost.

## Run

```sh
cargo test -p cycle-detection-and-deadlock-resolution -- --nocapture
cargo run -p cycle-detection-and-deadlock-resolution --example deadlock
```

In the exclusive example, A holds X and requests Y. B holds Y and requests X.
The cycle is A, B, A. Rolling back B costs 3 and lets A finish. In the pooled
example, C starts with a second Y and can finish immediately. The successful
order is C, A, B. The hypothetical graph of possible holders still contains
the A/B cycle.

## Methods and contracts

- Mandatory edges point from waiter to required task. Reverse Kahn peeling
  removes zero-wait tasks and returns dependency-first order. Walk residual
  edges to extract a closed cycle. The residual can include other waiters.
- The pool model assumes a single snapshot, a fixed outstanding request per
  task and reusable units. Grant an eligible request, finish the task, and
  return the grant and its allocation. The net increase is its prior
  allocation. Increasing free counts preserves eligibility in other rows.
- Test each eligible rollback on that snapshot and minimize cost among single
  rollbacks that clear the residual. Ties use task ID. A stalled task with
  zero allocation appears in the report. Its rollback cannot release units.
  Multiple independent cycles can require multiple rollbacks.
- Lock acquisition in strictly increasing rank prevents circular mandatory
  waits under that ordering policy. Live recovery also needs a coherent
  ownership revision and a rollback authority. The library simulates a
  snapshot and returns a decision.

Peeling costs O(n + e) time and space for n tasks and e mandatory edges.
Dense resource reduction costs O(nrp) time for r resource classes and p
passes. There can be n passes, with O(n + r) working storage excluding the
input matrices. Trying q eligible victims adds at most q reductions. An
exhaustive scheduler can examine up to n! complete orders. Use it on small
inputs to check decisions and certificates.

Use graph peeling for mandatory waits. Use resource reduction for
interchangeable pools. Require a simulated successful release before ranking
single-victim candidates. Queue policy, timeouts, unsynchronized snapshots,
alternative classes, and future requests require their own progress model.

## Primary sources

- [Coffman, Elphick, and Shoshani, System Deadlocks](https://doi.org/10.1145/356586.356588)
- [Holt, Some Deadlock Properties of Computer Systems](https://courses.cs.umbc.edu/graduate/CMSC621/fall02/lectures/some-deadlock-properties-of-computer-systems.pdf)
- [Kahn, Topological Sorting of Large Networks](https://doi.org/10.1145/368996.369025)
- [Tarjan, Depth-First Search and Linear Graph Algorithms](https://epubs.siam.org/doi/10.1137/0201010)
- [PostgreSQL deadlock handling](https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-DEADLOCKS)
