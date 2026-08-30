# Prefix sums and scans

A scan materializes the reduction of every ordered input prefix. This crate
uses wrapping `i64` addition so every candidate has one exact, associative
contract in debug and release builds.

## Contract

Let `a +% b` mean `a.wrapping_add(b)`.

For input `x[0..n]`, inclusive output has length `n` and satisfies:

```text
inclusive[0] = x[0]
inclusive[i] = inclusive[i - 1] +% x[i]
```

Seeded exclusive output also has length `n`:

```text
exclusive[0] = seed
exclusive[i] = exclusive[i - 1] +% x[i - 1]
```

Empty input returns an empty vector. Equality means the same length, order,
multiplicity, and `i64` bit pattern at every position.

The running example is:

```text
input:              [3, 1, 7, 0, 4, 1, 6, 3]
inclusive:          [3, 4, 11, 11, 15, 16, 22, 25]
exclusive, seed 0:  [0, 3, 4, 11, 11, 15, 16, 22]
```

`blocked_inclusive` rejects a zero block size with
`ScanError::ZeroBlockSize`. `parallel_inclusive` rejects zero workers with
`ScanError::ZeroWorkers`. Both functions borrow the input immutably.

The crate does not promise generic operators, floating-point reproducibility,
checked-overflow position reporting, in-place execution, partial overlap,
segmented scan, a persistent thread pool, or a universal dispatch threshold.

## Mental model

The scalar loop carries one prefix forward. A parallel scan splits the input,
computes local prefixes, scans the chunk totals, then adds each chunk's carry.
Associativity permits different parentheses. Operand order still matters, so
commutativity is not required.

For four blocks of length two, the running example becomes:

```text
local prefixes:  [3, 4] [7, 7] [4, 5] [6, 9]
chunk totals:    [4, 7, 5, 9]
chunk offsets:   [0, 4, 11, 16]
global prefixes: [3, 4] [11, 11] [15, 16] [22, 25]
```

The block invariant is:

```text
global prefix = reduction of earlier chunk totals +% local prefix
```

## Implementations

- `reference_inclusive` and `reference_exclusive` recompute each prefix from
  its start. They are deliberately quadratic and share no correctness-critical
  helper with optimized code.
- `linear_inclusive` and `linear_exclusive` carry one accumulator in one pass.
- `blocked_inclusive` performs local scans, scans chunk totals, then applies
  offsets on one thread.
- `parallel_inclusive` performs local scans and offset repair with scoped OS
  threads. The caller scans chunk totals in order. Safe owned chunk vectors
  avoid shared mutation, then a final copy assembles one output vector.

The scoped-thread candidate is intentionally self-contained. A production
thread pool changes startup cost and may move the crossover.

## Derived cost model

Let `n > 0`. The blocked scan uses block size `B`, chunk count
`q = ceil(n / B)`, and first chunk size `b0 = min(B, n)`. The parallel scan
chunks by effective worker count `k = min(workers, n)` with first chunk size
`b0 = ceil(n / k)`; each row's `b0` is that candidate's own first chunk size.

| Candidate | Wrapping additions | Logical main-array traffic | Extra state |
|---|---:|---|---|
| Quadratic reference | `n(n + 1) / 2` | Quadratic input reads, `n` output writes | one output allocation |
| Scalar inclusive | `n - 1` | `n` input reads, `n` output writes | one accumulator and output |
| Blocked inclusive | `n - 1` if `q = 1`; otherwise `2n - b0 - 2` | `n` input reads; `2n - b0` output writes; `n - b0` output reads | `q` totals and output |
| Parallel inclusive | scalar if one effective worker; otherwise `2n - b0 - 2` | blocked passes plus `n` reads and `n` writes to assemble the final output | per-chunk outputs, offsets, final output, and two thread phases |

For eight elements, the reference performs `36` additions and the scalar scan
performs `7`. A blocked scan with `B = 2` performs four local-prefix additions,
two chunk-total additions, and six offset additions, for `12` total.

An out-of-place `i64` scan must logically read `8n` input bytes and write `8n`
output bytes. The lower bound is `16n` bytes before cache lines, write
allocation, allocator metadata, or coherence traffic. The scalar candidate
also meets the binary-combine work lower bound of `n - 1`.

The kernels perform no key comparisons and no value-dependent branch. Loop
control still branches. With `k > 1` effective workers, the parallel candidate
spawns `k` local-scan threads and `k - 1` offset-repair threads, for `2k - 1`
thread spawns. It joins each phase before continuing. These costs are derived
from the source. Assembly and profiles are needed before assigning a hardware
mechanism.

## Correctness invariants

The scalar inclusive invariant says that before processing `x[i]`, the
accumulator equals the prefix through `x[i - 1]`. One wrapping addition makes
it the prefix through `x[i]`.

The scalar exclusive invariant says that the current accumulator is the next
output. The implementation writes it before adding the corresponding input.

The blocked candidate relies on three facts:

