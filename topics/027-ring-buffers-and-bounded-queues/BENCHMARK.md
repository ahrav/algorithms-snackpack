benchmark_required: true

# Fixed comparison

Compare five correct exact-capacity reject-on-full FIFO candidates. The target population
is the declared warm u64 traces on each named host, compiler and build. Treat each host
separately. The ordered-map oracle is outside timing and is not a speed candidate.

| Case | Capacity / initial occupancy | Trace | Repetitions per process |
|---|---|---|---|
| tiny | 4 / 0 | 64 push/pop pairs | 1000 |
| steady_power2 | 1024 / 512 | 8192 pop/push pairs | 8 |
| steady_odd | 1000 / 500 | 8192 pop/push pairs | 8 |
| burst | 1024 / 0 | 1032 pushes then 1032 pops | 32 |
| large | 32768 / 32768 | 4096 pop/push pairs | 2 |
| full | 1024 / 1024 | 8192 rejected pushes | 16 |
| lifecycle | 31 / 0 | create, 64 push/pop pairs, destroy | 1000 |

Payloads are deterministic increasing u64 values within each trace, repeated unchanged.
Capacity, occupancy, allocation lifecycle and admission failure vary. Payload width,
contention, batch APIs, cold cache and production tails are outside this population.

Timing covers operation dispatch, admission, payload moves, index/length updates and
checksum consumption. Setup, preload, oracle, final-state checking and final steady teardown
are excluded. Lifecycle includes per-cycle creation/destruction. One untimed full-trace
warmup precedes each measured process. Every warm and timed output is checked independently.

Ten process blocks, fixed stop, no tuning after results. Five-candidate rotations give each
candidate every position twice; the second five rotations are reversed. Case order reverses
every other block. Assignment is deterministic with no random seed. Distinct A/B labels run
the identical deque binary with alternating control order. All 490 process outputs per host
are retained. Inner iterations and operation counts are subsamples, not independent units.

A candidate is selected only when it has lowest median and every paired rival/best ratio
exceeds 1.03 in all ten blocks. Otherwise report unresolved rivals. This is a descriptive
all-pairs rule without confidence coverage, pilot variance, alpha budget or inferential
multiplicity claim. Retain medians, min/max, every paired ratio and A/A variability. Do not
pool the pre-teaching and committed-source campaigns as additional replication.

Separate processes reset allocation/state. Processes share an unpinned host, so interference
and time drift remain possible. There are no planned exclusions. On a failure, preserve the
attempt and mark the corresponding comparison unavailable rather than selecting from an
incomplete block. No candidate-only hardware counters are collected. Inspect exact linked
code only for static code-shape observations, and do not infer hardware causality from the
whole process, which also computes the untimed oracle.

Build: `rustc --edition 2024 -O -C target-cpu=native`, with per-host compiler, CPU features,
affinity, kernel and final binary hash in raw identity. Use `run.py` and its exact source
manifest for replay. Receipts retain external archive identities and compact aggregates.
