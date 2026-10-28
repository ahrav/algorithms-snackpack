benchmark_required: true

## Frozen protocol

Compare ordering (FIFO Kahn baseline, iterative DFS, minimum-ID Kahn) and cached
value updates (full baseline, dirty scan, change-driven heap). Any valid order is
accepted for the ordering experiment; canonical ordering is an additional contract.
All update candidates maintain the complete wrapping-u64 value vector.

Workloads are deterministic with reverse-numbered vertices: tiny8/fan2; chain4096
root and leaf; 16,384 vertices in chains of64 with one component-root edit;
4096/fan8 sink and unchanged-root edits; 1024/fan32 root edits. Ordering uses tiny,
chain_root,islands,dense_root. Updates use all seven. Each sequence has16 assignments.
Candidate trials perform 8..512 sequence repetitions per process, fixed by workload.
These repetitions are subsamples. The source generator is the replay specification.

Ordering includes indegree/color/frame workspace, output construction and output
teardown. Graph admission and candidate correctness checks are outside timing.
Updates include edits, scheduling, persistent-buffer growth if any, operation counters
and black_box calls. Initialization, oracle, full output copy, final validation and
teardown stay outside. Graph and initial-evaluator durations are retained separately
as single setup observations, not a comparative setup benchmark. Working sets are
process-initialized and warmed by preflight; cold hardware-cache latency is unmeasured.

Run one warmup and six fresh measured processes for every candidate/workload/host.
Each block reverses candidate order: ABC,CBA,ABC,CBA,ABC,CBA. Every pair has3:3
relative order. Separate processes reset state. Record process wall time separately
from internal operation timing. No affinity binding; retain available CPU affinity,
CPU model, features, compiler, kernel, source/runner/binary hashes. Distinct host
compilers limit causal hardware comparisons. Exact flags: edition2024,opt-level3,
target-cpu=native,codegen-units1. No pilot-based sample adaptation or optional looks.

A/A uses identical FIFO code under fifo/fifo_aa and identical full-update code under
full/full_aa, each on tiny, through the same runner/parser/six-block scheme.
Timing noise is diagnostic. Mechanical equality and null calibration are separate.

## Selection and evidence

Analysis unit: a fresh process. Report median and min/max nanoseconds per operation.
For each of22 workload/host cells, select the lowest-median correct candidate only
when its maximum is below every rival minimum divided by1.05. Otherwise report
unresolved and retain the lowest median descriptively. This finite observed-range
rule is exploratory; it supplies no confidence interval or simultaneous error-rate
claim. Six processes and one execution window limit generalization. An isolated
small result can be quantized or dominated by timer/control overhead.

Retain every failed, timed-out, partial or successful attempt. A missing process
prevents that cell's declared selection. Reject incorrect candidates before timing
selection. Check every update in preflight against independently memoized raw-edge
values; validate final complete values after timing. Verify orders independently.

Operation counters measure evaluated vertices, parent reads, discovery/propagation
edges and complete-order entries. They add common instrumentation overhead; algorithm
paths execute different counters. Disassembly is static code-shape evidence. Counters
and elapsed time alone establish no cache/branch/instruction mechanism. Record a
bounded perf-stat availability check; unavailable counters leave mechanism inferred.
Raw output lives outside Git with an archive receipt. Timing selection is restricted
to tested graph shapes, single-threaded recurrence, and declared timing boundaries.
