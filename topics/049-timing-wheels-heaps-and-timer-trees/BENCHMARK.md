# Benchmark decision

benchmark = true

Claim: select among four correct finite replay implementations for each declared
workload/host using per-process nanoseconds per complete replay. Timing includes
validation, allocations, operations, output sorting/construction, final-state
construction and teardown. Workload generation, untimed first replay, printing,
oracle, process startup and compilation are outside timing. No callback executes.

Workloads: U=16/4096; shape=near/far/burst; cancellation=0/90 percent
(ID modulo 10 determines the subset, so tiny counts are rounded).
All IDs arm once, every fourth ID rearms once. Near/far deadlines use
1+(73*ID+offset) modulo horizon; replacement offset=17. Horizon=256/65536.
Near/far have 256 equally spaced polls; burst puts all deadlines at 65536 with
one poll. Far/burst stress tick scanning; rearming stresses stale identity.

Treatment unit: one fresh process running a single candidate, 16 timed replays.
Analysis unit: complete four-candidate block within one cell and host. Inner
replays are subsamples. Six fixed blocks alternate forward/reverse order, three
of each. One warmup process per cell/candidate precedes them. A/A runs six paired
heap/heap processes under A/B labels, reversing order in alternate blocks.
Each process also performs one untimed replay to warm code and emit full state.
348 processes/host = 48 warmups + 288 measured + 12 A/A.

Criterion fixed before collection: a winner must be over 5 percent faster than
each rival in every paired block, with every full-output oracle passing. Report
unresolved cells, medians and min/max process ranges. This is an exploratory
descriptive screen with fixed sample count, no power or confidence claim and no
population-wide inference. Stop after six blocks. Retain failures and partial
attempts; a semantic runner/source repair needs a new frozen campaign.

Controls: fixed workload/source/runner, CPU0 affinity, native optimization flags,
same binary for all candidates on each host, separate full-output Python oracle,
contract tests and A/A control. Hosts remain shared; caches are warm, input state
starts empty, toolchains differ, host order is not randomized. No cold-cache,
service latency, peak-RSS or hardware-causal claim. Counters report inspections,
ticks and peak physical records. Retained assembly supports code inspection only.

Run from the topic directory with `bash benches/native.sh` on each Linux host.
The script compiles exact local source, records environment and hashes, runs
contracts, emits assembly and executes `benches/run.py`. The latter writes the
schedule before collection and retains stdout/stderr for every process.
