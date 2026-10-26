# Correctness strategy

Scope: immutable, validated directed multigraphs with dense vertex IDs.
Contract and failure classes: bad endpoints return an error; invalid roots are
ignored; duplicate sources/edges and self loops preserve discovery semantics;
unreachable distance is `usize::MAX`. BFS returns all distances. DFS returns
ordered first entries, ordered exits, and whether the searched region contains
a directed cycle. Supplying every vertex as a root covers disconnected regions.
Primary technique: exhaustive bounded differential checking.
Secondary techniques: exact regression examples, seeded graphs, explicit deep path.
Oracle: independently scheduled binary-heap minimum-key distances; recursive
DFS implemented only in tests and used for at most 80 vertices.
Input/model/schedule domain and bounds: all 512 directed graphs on three
vertices, six root lists each (3,072 comparisons); 80 deterministic graphs
with five edges per vertex; empty/malformed/duplicate/cyclic/diamond examples;
50,000-vertex path without recursive oracle.
Replay assets: `tests/contracts.rs`, source and `run.py`.
What a green run establishes: agreement for those finite domains and generated
inputs; deep-path completion without process-stack recursion.
Known gaps and delegated skills: arbitrary allocation failure, weighted or
concurrent mutation, exhaustive graphs above three vertices, and independent
recursive event checking at large depth. No unsafe code or concurrency.
Concrete tests to add: extend only when adding a promised new boundary.

# Invariant audit

`exhaustive_three_vertices`: Reject wrong queue/layer distances, early finish,
and wrong active colors. Compare every full distance vector to the heap oracle
and the ordered DFS struct to recursion. The exhaustive domain includes the
small diamond and two-cycle. Preserve event order in the comparator.

`empty_invalid_duplicates_and_running_example`: Reject accepted bad endpoints,
duplicate source resets, wrong cycle flags and wrong unreachable state. Check
the constructor error and exact example vectors. Use independent recursion
and heap comparisons for empty graphs, duplicates and self loops.

`seeded_graphs`: Reject neighbor-order reversal, stale root state and cross-edge
misclassification. Compare full outputs to the heap and recursive oracles for
seed42,n1..80,five outgoing edges per vertex and two roots.

`deep_path_and_cross_edge_are_distinct_from_back_edge`: Reject process-stack
recursion and treating any seen neighbor as a directed cycle. Observe50,000
entries and the expected reverse endpoint; compare both iterative outputs.
The independent four-vertex diamond recursion establishes the cycle boundary.

Mutation backstop: the exhaustive directed set includes self loops, cycles,
shared children, disconnected roots and convergent paths. Replacing active-only
cycle detection with any-seen detection fails the diamond. Finishing on entry
fails the running example's exact finish vector. Marking all outgoing neighbors
before descending changes recursive DFS ordering on convergent paths, which
the exact recursive comparison detects. The large-path mutual agreement is
explicitly narrower than an independent full event oracle at that depth.
