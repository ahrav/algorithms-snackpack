benchmark_required: true

# Frozen comparison

| Workload | Shape | Candidates |
|---|---|---|
| tiny | 17 records of 8 bytes | standard, exact, 1.5, double, reserved |
| medium | 8,193 records of 8 bytes | same |
| large | 262,145 records of 8 bytes, just beyond doubling boundary | same |
| wide | 65,537 records of 256 bytes | same |
| reuse | first fill plus 16 clear/refill cycles of 8,193 records | same |
| oscillate | fill to 1,025, then 4,096 pop/push pairs | standard retain, tight shrink, quarter shrink |
| drain | first fill plus four full drain/refill cycles of 8,193 | same shrink candidates |

The reserved candidate knows the exact peak before allocation. This information
advantage is part of its contract. The other grow policies are online. Reuse
includes the first fill; it does not isolate a preallocated steady-state loop.

## Units and controls

Each fresh process receives one candidate and workload. Its aggregate rate is one
sample. Inner repetitions are subsamples, never independent observations. Ten
fixed rotating mirrored blocks contain each candidate once per workload and two
identical standard-candidate controls under labels A/B. There are 450 processes
per host per campaign. The five-policy family uses every position twice. The
rotation with reversal does not balance three candidates over ten blocks: `std`
and `quarter` occupy positions 0/1/2 in 3/3/4 blocks and `tight` in 4/4/2, so
launch-order drift is spread unevenly for `tight`. Assignment is
deterministic, not random; no randomization-based inference is claimed.

A process first checks exact pop results and final records against an independent
ordered-map oracle, releases that storage, and performs one warmup. The timed
boundary includes allocation, operations, full output scan/checksum, and teardown.
Input generation, validation, process startup, and printing are excluded. The
runner fixes repetitions before collection and retains every output/failure.

Each host runs one campaign at a time. Hosts, compilers, source, runner, flags,
linked binary, and available CPUs are identified. Scheduling and page placement
are unpinned. Warm allocator/cache state can carry between processes, so fresh
processes are not proof of independence. Only these declared host/workload
observations are compared; no host-population or instruction-set causal claim.

## Selection and stopping

Fixed ten-block budget, with no peeking, retuning, or significance stopping.
Report the lowest median. Select it alone only if every rival is over 3% slower
in every paired block ratio. Otherwise report unresolved rivals. All sample and
ratio ranges are descriptive, not confidence intervals or error-controlled tests.
This exploratory budget has no prospective power claim or multiplicity guarantee.

A/A checks identical artifact/settings and matching output mechanically, then
reports descriptive ratio ranges. One campaign cannot calibrate a false-positive
rate and does not establish a noise floor. Initial and committed campaigns remain
separate; do not pool them or choose whichever result is more favorable.

## Mechanism boundaries

Counters record explicit grow/shrink API requests, actual reported capacity,
growth-address changes, and initialized lengths at growth requests. The last is
hypothetical full-copy work. None records allocator-internal call count, physical
bytes copied, shrink-address changes, peak resident memory, or physical release.
Code inspection records the actual linked paths without using whole-process
oracle counters as candidate-only evidence.

No cold-allocator, tail-latency, production-contention, universal-optimum, or
allocator-causality claim. Full raw observations stay outside Git; compact
aggregates and identity/archive receipts are committed.
