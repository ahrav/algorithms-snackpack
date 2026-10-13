# Evidence boundary

Initial scratch campaigns:400 fresh candidate processes on each declared Linux
host. Every library/public contract and process oracle passed. Exact committed
replay also completed400oracle-passing processes/host; results.json is authoritative. Initial source/rows remain separate.

Run `python3 measurements/runner.py --out /tmp/topic017-replay` from this topic.
The output path must not exist. This compiles from the six frozen files,tests and
runs the demo,then compiles a native optimized benchmark plus assembly. Raw remains
external. Each output has manifest,attempt log,per-process stdout,rows and summary.

Arm hostname dev-dsk-ahrav-2b-7dc7bd93.us-west-2.amazon.com,aarch64,64 available CPUs.
x86 alias xxl resolved to dev-dsk-ahrav-2c-32182091.us-west-2.amazon.com,x86_64,192 CPUs.
Kernel/toolchain/CPU/native features are recorded independently for each campaign.
Flags:`--edition 2024 -C opt-level=3 -C target-cpu=native`. Host scheduler is unpinned.
Initial Arm Rust1.98.1; x86 Rust1.99.0. Architecture alone cannot explain differences.

No false-negative result is inferred from timing:all member/oracle checks precede
it. Standard/blocked layouts share10 requested bits/key,rounded to64-bit words.
Cuckoo allocates power-of-two four-slot buckets targeting≤50% load. Its nonzero
12-bit fingerprints use16-bit storage. Quotient uses a power-of-two directory
with usize offsets and16-bit remainders; it is not compact QF. Payload bytes exclude
vector headers/allocator metadata/transient paths and the shared8n-byte authority.

Initial selections on both hosts:direct for tiny_miss,mid_hit90,large_hit90,build_mid;
cuckoo for mid_miss,large_miss,first_batch;quotient for hot_miss. No universal winner.
Actual false-positive counts are fixed-key/workload observations,not independently
sampled probabilities. first_batch has no warmup but preflight warms state,no cold
claim. Query timing excludes build/allocation/teardown;build includes temporary
allocation/copy/drop,excludes source sorting/oracle. Assembly retained,no PMU or
candidate-only cache/branch mechanism claim. Timing ranges are descriptive.

Revisit:real storage costs,matched bytes/FPR,other seeds/layouts,compact QF/Ribbon,
peak build memory,mutation/adversarial keys,affinity,cold-start and tail distributions.
