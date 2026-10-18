# Predeclared experiment

benchmark_required: true

Decision: compare three correct sequential hybrid ordered indexes, with one
simple sorted-vector baseline. Fixed data uses i64 scores/u64 members and
(score,member) order. Rank/select and response materialization are part of the
consumer. The oracle uses unsorted members and independently sorts answers.
Reject any differing response/final state before timing that process.

Eight cells: tiny n16; lookup/rank/update/ties/range n4096; ascending and
descending incremental builds n4096. Each steady trace has12000 operations.
Lookup is90%get/10%put; rank is40%rank/40%select/10%put/10%remove;
update is70%put/10%remove/10%get/10%rank; range is90%range(limit20)/10%put;
tiny/ties cycle put/remove/get/rank/select evenly. Equal-score cell starts with
four scores; descending build forces repeated front insertion for the vector.
Missing IDs and invalid selected ranks occur in the frozen generator. These
are fixed traces, not a distributional population sample. Construction is
incremental for all candidates; a bulk-sort vector is an unmeasured alternative.

Eight process blocks per host. Candidate order rotates across paired blocks
and reverses in the second block; cells use a fixed shuffled order. Each cell
also has two independent tree A/A process labels, reversed in paired blocks.
There are320 fresh candidate processes/host/campaign. Each process has one
warmup and three timed subsamples, whose median is one process observation.
Inner operations/subsamples are not independent statistical samples.

Steady boundary: construct before timing; operation dispatch, hash authority,
mutation, allocation of response Vecs, range output and checksum fold included;
index destruction after timing. Build boundary: construct empty index, insert
4096 records, materialize first20, fold and destroy index. Generator, oracle,
compilation, process startup and printing excluded from ns; process wall time
is retained separately. Drop validation state before warmup. HashMap's standard
per-instance random seeds are not fixed; skip heights/workload use nonzero
xorshift64 state22018, with explicit modulo range mapping in the source.

Flags: --edition=2024 -C opt-level=3 -C target-cpu=native. Record rustc-Vv,
target cfg/features, hostname, uname/kernel, lscpu, available CPUs and affinity.
Scheduling is unpinned. Different installed toolchains and hosts confound any
ISA-only comparison. No PMU/cold-cache/tail/RSS/candidate-only mechanism claim.
Timing includes allocation and virtual dispatch; it does not isolate search.

Stopping/selection fixed before initial runs: eight blocks, no winner-seeking
reruns. Lowest process median is resolved only if every paired rival/best ratio
exceeds1.03 in all eight blocks. Otherwise list unresolved rivals. Report
min/median/max and paired ratios. These are descriptive observed ranges, not
confidence intervals or universal crossover claims. A/A is diagnostic, not a
post-hoc filter. Initial and exact committed campaigns are retained separately.
