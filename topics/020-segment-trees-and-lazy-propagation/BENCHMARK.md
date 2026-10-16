benchmark_required: true

# Frozen comparison

Claim: select among direct Scan, rebuilding Prefix, eager leaf-update segment
tree and affine Lazy for matching wrapping range-update/range-sum semantics.
All candidate answers are admitted only after the independent eager-array oracle.
No claim that these educational versions beat production libraries.

Eight cells: tiny (n32/q512), read_only (n32768/q2048), mixed (n32768/q1024),
short (n32768/q2048/width<=8), update_heavy (n32768/q1024), hot (n32768/q2048,
starts<56,width<=8), adversarial (n32769/q1024/full-range updates), lifecycle
(n32768/q16). Fixed seed20; nominal update shares 0/50/90/67 percent, exact
counts and logical spans in retained rows. Actual ranges are nonempty; boundary
and empty-range coverage is correctness testing, not a timing claim.

One fresh process is the sample unit. Eight balanced blocks use cyclic orders
followed by their reverses. Every process independently validates every query
and final full sum, drops validation allocations, warms one full workload and
sums eight internal replay times. Internal replays are not independent samples.

Steady boundary includes dispatch, operation iteration, updates and checksum
queries. Constructor, destructor, input generation, oracle and printing are
excluded. setup_ns separately measures eight constructors. Lifecycle includes
constructor/run/drop; an excluded preliminary constructor is also performed
before each measured lifecycle replay and is recorded in setup_ns. Output is
consumed and checksummed. This is warm initialized-memory evidence, not OS-cold
startup, cold cache or a one-shot allocator experiment. No frequency/pinning
controls, PMU or causal mechanism attribution. Default rustc -O target features;
no target-cpu=native. Different toolchains preclude ISA-only comparisons.

Control: after the candidate blocks, each cell runs eight further blocks of two
processes labeled `lazy_a` and `lazy_b`. Both labels run the one benchmark
executable with the `lazy` candidate; their order alternates by block. The same
parser, checksum check and selection rule analyze them. Checksum equality is
mechanical integrity; the label ratio range is the null calibration. A control
cell that reports a winner invalidates every winner label on that host for the
campaign; the summary then records `invalid` beside the uncontrolled label.

Predeclared selection: minimum-median candidate wins only if every rival's
paired time ratio exceeds 1.05 in every one of eight blocks. Otherwise report
unresolved. Min/max and ratio ranges are descriptive, not confidence intervals.
Stop after eight blocks; do not rerun to manufacture a winner.

Inference: the effect boundary is a 5% paired ratio, one-sided (rival slower).
The protocol declares no nominal confidence level and no family-wise error
bound. Candidate order within a block is a fixed rotation rather than a random
assignment, block timings can share host-load drift, and the median-best
candidate is chosen from the same observations, so the eight ratio signs are
neither randomized nor independent and a sign-test or permutation level does
not follow from this design. Winner labels are descriptive selections under a
rule fixed before execution; the A/A control is the empirical null check. The
family is eight cells times three rivals per host, and every cell is reported
whatever its outcome. Quantified error control needs randomized within-block
assignment and a permutation analysis over that family, which is revisit scope.

Failures: each subprocess has a 600 s timeout. Every invocation, including a
nonzero exit, launch failure or timeout, is appended to `raw/attempts.jsonl`
before the campaign aborts. `raw/` is removed and recreated at campaign start,
so one directory holds one campaign. A campaign with any failed attempt is not
published; timing uses complete campaigns only.

runner.py records source/runner/test hashes, hostname, architecture,
uname/kernel, CPU model, rustc -Vv, cfg queried with -O, CPUs/affinity, flags,
the linked benchmark image SHA-256, its disassembly, unit and integration test
logs, process samples and summaries. measurements/report.py combines per-host
raw summaries into the committed summary.json and prints the README table. Raw
retained outside Git; compact receipt and summary here. No whole-process counter
is attributed to candidate-only code.
