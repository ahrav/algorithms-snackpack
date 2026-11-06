# Benchmark decision

Benchmark: true. Choose among correct complete dictionary lookups.

Freeze src/lib.rs, tests/contracts.rs, examples/edit.rs, benches/lookup.rs and
measurements/run.py at the recorded source commit. Each process independently
builds its deterministic workload, runs a separate full-matrix correctness
oracle, and asserts full result-ID equality before timing. Correctness and
compilation are outside lookup timing. The independent edit-graph oracle runs
in the contracts executable before the campaign.

Seventeen fixed cells: N=32/4096, m=8/48, random/prefix/dense layouts; k=1 for
random/prefix and k=3 for dense. Additional cells use N=4096,m=48,k=4 for
random/prefix, dense k=0, and N=32,m=64/65 random k=1. A fixed xorshift workload
uses a 16-byte alphabet. The first term equals the query. Prefix layout shares
three quarters of the query. Dense layout has zero or one substituted byte
and many duplicates. Every duplicate ID must survive. Contracts cover varying
lengths; mixed-length performance is deferred.

For each cell: four candidates, one untimed warmup process each, eight measured
process blocks. Cyclic order then reversed cyclic order balances every order
position twice per candidate. Each process makes 64 queries for N=32 or three
for N=4096. Those queries form one process sample, not independent replicates.
Before timing each process checks one lookup, so these are warm measurements.
The process pins to the lowest available CPU. No cache flushing or host
isolation is claimed. The fixed stopping rule is eight blocks, without retries
for favorable timings.

Lookup timing includes per-query preparation, row/output allocations, complete
ID production, trie result sorting and result destruction. Dictionary creation,
trie construction/destruction, independent oracle, startup and compilation are
outside lookup timing. Trie build time and whole-process elapsed time are
retained separately. Trie node/visited-edge counts expose work; linked binary
disassembly is retained. No CPU mechanism is inferred from whole-process counts.

Selection: lowest median plus >5% advantage over each other candidate in every
paired block. Otherwise report unresolved, while naming the fastest median.
Report min/max process-sample dispersion. One fixed small full-row A/A control
has eight balanced pairs, retained separately. This exploratory screen has no
population confidence or production-optimality claim. Build crossover is an
arithmetic projection from the measured build/query endpoints.
