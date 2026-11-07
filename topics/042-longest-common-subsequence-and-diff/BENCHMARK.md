# Benchmark decision

benchmark_decision: true

Claim: select the fastest correct script-producing implementation among matrix,
Hirschberg and compact-history Myers for each declared synthetic workload/host.

Frozen before collection: lengths 16/256/1024; identical, one replacement per
64 bytes, unrelated, repeated and unequal lengths. Repetitions 200/20/10 per
process respectively. Unequal target length is n/16. Repeat replaces first half
of an all-a source with b. Unrelated uses target byte 255 absent from source.

Treatment application is one fresh candidate process. Analysis is one paired
three-candidate block. Eight measured blocks per cell rotate/reverse candidate
order deterministically; one warmup process per candidate/cell is discarded.
This fixed-order screen has no randomized population or confirmatory confidence
claim. Repetitions inside processes are subsamples. Each binary/host is one build
and machine realization. Shared CPU/cache/allocator/thermal interference remains.

Timing includes allocations, score search, traceback, output construction,
black_box consumption and teardown. Generation, edit-graph oracle, replay,
startup and printing are outside timing. One validated candidate invocation in
each process warms its code and allocation path. CPU affinity is pinned to the
first available logical CPU. Native target flags; hardware/toolchain differ.

Stop after exactly eight blocks regardless of result. Candidate wins only when
its time is below 0.95 times every other candidate in every paired block.
Otherwise unresolved. Report median and full process range. No significance,
familywise confidence, universal optimum or hardware-causal claim. A/A control:
eight paired matrix processes on sparse256, alternating A/B labels. Retain every
attempt; failed correctness excludes that candidate's timings from selection.

Deterministic counters report score states, byte comparisons, score-buffer
allocations and peak live score capacity. Counter meanings differ by candidate;
vector headers, pairs and script capacity are separately documented. Counters
support algorithmic-work claims. Hardware branch/cache mechanisms are unmeasured.
