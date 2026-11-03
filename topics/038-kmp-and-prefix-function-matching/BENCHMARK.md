benchmark_required: true

Compare the direct left-to-right baseline, prefix-table KMP and dense byte DFA
for exact all-overlap results. The oracle separately compares complete slices.
The target is this fixed synthetic workload on the two declared hosts.

Workloads are frozen in `benches/matching.rs`: tiny 34/3, uniform 8192/16 and
131072/64 over 26 symbols, repeated-prefix near miss 8192/64, dense overlaps
8192/64, and binary-alphabet 8192/16. Deterministic xorshift seeds are 17/91,
19/93 and 23/97. Each process verifies all candidates before timing. Fixtures,
oracle, diagnostic comparisons and stdout are outside the timing interval.

`total` includes preparation, owned pattern copying, query, output allocation
and destruction. `prepared` includes query, output allocation and destruction;
the immutable prepared matcher stays outside the interval. Each batch performs
2000, 20 or 2 queries as fixed by the workload. Startup and cold invocation
remain outside both endpoints. Three in-process warmups precede timing.

One fresh process receives one candidate. One block contains all three
candidates. Six blocks use all six deterministic permutations, giving each
candidate each position twice. Assignment is deterministic, with no random
seed. Analysis uses the six paired process aggregates, not inner queries.
Each candidate also has one excluded whole-process warmup per cell.
Affinity is the minimum allowed CPU. Host load and thermal/cache state remain
shared and uncontrolled; warmups provide a warm-data endpoint.

Stop after the frozen schedule. Retain every attempt; a failed process aborts
the campaign and leaves partial evidence. No failed observation is replaced.
Select a candidate only if its time per query is below 95% of each rival's in
every complete block. Otherwise record unresolved. Report median and full
observed range. These are descriptive finite-campaign screens across 12 cells
per host, without a population confidence level, power claim or multiplicity
guarantee. There is no pilot-driven sample-size choice or repeated look.

Identical-artifact KMP/KMP-AA controls use the same executable, invocation,
parser and interval. Six alternating-order pairs run on tiny and near workloads.
Their ratio range diagnoses variation; it does not establish universal noise.
The runner records 280 processes: 216 main, 24 A/A and 40 warmup processes.

Source and linked binary hashes, flags, environment, order and all samples are
retained externally. Separate diagnostic traversals count direct and KMP byte
comparisons. Linked disassembly is retained for inspection. Hardware counters
and profiles isolating cache/branch causes are outside this campaign; such
mechanisms remain unmeasured. Results cannot establish a universal matcher or
compare host microarchitectures independently of their different toolchains.
