# Frozen comparison

benchmark_required: true

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

Predeclared selection: minimum-median candidate wins only if every rival's
paired time ratio exceeds 1.05 in every one of eight blocks. Otherwise report
unresolved. Min/max and ratio ranges are descriptive, not confidence intervals.
Stop after eight blocks; do not rerun to manufacture a winner.

runner.py records source/runner hashes, hostname, architecture, uname/kernel,
CPU model, rustc -Vv, default cfg, CPUs/affinity, flags, binary disassembly,
process samples and summaries. Raw retained outside Git; compact receipt and
summary here. No whole-process counter is attributed to candidate-only code.
