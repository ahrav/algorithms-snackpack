benchmark_required: true

## Frozen claim and design

Compare policy request-processing CPU separately from miss counts. Equal-size
sequential demand-admit contract, direct dense-key index. Four candidates: scan,
linked, clock and segmented. Scan/linked have identical logical LRU outcomes;
other candidates have independent specified outcomes. The target population is
these deterministic traces on the two named hosts and exact built images.

Workloads: tiny capacity3/N9; hot, scan, capacity-plus-one loop, four phases and
uniform at capacities4/64/1024, N32768. Fixed xorshift64 seed123456789abcdef0;
run.py independently generates keys. All start logically empty. CPU caches are
warm without flushing. CPU affinity uses one allowed CPU. Setup allocates
metadata and is timed separately; trace generation, hit-buffer reservation,
snapshots, output formatting, oracle and teardown lie outside request timing.
Request timing includes API dispatch, checks, policy transitions, work counters,
black_box and writing the hit buffer. This is a simulator cost boundary.

Treatment unit: one fresh process. Analysis unit: one complete block with all
four candidates. Subsamples: requests within a process. Six fixed blocks per
cell; each forward rotation is followed by its reverse. No stochastic treatment
assignment; each pair balances linear position effects. No host-population,
nonlinear-drift, carryover or independent-process claim. No pilot variance or
power estimate; descriptive exploratory screen only. No confidence level or
familywise significance claim. No optional stopping or favorable reruns.

One warmup process per candidate/cell. Identical binary A/A: six two-process
blocks with alternating labels A/B on linked/scan capacity4. Same parser,
affinity, oracle, image and environment. Record ratios and integrity separately;
this diagnostic does not calibrate inferential coverage.

Report request and setup median/min/max. CPU winner must be >5% faster than
every other candidate in each of six paired blocks. Otherwise unresolved.
Report minimum-miss policies including ties independently. Timing winners use
all correct candidates; policy choice needs miss cost. Failed/partial attempts
are retained. A failed oracle rejects timings; an incomplete cell stays
unresolved. Frozen deterministic source, runner, schedule, transferred hashes,
compiler/flags/features, architecture/kernel/CPU/affinity and linked-image hash
are retained. Different host compilers prevent hardware-only causal claims.

No backend, loader, allocator, service latency, concurrency, total RSS, hardware
counter, or general policy optimality measurement is claimed. Victim inspections
and list operations are explicit implementation counters. Generated assembly
is retained only as source-shape evidence; no counter-derived mechanism claim.
