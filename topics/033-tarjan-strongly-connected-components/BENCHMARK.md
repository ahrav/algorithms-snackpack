benchmark_required: true

Predeclared before the committed-source campaign,2026-10-29.

Claim: identify the fastest correct measured partition implementation for each
fixed workload/host among Tarjan, Gabow and one-shot Kosaraju. Prepared Kosaraju
is reported separately because its transpose lifetime changes the storage contract.

Workloads: tiny-first and warmed tiny(7 vertices/8 edges), isolated/chain/cycle/
groups/permuted/duplicates/random(20,000 vertices), dense256(65,280 edges).
Groups have8-vertex cycles and one-way links. Permuted relabels them by multiplication
7919 modulo20,000. Duplicates repeats each cycle edge8 times. Random uses fixed
xorshift seed20261029 and four outgoing entries per vertex. Read-only full partitions;
mutation behavior has correctness tests. Working-set/degree/depth/locality vary;
size crossover curves, CSR and mixed-update timing are explicit revisit work.

Boundary: graph creation/process startup outside timer. Each call includes input
validation, scratch allocation, output allocation and teardown. Prepared mode's
transpose construction/retention is outside timer and reported as a separate contract.
Tiny-first times one call before any candidate call; other cases warm once and
then time fixed10,000 tiny or16 large calls(8 dense). These inner calls are subsamples.
The first-call case measures first candidate invocation on a generated resident
graph, with process startup excluded; it is not an evicted-cache experiment.
Correctness certification runs after timing. No oracle runs inside the timer.

Unit: one fresh process. Six paired complete blocks per workload and one warmup
process per candidate. Five arms include identical-code tarjan-aa. Rotations0,1,2
and each reverse give3:3 ordering for every pair. One fixed CPU per host from the
middle of its allowed affinity list. No exclusive CPU or frequency control.
Treatment/scheduling unit is process; analysis unit is paired block; inner calls
are subsamples. Generalization is limited to declared graphs/host/toolchain.

Decision: smallest median among correct candidates; select only if its median
is at least5% faster and all six block ratios favor it against every competitor
in the same contract. Otherwise unresolved. Prepared mode gets a conditional
median comparison with the outgoing-only winner, including its storage caveat.
Report medians, full process ranges and paired ratio ranges. Descriptive fixed
stopping after six blocks; no confidence level, p-value, familywise guarantee,
posthoc stopping or population superiority claim. No pilot-based precision claim.
A/A checks mechanics separately from observed noise; retain wide ranges.

All failures/timeouts remain in raw evidence and stop the campaign. Selection
requires six complete blocks; missing cells remain unresolved. Tests/certification
reject candidates before timing can select them. Large certification is a separate
linear checker; exhaustive small comparisons use repeated BFS reachability.

Build both library and executable with rustc edition2024,-O,target-cpu=native,
codegen-units=1. Freeze all executable inputs and this runner; verify path hashes
before/after. Record architecture, kernel, CPU, rustc,target cfg, allowed/pinned
CPUs, binary hash and linked disassembly. No causal cache/branch/allocation-count
claim from timings or whole-process counters. Static code/storage counts are
derived; allocation counts and hardware counters remain unmeasured.

Raw runs remain external. Compact summary, replay and verified archive receipt
belong in Git. Retain source identity from the executable-source commit even when
later commits add documentation/results only.
