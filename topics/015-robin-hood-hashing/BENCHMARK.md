# Benchmark decision

benchmark_required: true

Claim: choose the best tested correct candidate per declared workload and host.
No universal optimality, collision-security, per-operation tail or ISA claim.

Workloads: capacities32/2048 at50% hits;32768 at87% with hits/misses/half hits;
262144 at87% half hits;90% hot keys;256 constant-home at87%;4096 at87% churn;
32768 at87% build and first batch. Exact floor(n=capacity*percent/100), queries,
seed and repeats are in benches/portfolio.rs. Each process uses16384 queries.

Timing: hash, probes and checksum included. Lookup excludes generation, setup,
allocation, BTreeMap oracle, diagnostic counts and final teardown. Churn includes
remove, reinsertion, lookup; its unit is one triplet. Build includes allocation,
all inserts, one lookup and drop; unit is inserted key. First batch skips warmup
but preceding oracle accesses code and another table; it is not cold/startup.
Full process elapsed time is retained separately and not used for selection.

Sampling:12 process blocks, six permutations repeated twice, same queries/seed
within a paired block. Rotate workload order across blocks. One batch warmup
except first_batch. Inner repetitions are throughput work, not independent samples.
Freeze five source paths and runner hash before compilation. Verify every candidate
against independent BTreeMap and checksums before accepting times.

Selection: lowest median; uniquely selected only if every paired rival/winner
ratio exceeds1.03 in all12 blocks. Otherwise unresolved. Min/max ranges describe
dispersion, not a confidence interval. No discretionary reruns after fixed stopping.
Every selection is provisional until the identical-artifact A/A control required
by METHODOLOGY.md runs through this same pipeline.

Controls: same layout, hash, input and capacity; all insert paths presearch keys;
RobinScan removes the miss stop while retaining Robin placement. Pin one available
CPU on Linux; macOS has no affinity pin. Capture compiler, kernel, CPU, flags,
target cfg, available CPUs, binary/source hashes. Optimized rustc native target.
Retain generated assembly; verify scalar DIB comparison without attributing cache
or branch costs from timing alone. No PMU/candidate-only counter claim.

Limits: no A/A calibration, forced cold cache, other payload widths, resize,
concurrency, keyed hash defense or production map comparison. Both declared Linux
hosts unavailable due expired Midway session; exact replay pending. No other host
substitutes for either requirement. Raw data and replay inputs remain external.
