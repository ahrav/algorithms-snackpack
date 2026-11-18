# Benchmark decision

Benchmark required. Compare the dispatch work and modeled routing outcomes of six
correct policies on shared event traces. The dispatch result includes the policy
choice, queue-tail update, response arithmetic, and output store. It excludes
input generation, setup, resets, printing, teardown, and the oracle. Record the
process's file parse and allocation time separately as setup.

The pinned inputs use an explicit seed and eight cells. These include 4 backends
with 16 requests, six 32-backend workloads with 4,096 requests each, and a
256-backend workload with 8,192 requests. At 32 backends, vary service duration,
arrival batches, key frequency, demand imbalance between zones, and network
penalty. The expensive-remote cell reuses the locality-skew trace with a
different penalty. The hot cell gives the same key to four of every five arrivals.

Each cell uses a warmup block then six cyclically rotated measurement blocks.
One process supplies one timing sample. Each process replays from empty state
256 times for tiny and eight times otherwise. Record elapsed time per replay,
the median and range, and one regular-cell round-robin A/A control with six
fresh pairs. Use one allowed CPU per Linux host. A candidate wins dispatch time
only if its measured time is more than 5% below every rival in all six paired
blocks. Keep unresolved results. The stopping rule is the fixed schedule.

The independent oracle checks every dispatched ID, wait, and response. Count
logical decision probes and record response P95 and remote assignments. The
quality metrics are deterministic outputs of the simulator in ticks. This
experiment makes no microarchitectural attribution from wall time. Native
assembly is retained for reproducibility. It describes a warm dispatch loop on
synthetic inputs. Production feedback, capacity and network effects require a
separate workload and contract.
