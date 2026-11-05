benchmark_required: true

Claim: exploratory warm complete-call latency and exact reuse after insertion.
Population: two named Linux hosts, one native build each, fixed synthetic inputs.
Inputs: n=256/65536, target=256/1024, min=target/4, max=target*4, w=32;
shapes `random` (xorshift seed 7), `zero`, `periodic` (i mod 7). Candidates
`fixed`, `direct`, `rolling`, `gear`, `normalized` (normalized Gear).
Direct/rolling share one boundary contract. Other policies have distinct contracts;
report latency separately from chunk quality and metadata cost.

Boundary: includes polynomial power setup, scan, allocation, boundary output and
vector destruction. Excludes input/edit construction, oracle, printing, process
startup, compilation, digest, I/O and storage. Every process computes the original
and edited boundary vectors once for the oracle, then runs two untimed warmup
calls, then the timed batch; four calls precede `Instant::now()`. No cold-cache
claim. One warmup process/candidate/cell, then eight complete blocks.
Candidate order alternates forward/reverse by block, a fixed deterministic
schedule with no random assignment and no seed. With five candidates, reversal
leaves `rolling` at the middle position in every block, so position drift is
counterbalanced for the other four candidates only.
Each block pairs fresh processes. Process receives the treatment; block is the
analysis unit; internal repetitions are subsamples. Shared host/cache/thermal
state can persist. Alternation counters order drift; nonlinear interference remains.

Use 512 iterations for small input, 2 for large direct, 16 for other large cells.
Fixed stop after 8 blocks. Sixteen identical-artifact rolling A/A processes share
parser, runner, affinity and record paths. Retain all failures, timeouts and
partial schedules in `results.jsonl`; `complete.json` counts them, and a failure
invalidates the affected cell's selection. No timing-based exclusions or retries.

Report median and full eight-process range per candidate/cell. A clear speed
winner must beat every other candidate by >5% in all 8 paired blocks. Separately
screen direct/rolling under this rule. Treat all 24 host-cells and all pair contrasts
as exploratory; no confidence level, familywise error control, power guarantee or
population inference is claimed. Eight blocks are a fixed bounded budget; pilot
variance and minimum detectable effect calibration are unavailable. No sequential
peeking or extra runs for a winner. A/A measures harness spread, not statistical
null calibration. Frequency and competing-host activity remain uncontrolled.

Independent Python oracle uses explicit positional polynomial sums and closed-form
Gear weighted sums; it verifies every original and edited boundary vector before
accepting the process. Exact bytes establish chunk reuse. Rust tests cover bounds,
small/empty inputs, reset parity, rejected settings, overlaps and a true collision.
Retain source hashes, runner, schedule, binaries, complete disassembly, environments,
process records and hashes outside Git. Assembly supports code-path inspection;
whole-process counters containing oracles cannot establish candidate-only causes.
