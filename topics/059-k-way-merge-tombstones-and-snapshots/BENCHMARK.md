benchmark_required: true

The selection target is median time per complete snapshot scan of presorted
resident inputs. Three streaming/materialized selectors share visibility logic.
Chronological replay is an independent algorithm and fourth timing arm.

| Case | Runs | Points | Ranges | Layout | Range width at most |
| --- | ---: | ---: | ---: | --- | ---: |
| tiny | 2 | 32 | 2 | mixed | 3 |
| low-fan-in | 2 | 16,384 | 64 | mixed | 8 |
| mixed-8 | 8 | 16,384 | 64 | mixed | 8 |
| mixed-32 | 32 | 16,384 | 64 | mixed | 8 |
| overlap-32 | 32 | 16,384 | 2,048 | mixed, 1,024 keys | 768 |
| skew-64 | 64 | 16,384 | 64 | 90% in one run | 8 |
| disjoint-32 | 32 | 16,384 | 64 | disjoint key ranges by run | 8 |

The exact fixed fixture seed is 5917. Mixed large cases use 8,192 possible
keys except overlap. Point deletes occur when the generated payload is divisible
by 11. The snapshot is roughly three-quarters through operation sequences.
Judge every implementation by exact output before timing it.

Use eight complete blocks per case and host. The four-arm Williams order
and its reversed block sequence balance positions. Each arm gets a new
process per block, three untimed scans inside that process, and one elapsed
batch of 3,000 tiny or 32 large scans. The block is the paired analysis unit.
Input generation and verification happen before timing. Selector allocation,
range sorting, traversal, result allocation, and destruction are timed.
Process wall time includes spawn, input generation and warmup. Use one
sequential campaign per host and a fixed available CPU. Record every attempt.
Use a separate initial process for each arm and two fresh identically built
heap processes per A/A control block, alternating labels.

Use the per-process batch mean as a subsample summary. Report median and
interquartile range across blocks. Select an arm only if the median paired
ratio is at most 0.95 and the arm wins at least six of eight blocks against
every rival. The schedule stops after eight blocks. Use no interim looks,
formal confidence claim, or population inference. This is a descriptive
engineering criterion. Report wide A/A dispersion with the decision.
Point comparison counts come from a separate instrumented run. Elapsed time
includes range work, output allocation and teardown, so it does not isolate
comparisons. Replay has no point-head comparisons. The run covers warm
in-memory forward scans on two kernels/builds. Cold I/O and compaction are
out of scope.
