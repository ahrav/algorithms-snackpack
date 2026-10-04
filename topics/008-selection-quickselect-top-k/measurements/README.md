# Committed-source Linux results

Source commit: `5bfb0fdd5e3dac6ceab336b7c5b9a1b38c620d33`. Later commits change evidence prose only.
The source/runner identities and verified external archives are in
[EVIDENCE_RECEIPT.json](EVIDENCE_RECEIPT.json). Raw records remain outside Git.

Both declared hosts completed tests/example and 1140 independent-process
campaigns. The initial scratch source also ran 1140 processes on each host.
Final selections below use committed-source campaigns, not pooled runs.

Arm: declared host, aarch64, 64 available CPUs, ARM r1p1, Rust 1.98.1.
x86: xxl resolved to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com,
x86_64, Xeon Platinum 8488C, 192 available CPUs, Rust 1.98.0.
Both: Linux 6.12.110-135.202.amzn2023, CPU 0, compiler-default features,
rustc opt-level=3/debuginfo=1/edition2024. Complete uname/lscpu/cfg in archives.

Geometric mean microseconds per complete API call. Selection requires
simultaneous paired log-t intervals to show >5% separation from every rival.
Unresolved entries name contenders without asserting equality.

| Workload | Arm selection and time | x86 selection and time |
|---|---|---|
| tiny | heap 0.063 | heap 0.065 |
| small_k8 | heap 1.824 | unresolved: heap, buffer; point heap 2.078 |
| large_k8 | unresolved: heap, buffer; point heap 26.679 | unresolved: heap, buffer; point heap 22.175 |
| ascending | unresolved: heap, buffer; point heap 25.891 | unresolved: buffer, heap; point buffer 20.707 |
| descending | sort 49.940 | sort 43.724 |
| duplicates | unresolved: heap, buffer; point heap 26.011 | unresolved: heap, buffer; point heap 20.803 |
| equal | unresolved: heap, buffer; point heap 25.693 | unresolved: buffer, heap; point buffer 22.135 |
| organ | unresolved: heap, buffer; point heap 25.836 | unresolved: buffer, heap; point buffer 21.122 |
| half | standard 24.994 | standard 22.765 |
| all | unresolved: three_way, sort, standard, buffer; point three_way 40.653 | unresolved: standard, sort, three_way, buffer; point standard 36.500 |
| first | unresolved: heap, buffer; point heap 29.691 | unresolved: heap, buffer; point heap 31.485 |

Workload definitions, timing boundary, controls, fixed samples and stopping
rule are in [BENCHMARK.md](../BENCHMARK.md). Every candidate oracle passed.
[RESULTS.json](RESULTS.json) retains contender ratios/intervals and min/max
dispersion. All pairwise contrasts remain in the verified raw archives.

A/A (identical standard candidate, B/A ratio and simultaneous interval):
- arm: 1.0004 [0.9889, 1.0120].
- xxl: 0.9941 [0.7931, 1.2460].

The x86 A/A interval is wide and does not establish a stable 5% noise floor.
It changed from the initial campaign; do not pool runs or prefer the initial
stronger winner. Final x86 heap/buffer cells remain unresolved.
Intervals assume independent approximately normal process-block log ratios.
First-use is not cache-flushed or process startup. Library adaptive sorting
and update counts are plausible explanations; linked disassembly and source
counters do not establish candidate-only dynamic branch/cache mechanisms.
No universal optimal candidate, throughput scaling, allocator, or input
acquisition claim is made. Comparator and element-size changes need new data.

Choose heap/buffer for tiny K using the measured host/shape guide; choose
standard for the half-output cell; retain sort for descending data. K=N
reduces four wrappers to sorting and remained unresolved.
