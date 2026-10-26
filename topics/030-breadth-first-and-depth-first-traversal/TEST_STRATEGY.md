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

| Test / claim | Plausible violation | Exact observation and independent oracle | Small discriminating input |
|---|---|---|---|
| exhaustive_three_vertices | Wrong queue/layer distance, early finish, wrong active color | Each full distance vector equals heap oracle; full DFS struct equals recursion; no sorting erases event order | Diamond and 2-cycle in exhaustive set |
| empty_invalid_duplicates_and_running_example | Invalid endpoints accepted; duplicate source reset; cycle or unreachable state wrong | Constructor error and exact example vectors/cycle flag; independent recursion/heap for duplicates | Empty graph, invalid endpoint, duplicate self/neighbor edges |
| seeded_graphs | Neighbor-order reversal, stale root state, cross-edge misclassification | Exact full outputs versus heap and recursion after each complete traversal | Seed42, n1..80, two roots |
| deep_path_and_cross_edge_are_distinct_from_back_edge | Recursive process-stack dependence; a finished neighbor treated as a cycle | 50,000 entries and exact reverse endpoint; both iterative outputs equal; independent bounded diamond recursion | Four-vertex diamond has no directed cycle |

Mutation backstop: the exhaustive directed set includes self loops, cycles,
shared children, disconnected roots and convergent paths. Replacing active-only
cycle detection with any-seen detection fails the diamond. Finishing on entry
fails the running example's exact finish vector. Marking all outgoing neighbors
before descending changes recursive DFS ordering on convergent paths, which
the exact recursive comparison detects. The large-path mutual agreement is
explicitly narrower than an independent full event oracle at that depth.
