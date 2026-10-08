# Sorting algorithm portfolios

Choose the required order and stability before choosing a sorting path. This lab sorts
fixed-width records by an unsigned 64-bit key. The high 32 bits are a leading prefix;
the low 32 bits are a suffix. The `id` field is payload, not a tie breaker.

## Running example

Input: `(1,7,#0), (1,2,#1), (1,2,#2), (2,8,#3), (2,1,#4), (2,4,#5)`.
After sorting the first prefix group: `(1,2,#1), (1,2,#2), (1,7,#0)` followed by the unchanged second group.
Final stable output: `(1,2,#1), (1,2,#2), (1,7,#0), (2,1,#4), (2,4,#5), (2,8,#3)`.

The prefix sequence is nondecreasing. Each equal-prefix group is complete and contiguous.
Sort each group stably and concatenate. Arbitrary chunks need a full-comparator boundary
proof or a merge: sorting `[9,1]` and `[8,2]` independently produces `[1,9,2,8]`.

The rendered before/during/after visual is retained externally with the lesson and receipts.

## Contracts and candidates

All candidates preserve complete-record multiplicity and nondecreasing keys. Stable candidates
preserve arrival order among equal keys, even when payload IDs are shuffled or repeated.
`groups` validates prefix order before mutation and returns the first offending boundary.
Unknown candidate names panic. This is a teaching API with fixed unsigned keys, infallible
comparison and ordinary Rust allocation behavior, not a generic production sorting interface.

| Candidate | Mechanism | Stability | Main limit |
|---|---|---|---|
| `unstable` | standard `sort_unstable_by_key` | unconstrained | equal-key order can change |
| `stable` | standard `sort_by_key` | stable | scratch policy depends on implementation and input |
| `quick` | median pivot, three-way partition, depth budget, heap fallback | unconstrained | teaching partition has large constants |
| `heap` | maximum heap and repeated extraction | unconstrained | noncontiguous accesses and swaps |
| `merge` | detect natural runs, normalize, pairwise merge | stable | run metadata, scratch and copy-back work |
| `radix` | eight stable least-significant-byte passes | stable | fixed encoding, bucket overhead, scratch/movement |
| `groups` | validate prefix order, stable sort each equality group | stable | skew and validation/batch overhead |
| `dispatch` | insertion through 32, groups if valid, stable fallback | stable | heuristic threshold and extra scan |

Rust 1.81 changed the standard core sorts to driftsort and ipnsort. In tested Rust 1.98,
these are default implementation details; size-optimized configurations can use other paths.
The teaching merge and quick candidates do not implement driftsort or ipnsort.

A strictly decreasing run can be reversed without changing equal-key order. Blind reversal
of a nonincreasing run can reorder equals: `3,2a,2b,1` becomes `1,2b,2a,3`.
An algorithm supporting nonincreasing runs needs additional equality repair.

## Derived cost model

Let `n` be records, `r` natural runs, `g` prefix groups, `m_i` group sizes,
`w` key bits, and `b` digit bits. These are work models, not predicted nanoseconds.

- Balanced comparison work: `O(n log2 n)`. Six records give scale `6 log2 6 = 15.5`.
- Naive quicksort with one-record progress: `T(n)=T(n-1)+Theta(n)`, quadratic.
  Guarded partition depth is `2 floor(log2 n)`; fallback retains `O(n log n)` worst-case work.
- Natural merge: `O(n+n log2 r)`. This example has three detected runs,
  giving `6+6 log2 3 = 15.5` scale units. One run needs only detection/normalization.
- Groups: `O(n+sum m_i log2 m_i)`. Here two groups of three give `15.5` units.
  4,096 records in groups of 16 give `20,480`, versus `49,152` global comparison units.
  One giant group restores almost the global-sort work.
- Radix: `p=ceil(w/b)` passes and `O(p(n+2^b))` work. Here `p=8`, so the six-record
  scale is `8(6+256)=2,096`. Scratch has `n` record slots and a 256-entry stack histogram.
  Eight scatter and eight copy-back passes write `16n` records, plus initial scratch copy.

Heap uses no record scratch; quick uses bounded recursion. Merge allocates run metadata,
`n` scratch records and next-level metadata. Group sorting's standard scratch is bounded by
the largest group in this sequential implementation. Dispatcher can add a rejected prefix scan.
Count caller input, any cloned work input and sorting scratch separately.

## Run

From the repository root:

```bash
cargo test -p sorting_lab --all-targets
cargo run --release -p sorting_lab --example portfolio
cargo bench -p sorting_lab --bench portfolio -- stable 4096 random 16 9000
```

On Linux, the exact frozen process campaign can run from the repository root:

```bash
python3 topics/010-sorting-algorithm-portfolios/scripts/run_linux.py \
  --topic topics/010-sorting-algorithm-portfolios --out /tmp/sorting-results-unique
```

Use a new output directory. The script verifies a transferred manifest when supplied and
always records current Rust source hashes. It compiles with `rustc --edition=2024 -C opt-level=3`,
runs contracts/example, pins the first available CPU, retains linked disassembly and raw samples.
The two-host summarizer consumes retained `arm-attempts.jsonl` and `xxl-attempts.jsonl`.

## Selection and limits

Start from `stable` when arrival order matters and `unstable` when equality order is unconstrained.
Compare group paths only when their admission contract holds. Radix is worth testing for large
fixed-width encoded keys; natural runs can favor merge paths. The dispatcher threshold is a
teaching choice, not a measured universal rule. See measurements for exact-source selections.

Signed integers, floats, strings and expensive comparators are explicit revisit scope. Other
revisits include indirect sorting of large payloads, cached-key extraction, reusable scratch,
weighted run-merge policies, comparator panic/drop safety, external spilling and parallel segments.
The first visit implements and tests every named catalog family and demonstrates group/dispatch
contracts. It does not infer production crossover thresholds or ISA-specific speedups.

## Primary sources

- [Rust 1.81 release](https://doc.rust-lang.org/releases.html#version-1810-2024-09-05).
- [Rust 1.98 stable source](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/slice/sort/stable/mod.rs).
- [Rust 1.98 unstable source](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/slice/sort/unstable/mod.rs).
- [Rust 1.98 run recognition](https://github.com/rust-lang/rust/blob/1.98.0/library/core/src/slice/sort/shared/mod.rs).
- [PostgreSQL 18 incremental-sort example](https://www.postgresql.org/docs/18/using-explain.html).
