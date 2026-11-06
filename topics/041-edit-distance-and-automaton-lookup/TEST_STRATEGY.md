# Test strategy

Scope: global byte distance and complete ordered dictionary IDs.
Contract and failure classes: unit edits; empty inputs; global leading-prefix
cost; duplicate IDs; threshold boundaries; band staleness; word-width fallback;
raw bytes; row-minimum pruning; exact output order.
Primary technique: exhaustive agreement with an independent shortest-path edit
graph over all 63 binary words through length five (3969 ordered pairs).
Secondary techniques: thresholds 0..6; dictionary agreement at k=0..3; explicit
0/1/63/64/65/128-byte boundaries, usize::MAX threshold, and native workload
full-matrix differential tests outside timing.
Oracle: Dijkstra exploration of insertion/deletion/alignment graph edges. It
calls no candidate helpers. The benchmark uses a separate complete matrix.
Input/model/schedule domain and bounds: sampled byte/length boundaries and
exhaustive finite binary domain. Performance schedule is in BENCHMARK.md.
Replay assets: tests/contracts.rs, examples/edit.rs, measurements/run.py.
What a green run establishes: exact agreement over the declared finite domain
and workloads, including complete ordered IDs. It supplies bounded evidence.
Known gaps and delegated skills: Unicode transformations, transpositions,
weighted edits, multiword recurrence and edit scripts are revisit scope.
Concrete tests to add: add independent model cases when extending that scope.

## Invariant-test audit

Claim and scope: exhaustive pairwise edit-graph and dictionary agreement.
Plausible violations: charge equality one; skip a band boundary; prune on final
cell; discard duplicate IDs; clear global Myers boundary.
Observation: exact distances, thresholded Options, and full ordered ID vectors.
Oracle and independence: edit graph with shortest-path queue, separate storage
and control flow; benchmark full matrix has separate allocation/indexing.
Comparator: integer/Option/vector equality, preserving IDs and multiplicity.
Smallest discriminating cases: cat/xxcat for global prefix, c->cat for subtree
pruning, identical bytes for substitution, empty/nonempty for band boundaries.
Mutation backstop: retained mutation checks separately record executable kills.

Claim and scope: word-width, global boundary and byte semantics.
Plausible violations: shifting by 64; free leading prefix; implicit Unicode
normalization; dropping duplicate terminal IDs.
Observation: exact results at 63/64/65 and empty patterns, distance(cat,xxcat)=2,
distance(ab,ba)=2, UTF-8 byte distance(e-acute,e)=2, duplicate IDs [0,1,3].
Oracle and independence: full-row crosscheck at longer lengths plus explicit
literal expected outcomes. The exhaustive test separately validates rows.
Comparator: exact integer and full vector equality.
Smallest discriminating case: pattern length one, leading mismatch, duplicate cat.
Mutation backstop: global-boundary mutant must fail the literal prefix assertion.
