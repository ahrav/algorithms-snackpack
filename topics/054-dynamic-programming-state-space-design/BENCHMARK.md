# Benchmark decision

Benchmark required. Compare the cost of one exact score solve for bounded 0/1
knapsack across exhaustive enumeration, full capacity table, rolling capacity
row, value-indexed row and a sparse frontier.

The fixtures cover a three-item request, tight and medium budgets, wide capacity
with small values, wide capacity with coarse weights, and an adversarial dense
frontier with powers-of-two weights and values. All inputs are fixed. The runner
independently enumerates raw subsets and checks each fresh process against the
same full vector and score. Logical counts report score slots, take updates and
frontier inputs. They describe the algorithms and do not measure hardware.

The timed loop begins after input creation and ends before printing. Each call
allocates, initializes, computes and frees its own solver state. The loop also
accumulates one score per invocation. Process launch and output are recorded
separately as caller-observed process time. Each fixture has a fixed repetition
count shared by candidates. Repetitions amortize timer overhead.

One sample is a fresh process. Run one untimed-for-selection process per arm.
Collect ten paired blocks using every cyclic rotation of the five candidates,
then the reverse of every rotation. Append an identical rolling invocation to
each block as an A/A noise screen. Pin the runner to one available logical CPU.
Use a native release build on each host. Select a median leader only if its time
is at most 95% of each rival's time in every paired block. Report median and
full sample range, A/A ratios, and any unresolved result. Stop after ten blocks.
The first-in-process call is mixed with repeated calls, so this design answers
neither isolated cold-call latency nor steady allocator reuse across requests.

Record source, runner and binary hashes, environment, assembly for each host,
and raw ordered samples. Use assembly to inspect generated loops and report
elapsed differences without assigning hardware causes. Find build and replay
commands in the [measurement record](measurements/README.md).
