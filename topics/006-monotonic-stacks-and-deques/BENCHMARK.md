benchmark_required: true

# Frozen process experiment

Claim: compare full-call rolling-maximum costs of scan, deque, and block methods
on two named Linux hosts. No universal dispatcher or per-arrival time bound is
claimed. The stack's linear-work and burst claims use operation counts and
correctness checks; stack elapsed-time comparisons are outside this campaign.

Each process generates immutable input, validates against the direct scan,
warms three calls, then times 16 calls. Timing includes candidate preprocessing,
allocation, output consumption through `black_box`, and destruction. Input
generation, validation, warmup, instrumentation, and process startup are outside
the timing boundary. `setup_ns` measures in-process setup separately. These are
warmed full calls, not allocation-free queries or cold-start results.

Frozen cells: `random/4096/3`, `random/32768/3`, `random/4096/256`,
`random/32768/256`, `increasing/32768/256`, `decreasing/32768/256`,
`equal/32768/256`, `burst/32768/256`. Fields are shape/input length/window width.
Random generation uses xorshift with seed `0x600d20260930`, mapped to 1024 values.
Burst cells have descending runs of window length followed by `i64::MAX`.

Each cell compares deque/scan and blocks/scan in 12 independent process pairs.
The assignment is deterministic alternating AB then BA, six of each. Each
process starts with fresh allocator state, but kernel and machine state remain
shared. Pairing controls local level variation. Alternating order balances
position effects; it does not eliminate nonlinear drift or interference.
Every process uses the first CPU in its allowed affinity mask. There are 408
process samples per host: 384 comparison samples and 24 A/A samples.

Experimental and treatment units: processes. Analysis units: process pairs.
Timed repetitions are subsamples, not independent observations. Generalization
is limited to these workloads, host sessions, compilers, and full-call boundary.
No pilot variance estimate is used. The fixed count is a small diagnostic
campaign, with potentially wide intervals. No optional stopping or expansion
after inspecting results is permitted. No output is silently excluded.

Analysis: use the mean paired log(candidate/scan) ratio, exponentiated, and a
two-sided Student-t interval with 11 degrees of freedom. The model assumes
independent approximately normal pair log ratios. Familywise nominal confidence
is 95% using Bonferroni over 32 contrasts (16 cells/contrasts per host times two
hosts). Declare a scoped 5% separation only if the entire interval lies below
`1/1.05` or above `1.05`. Median and min/max process time accompany intervals.
Finite samples and shared hosts limit those model-based claims.

A/A uses the same deque binary with distinct `aa-a` and `aa-b` labels, random
32768/256, the same 12-pair assignment and parser. Mechanical identity, complete
pairs, and positive times are mandatory. Its unadjusted 95% interval is a
diagnostic, not a proof of null calibration or equivalence.

Any failed correctness workload, invalid source identity, missing process,
timeout, parser failure, or nonpositive elapsed time fails the campaign. Retain
its partial receipt and preserve curriculum state. No candidate reliability
difference is estimated from a failed campaign.

Build flags: edition 2024, optimization level 3, debug information level 1,
default target CPU/features, warnings denied, unsafe code forbidden. The runner
records compiler, host, kernel, CPU, available CPUs, affinity, hashes, tests,
doctests, library assembly, and linked-image symbols. It attempts a `perf stat`
profile separately. Whole-process counters include oracle and instrumentation
work and cannot establish a candidate-only mechanism. Unavailable counters
support no branch/cache mechanism.
Counted implementations run outside timing. No runtime allocation counter is
claimed; buffer counts come from code inspection.

Reproduction: create a path-limited `git archive` of the root Cargo files and
this topic, record SHA-256, transfer it and `scripts/run_linux.py`, then run:

```bash
python3 run_linux.py source.tar ARCHIVE_SHA256 SOURCE_COMMIT EXPECTED_ARCH
```

The runner fails closed on archive paths and architecture. Archive, runner,
source, binary, machine and result identities are retained externally; compact
results and a verified receipt are committed after collection.
