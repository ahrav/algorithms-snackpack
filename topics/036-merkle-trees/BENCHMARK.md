# Benchmark decision: true

Compare the same final-root contract using Full, Eager and Batch. Fix the source,
runner and schedule before native execution. Each host runs nine workloads:
4/64/1024/8192 leaves, 1/32/4096 bytes, one/8/64/all writes, cluster/scatter,
1024 repeated-index writes and byte-equal no-ops. See run.py for exact inputs.

Per cell: one warmup process per candidate followed by six fresh-process blocks,
each of the six candidate permutations once. Each candidate occupies each
position twice. Repetitions within a process are one sample. Stop after this
schedule. Report medians and observed min/max; these are descriptive finite-run
results. Select the lowest median only when every paired block beats both other
candidates by 5%; otherwise report unresolved. Name fastest medians separately.

Update timing includes input validation, record clone/equality, allocating
portable SHA-256, batch sort/dedup, scratch drop and root extraction. Initial
construction has its own timer. Lifecycle timing includes clone of initial
records, construction, one batch update and tree drop. Workload construction,
process startup, correctness oracle and output are outside both timers. Final
tree teardown is outside update timing. Complete wall time is retained as a
separate diagnostic. Full rebuild reuses its allocated digest array.

All candidates use the same final-root contract. Eager additionally constructs
intermediate roots. Batch hashes duplicate writes before deduplicating ancestors;
last-write coalescing is a future alternative. Cached no-op detection trusts
retained exact bytes. Changed-file discovery remains an application obligation.

Compile with rustc edition 2024, opt-level=3, target-cpu=native. Record toolchain,
target cfg, available CPUs, hostname, uname and /proc/cpuinfo. Keep binaries and
raw logs externally. Compare each result with an independent recursive Python
hashlib root; reject disagreement before selection. Verify SHA256 lengths
0..129,255,256,511,512,1000,4096 against hashlib. Run bounded update/proof tests.
Inspect linked disassembly as static evidence. This run makes no dynamic CPU
cause claim, cold-cache claim, allocation-count claim or population interval.
Host load/frequency and CPU affinity remain uncontrolled and recorded as limits.
