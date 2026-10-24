# Benchmark decision

Timing is required for choosing among these safe locked scheduler models.
The baseline is one global FIFO. Each workload compares it with per-worker
owner-bottom, thief-top deques using the stated initial placement.

| Workload | Workers | Tasks | Leaf iterations | Placement |
| --- | ---: | ---: | ---: | --- |
| tiny_balanced | 2 | 256 | 0 | Round-robin |
| tiny_skewed | 2 | 256 | 0 | All on worker zero |
| medium_balanced | 4 | 16384 | 8 | Round-robin |
| medium_skewed | 4 | 16384 | 8 | All on worker zero |
| coarse_skewed | 4 | 4096 | 512 | All on worker zero |

Build once, then run fresh processes:

```bash
cargo build --release -p topic028-work-stealing-deques --example compare
python3 topics/028-work-stealing-deques/scripts/run_campaign.py \
  --binary target/release/examples/compare --repetitions 8 --out campaign.json
```

The protocol is frozen before either Linux campaign: one discarded warmup
process per candidate and workload, then eight paired repetitions with strict
alternating order. This gives 80 measured processes and 10 warmup processes per
host. Report median, minimum and maximum plus the eight paired steal/global
ratios. The lowest median is a descriptive ranking. Select a candidate only
when every paired ratio lies beyond the same 3 percent boundary: below 1/1.03
for stealing, above 1.03 for global FIFO. Otherwise record unresolved. This
descriptive rule supplies no confidence interval or significance claim.
Stop after eight repetitions and retain every valid observation.

The binary timer starts after argument parsing and sequential oracle computation.
It includes queue allocation and population, worker startup, scheduling, task
work, shared accounting, joins and the final equality check. Compilation, process
startup, output and JSON writing remain outside that timer. Fresh threads and
queues are created in every measured invocation. Warmups warm executable pages
and host state. Scheduling is unpinned, and caches are uncontrolled.

The source manifest freezes all topic inputs and the original workspace inputs.
Each host verifies SHA-256 before building the unchanged topic in a one-member
replay workspace with the same edition, minimum Rust version and lints. Record
the replay wrapper, host, kernel, CPU, available CPUs, compiler, target features,
release flags, binary hash, tests and raw outputs. Run the exact committed
source on both declared hosts. Toolchain and host differences prevent an
ISA-only interpretation.

The experiment covers complete invocations of a mutex-based teaching model
with preloaded independent tasks and cyclic victim scans. Selection applies
within those limits. Dynamic spawning, parking, cancellation, lock-free
throughput, allocator bytes, hardware counters and production tail latency
require further experiments. Shared counters and setup contribute to the
measured costs, so elapsed time alone cannot attribute a mechanism.

The original seven-repetition macOS smoke campaign is retained separately.
Its 4:3 order split and broad ranges support descriptive local observations.
