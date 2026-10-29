# Test strategy

Scope: immutable directed SCC partition, prepared-transpose precondition and graph edits.
Contract and failure classes: empty/disconnected graphs, loops, duplicate edges,
invalid destinations, component split/merge, active state and deep recursion risk.
Primary technique: exhaustive independent-model comparison for all66,067 directed
graphs through four vertices, including loops, each with three algorithms.
Secondary techniques: exact regressions, malformed-certificate rejection, deep
100,000-vertex chain/cycle and edit/recompute checks.
Oracle: independent repeated BFS reachability; canonical minimum-member labels.
Large certificate: restricted forward/reverse reachability plus quotient Kahn;
these operations share no SCC algorithm with the candidates.
Input/model/schedule domain and bounds: finite graphs<=4 exhaustive; two100k
adversarial shapes; fixed named benchmark graphs. Sequential immutable snapshots.
Replay assets: tests/contracts.rs, examples/components.rs, measurements/run.py,
measurements/test_run.py (stub-process timeout and unparsed-output retention, flag
refusal and full schedule) and measurements/test_analyze.py (synthetic two-host
reduction; mixed-source, unfinished-campaign and inconsistent-metadata rejection).
What a green run establishes: exact agreement in the enumerated finite domain
and independent certificates for the large inputs actually executed.
Known gaps and delegated skills: concurrency, allocator failures, dynamic SCC
maintenance, stale transpose misuse and persistent identity are separate contracts.
Concrete tests to add: dynamic-update oracle if a future maintained index is added.

# Invariant test audit

all_directed_graphs_through_four_vertices: wrong active/low propagation or root
closure changes a partition. BFS supplies exact equivalence; canonicalization
preserves all pair memberships while discarding intentionally unspecified IDs.
The three-vertex unresolved-finished-sibling graph is a minimal targeted backstop.

active_finished_sibling_and_completed_target: treating returned frames as emitted
splits `[0->[1,2],1->[0],2->[1]]`; treating emitted targets as active loses vertex1
in `[[],[0]]`. Exact BFS labels discriminate both failures; loops/duplicates retained.

certificate_rejects_splits_merges_and_malformed_labels: all-one labels wrongly
merge the example; singleton labels split its cycles. Restricted reachability
rejects the merge and quotient acyclicity rejects the split. Missing, sparse and
out-of-range IDs must return false rather than pass a weaker coverage proxy.
`[0,1,2,3,4,5,0]` merges vertex0 with the unreachable sink6 and is rejected by
the forward restricted scan, which the all-one labeling leaves to the reverse scan.

deep_chain_and_cycle_use_heap_frames: recursive depth or missed edge processing
fails100k inputs. The exact component count is100k for the chain and1 for the
cycle; independent certification checks all edges and memberships.

graph_edit_rebuild_changes_partition: stale component reuse misses insertion6->1
or deletion4->3. Independent BFS runs after each edit through `check`, which also
runs prepared Kosaraju on a fresh transpose; exact all-one result checks the merge
and distinct3/4 labels check the split.

invalid_destination_rejected: the exact validation assertion panics for one vertex
with edge0->1. The immutable input cannot be partially edited. This checks the
Tarjan admission path; Kosaraju/Gabow validate through their entry assertions.

every_entry_rejects_invalid_destinations_and_mismatched_transposes: `kosaraju`,
`gabow`, `oracle`, `transpose` and `kosaraju_prepared` each panic on the one-vertex
graph with edge0->1; prepared Kosaraju also panics on an invalid or shorter
transpose and agrees with one-shot Kosaraju on an exact transpose.

canonical_rejects_labels_outside_vertex_range: `[10, 10]` is a valid two-element
partition whose labels exceed the vertex count. The documented dense-label assertion
names the bound; a raw index panic or a silent remap would both fail this test.

Mutation backstop: explicit wrong partitions and two state-confusion regressions.
A mutation-tool campaign is unmeasured; these named counterexamples are executable.
