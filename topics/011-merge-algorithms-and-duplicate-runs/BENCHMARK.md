# Fixed benchmark protocol

benchmark_required: true

Freeze library, benchmark, tests and runner hashes before transfer. Initial scratch and committed
campaigns are separate. Each declared host executes 672 candidate processes, 16 cells, 12 blocks.
Each cell uses six seeded shuffled orders followed by their reversals, balancing every pair 6/6.
Input seeds are fixed across blocks. A process is the independent sample; loop iterations are
amortization within that sample. Stop after 12 blocks regardless of the result.

Merge cells: 32 rows cold/warm; 4096 random; 1048576 interleaved/random/disjoint/equal; skewed
1024+1048576 random. Join cells: 64 cold; 8192 sparse disjoint keys; 8192 duplicate keys in 256
classes; 1024 all equal (262144 output pairs). Cascades: 16x16; 4x4096 random; 32x4096 duplicate;
one32768 plus fifteen32 random. Candidates and per-cell iteration counts are in campaign.py.

Timing starts at candidate entry and ends after output destruction. Allocation, growth, output
writes, output initialization, co-ranking, thread launch/join are included. Input generation,
sorting and independent oracle are excluded. Every candidate's exact workload output is checked
before timing. Warm cells use three warmup calls. Cold cells have zero warmups but run an untimed
correctness check first; this is not cold-cache/storage or process-startup latency. Process wall
time is retained separately and includes generation/checks/startup.

All candidates get the same first four CPUs in sched_getaffinity. This allows four spawned workers
to run concurrently; it does not isolate CPUs or control NUMA allocation. Compiler flags are
edition2024,opt-level3, default target features (no target-cpu=native). Environments retain resolved
hostnames, architecture, kernel, CPU details, available CPUs, compiler/LLVM and target cfg.

Selection: lowest geometric-mean time; unique only when every rival/fastest lower simultaneous95%
paired-log t interval exceeds1.05. Bonferroni family152 covers all pairs across both hosts:
2*(8*6+4*1+4*6). Critical t(df11)=5.1288. Report median/range and intervals, with unresolved rivals.
There is no A/A calibration, input-population interval, isolated ISA attribution or winner-driven
rerun. Large-output cells are warm allocation/teardown measurements of this owned-vector API.

The runner compiles exact Rust files with rustc and retains contracts, binary hash, linked objdump,
environment and every process attempt. Generated code supports inspection, not causal timing
attribution. Logical row-copy formulas are code-derived counts, not PMU traffic or allocation
measurements. No whole-process counters containing the oracle are attributed to a candidate.

Replay assets and raw bundles are external. See measurements/README.md and its receipt. To replay,
extract the retained frozen source archive in an empty directory on one declared host and run
`python3 campaign.py --out results`. The archive contains the exact committed runner as campaign.py,
topic/src/lib.rs, topic/benches/portfolio.rs, topic/tests/contracts.rs and source-manifest.json.
