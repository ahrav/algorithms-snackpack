# Contract checks

Scope: immutable DAGs, valid endpoints, reflexive reachability, checked closure budget.
Failure classes: invalid endpoint, cycle, self-edge cycle, insufficient payload budget,
scratch or filter sized for another graph (panic).
Primary technique: independent per-source breadth-first traversal.
Secondary techniques: exhaustive ordered DAG edge subsets and reversed-ID relabeling.
Oracle: queue-based BFS shares graph storage, with separate traversal and no ranks,
labels or closure recurrence. Model equality checks every source/target result.
Domain: all 1024 five-vertex forward-edge subsets and their reversed-ID copies;
word boundaries 63/64/65/129; duplicates, cross edges, disconnected, empty, singleton;
a filter built for a larger graph panics on entry; the 20,000-vertex chain drives label
construction's explicit stack to depth 20,000 without recursion, while `search`'s stack
holds one entry on a chain and grows with out-degree in the five-vertex subsets.
Replay: tests/contracts.rs, examples/running.rs and scripts/run.py.
Green establishes these bounded checks and oracle agreement for benchmark batches.
Gaps: exhaustive arbitrary permutations, allocation-failure injection, dynamic graph
updates, concurrent publication and production-size closure are outside scope.

## Invariant audit

Claim: all candidates return the same exact reachable set.
Violation: forward rather than reverse row union, numeric-ID rank, interval pass as true.
Observation: exact booleans for each source/target against BFS.
Comparator: direct equality, including diagonal true and disconnected false.
Discriminating cases: reversed chain, diamond cross edge, and the interval false positive
0->4, 1->3, 2->4 on pair (2, 3), asserted to pass `may_reach` and fail the oracle.
The exhaustive test includes these structures in both ID directions.
Mutation backstop: structural counterexamples are retained; no mutation runner claim.

Claim: malformed input and oversized closure are rejected explicitly.
Violation: silently accept cycle or allocate beyond the caller's payload budget.
Observation: exact error strings for endpoint, cycle and budget; exact-fit succeeds.
Oracle: literal inputs and independently calculated two-row 16-byte payload.
Smallest cases: one invalid edge, one self edge, two-vertex cycle, budget 15 vs 16.
Mutation backstop: deleting a corresponding rejection changes the observed error.
