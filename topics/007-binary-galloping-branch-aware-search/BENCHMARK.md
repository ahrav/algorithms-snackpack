benchmark_required: true

# Frozen protocol

Five correct lower-bound candidates, ten workloads, 12 independent process
blocks per workload, plus a 12-block identical-artifact A/A control. Fixed
624 processes; no optional stopping, exclusions, or winner-seeking reruns.
Raw failed/partial attempts remain retained if execution fails. No pilot
variance or power claim: 12 blocks is a fixed practical budget and may leave
5% effects unresolved.

Workloads: 8/4096/1048576 values for uniform keys; 4096/1048576 for keys in
0..16; 4096 for repeated, monotone, duplicate-run, and alternating
below/above-range queries; 1048576 for first-pass uniform queries. Distinct
values are `2*i`; duplicate runs use `i/32`. A fixed LCG seed
`0xb817934a135dac71` produces 128 keys per batch. Exact tuples are in run.py.

Every lookup returns the same first-duplicate index. The oracle counts all
values below the key. A candidate's timed workload is checked before and
after warm timing; first-pass checks follow timing. Applicable correctness
tests run before the campaign. No search allocates.

Timing includes search calls, function dispatch, black_box, and checksum
accumulation. Setup allocation/construction, oracle checks, output, and
teardown are outside lookup timing. Setup and teardown nanoseconds are
recorded separately; process wall time is diagnostic and includes oracle
work. Warm mode uses four untimed batches, then fixed repeated batches.
First mode uses one batch after construction with no oracle scan or warmup.
Construction writes leave data in caches: first-pass is not a cache-flushed
cold or DRAM-residency claim. Repeated inner calls are subsamples, not units.

The process is the treatment-application/subsample container. A complete
five-process block is the pairing and analysis unit. Candidate order rotates
every two blocks; adjacent blocks reverse that rotation. All pairwise orders
occur six times in each direction. This controls first-order ordering effects
but cannot guarantee absence of shared-machine interference or carryover.
Generalization is limited to this machine, compiler, window, and fixed inputs.

Geometric-mean ns/query is the point criterion. Paired log-ratio Student-t
intervals use 11 degrees of freedom and Bonferroni across 100 candidate
contrasts plus one A/A contrast, with two-sided familywise 95% coverage under
the normal independent-block log-ratio model. This is a model-based interval,
not a distribution-free guarantee. Standard-library quadrature computes the
critical value and checks a known t quantile. A unique selection requires
every competitor's lower bound against the point-fastest candidate to exceed
1.05. Otherwise report the point-fastest and unresolved candidate set.
Min/max process timings accompany each geometric mean. The A/A control uses
the same standard candidate and binary through the same execution/parser/
interval path under two labels; mechanical completeness and null uncertainty
are reported separately.

Flags: rustc edition 2024, opt-level=3, debuginfo=1, default target features,
no target-cpu=native, LTO, PGO, or custom panic flag. Linux runner pins the
first allowed CPU when taskset exists; macOS has no affinity. Record source,
runner, linked image, compiler, target cfg, CPU, architecture, kernel and
available CPUs. Retain linked-image disassembly. Code-shape observations
cannot establish dynamic branch misses or candidate-only latency causality.

Required Arm and x86 hosts rejected initial execution and one bounded retry
because Midway authentication expired. No architecture/CPU/kernel/compiler
claims from old runs are reused. Their exact committed-source correctness,
example, campaign and receipts remain pending. Local Apple M1 Pro results
are supplementary evidence, not Linux-host completion. A truthful draft may
publish while this requirement is queued.
