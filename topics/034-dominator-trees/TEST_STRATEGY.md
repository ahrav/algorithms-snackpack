# Correctness strategy and invariant audit

The production candidates use predecessor intersections or parent chains. The
oracle instead removes each vertex and runs a separate breadth-first search.
Compare every full-set bit and expand each computed tree before comparison.
Check parent extraction independently against CHK's parents.

Exhaustive:512 directed three-vertex graphs, each of3 roots and all9 single-edge
insertions, including duplicate insertions. Each edited graph is checked again
with every static candidate. Boundaries cover n0,1,2,63,64,65,127,129; chains,
permuted IDs, loops, cycles, disconnected tails, duplicate edges and invalid root.
A65-vertex sequence checks five successive insertions against the oracle.
A separate deletion regression changes a two-vertex cycle's entry paths and
requires fresh sets to differ from the stale cache. It discriminates the claimed
insertion-only propagation scope. Root singleton and unreachable zero rows are
checked as part of exact matrix equality, including unreachable incoming edges.

These tests are bounded evidence. Caller-provided malformed caches and concurrent
mutation lie outside the API contract. Each benchmark process independently
checks its candidate against the removal oracle before and after timing.