1. Every local value is the prefix within its chunk.
2. Every chunk offset is the exclusive scan of earlier chunk totals.
3. Offset plus local prefix preserves the global input order.

The parallel candidate gives each worker disjoint output positions. The first
join finishes local scans before offset construction. The second join finishes
offset application before return.

See [TEST_STRATEGY.md](TEST_STRATEGY.md) for the independent oracle, bounded
domains, mutation backstops, and invariant-test review.

## Selection guide

Use the quadratic functions as oracles.

Use the scalar functions for small inputs, single-core ownership, or calls
that include thread startup. They do the least arithmetic and touch the output
once.

Use the blocked form to study chunking cost or when a caller already has a
suitable execution plan. The blocked form rereads and rewrites most output
elements.

Use the scoped-thread form only when the measured input size repays thread
creation, joins, and the offset pass. Keep a scalar and parallel portfolio if
the winner changes by size or worker count.

Use an established GPU primitive when data is already resident and the
operator, overlap, temporary-storage, and determinism contracts fit. Kernel
time and end-to-end host time are different claims.

## Benchmark

Runtime, memory traffic, worker overhead, and crossover are central, so
[BENCHMARK.md](BENCHMARK.md) declares `benchmark_required: true`.

Compile the harness:

```bash
cargo bench -p prefix-sums-and-scans --bench prefix_sums --no-run
```

Run one machine-readable position after building the release harness:

```bash
cargo bench -p prefix-sums-and-scans --bench prefix_sums -- \
  --algorithm linear --label A --attempt-id focused-linear \
  --contrast-id focused-linear --phase focused --n 262144 \
  --pattern mixed --seed 2026082901 --block-size 1 --workers 1 \
  --warmups 1 --samples 3
```

Run the retained outer experiment with the commands documented in
[measurements/README.md](measurements/README.md). Inner benchmark iterations
are subsamples. Complete four-position blocks are the analysis units.

The completed Apple M1 Pro run used linked-image SHA-256
`0144d4ef720e2f721643a00f3b5d4775b65d753c34781efe6c1e7b714d9dd4a9`.
Eight-worker parallel scan at `n = 4,194,304` had `B / linear = 0.614` with a
simultaneous interval of `[0.483, 0.781]`, which cleared the predeclared 5%
boundary. Linear scan cleared that boundary against the reference at all three
small sizes, blocked scan at `n = 4,096` and `n = 4,194,304`, and no other
declared candidate cell. The A/A diagnostic interval was wide, so these are
one-build, one-host, one-window results rather than a universal threshold. See
[the retained result and raw-evidence boundary](measurements/README.md).

## Limits and production adaptations

- Floating-point addition is not associative. A parallel tree may differ from
  a scalar left fold and may vary across schedules.
- Checked overflow can expose a different first failure after reassociation.
- Saturating signed addition is not a safe generic scan operator without a
  separate associativity proof.
- Tree padding needs a real identity.
- Exact in-place execution needs a read-before-overwrite contract. This crate
  allocates a separate output.
- Segmented scan resets at explicit boundaries and needs its own representation
  and tests.
- GPU decoupled look-back needs ordered publication and forward-progress
  assumptions that this CPU crate does not model.

## Evidence boundary

Derived evidence includes operation counts, logical byte counts, and the
correctness invariants above.

Observed source evidence includes current API semantics and published
algorithm structures. It does not establish local performance.

Measured evidence, exact artifact identity, raw attempts, A/A results, and
profiles belong under `measurements/` after execution.

Any memory-bandwidth, vectorization, cache, or scheduler explanation remains
an inference until the exact linked artifact supplies mechanism evidence.

## Primary sources

- Guy E. Blelloch, [Prefix Sums and Their Applications](https://www.cs.cmu.edu/~guyb/papers/Ble93.pdf).
- Richard E. Ladner and Michael J. Fischer, [Parallel Prefix Computation](https://doi.org/10.1145/322217.322232).
- W. Daniel Hillis and Guy L. Steele Jr., [Data Parallel Algorithms](https://doi.org/10.1145/7902.7903).
- Duane Merrill and Michael Garland, [Single-pass Parallel Prefix Scan with Decoupled Look-back](https://research.nvidia.com/sites/default/files/pubs/2016-03_Single-pass-Parallel-Prefix/nvr-2016-002.pdf).
- Current C++ draft, [generalized numeric operations](https://eel.is/c++draft/numeric.ops).
- OpenMP 5.2, [scan directive](https://www.openmp.org/spec-html/5.2/openmpse28.html).
- oneTBB, [`parallel_scan`](https://uxlfoundation.github.io/oneTBB/main/specification/source/algorithms/functions/parallel_scan_func.html).
- NVIDIA CCCL, [`cub::DeviceScan`](https://nvidia.github.io/cccl/unstable/cub/api/structcub_1_1DeviceScan.html).
- Rust, [`i64::wrapping_add`](https://doc.rust-lang.org/std/primitive.i64.html#method.wrapping_add) and [overflow rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).
