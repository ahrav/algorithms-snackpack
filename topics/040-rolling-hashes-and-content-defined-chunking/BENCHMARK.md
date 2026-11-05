benchmark_required: true

Claim: exploratory warm complete-call latency and exact reuse after insertion.
Population: two named Linux hosts, one native build each, fixed synthetic inputs.
Inputs: n=256/65536, target=256/1024, min=target/4, max=target*4, w=32;
random xorshift seed7, zero, period7. Fixed, direct, rolling, Gear, normalized Gear.
Direct/rolling share one boundary contract. Other policies have distinct contracts;
report throughput separately from chunk quality and metadata cost.

Boundary: includes polynomial power setup, scan, allocation, boundary output and
vector destruction. Excludes input/edit construction, oracle, printing, process
startup, compilation, digest, I/O and storage. Two warmups inside every process.
No cold-cache claim. One warmup process/candidate/cell, then eight complete blocks.
Alternate forward/reverse candidate order; deterministic assignment seed0.
Each block pairs fresh processes. Process receives the treatment; block is the
analysis unit; internal repetitions are subsamples. Shared host/cache/thermal
state can persist. Alternation counters order drift; nonlinear interference remains.

Use512iterations for small input,2large direct,16other large. Fixed stop after
8blocks. Sixteen identical-artifact rolling A/A processes share parser, runner,
affinity and analysis paths. Retain all failures and partial schedules; failures
invalidate the affected cell's selection. No timing-based exclusions or retries.

Report median and full eight-process range per candidate/cell. A clear speed
winner must beat every other candidate by >5% in all8pairedblocks. Separately
screen direct/rolling under this rule. Treat all24host-cells and allpair contrasts
as exploratory; no confidence level, familywise error control, power guarantee or
population inference is claimed. Eightblocks are a fixed bounded budget; pilot
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
