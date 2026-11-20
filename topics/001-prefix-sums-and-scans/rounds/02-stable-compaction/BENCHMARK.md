# Benchmark decision

`benchmark_required: true`

The experiment compares one cursor, materialized ranks, preassigned tile
offsets and selected-run copying. The four candidates share values and flags.
The grid uses 16 records with a 50% random mask, plus 4,096 and 1,048,576 records
under random 1%, 50%, and 99%, alternating, and alternating 256-record runs.
The script fixes the input seed. It reports exact counts and run lengths.

Time the compaction call into preallocated buffers after three warm calls in
each fresh process. Input generation, buffer and scratch allocation, the oracle
and teardown run outside the timer. Each process reports its mean nanoseconds
per call. A process block gives each candidate one position. Cyclic rotations
of forward and reverse orders give every candidate two appearances at each
position across eight blocks. One unscored block warms each cell. The run ends
at the predeclared count. Two eight-pair cursor controls cover the smallest
and largest sizes. The candidate with the lowest median wins if it uses at
most 0.95 times each rival's time in every matched block. Otherwise the
result remains unresolved.

Source gives logical operation counts. Linked disassembly is retained for the
incorrect Arm build and the final kernels. The campaign does not measure
hardware counters, allocator work, or cold-start delay.
