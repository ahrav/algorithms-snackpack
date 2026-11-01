# Merkle trees and incremental rebuild caches

An ordered hash tree lets unchanged subtrees retain their digests. A batch can
recompute shared ancestors once. This crate compares full rebuilding, eager
root-path updates and sorted/deduplicated batch frontiers.

## Contract and example

Use a nonempty power-of-two leaf count. Preserve leaf position and child order.
Hash leaves as SHA256(0x00 || bytes) and parents as
SHA256(0x01 || left_digest || right_digest). Validate all write indices before
mutation; duplicate indices use last-write-wins input order. Eager and Batch
skip exact byte-equal writes. Allocation failure follows Rust's allocator policy.
The synchronous mutable API exposes the completed root after returning.

| State | Leaf values | Recomputed digests | Root |
|---|---|---|---|
| Before | A, B, C, D | initial seven digests | R |
| During private update | A, B, C2, D | c2, Q2, R2 | publish after completion |
| After | A, B, C2, D | unchanged P retained | R2 |

Here a=H(0||A), b=H(0||B), c=H(0||C), d=H(0||D),
P=H(1||a||b), Q=H(1||c||d), R=H(1||P||Q).
After replacing C, c2=H(0||C2), Q2=H(1||c2||d), R2=H(1||P||Q2).
The proof for index 2 is [d, P]. Verify exact proof length, index, tree size,
operand order and an externally authenticated root. Bind the root to its version.

## Costs and choice

Let n be leaves, h=log2(n), c changed writes and D distinct dirty ancestors.
Full performs 2n-1 hashes. Eager performs c(h+1). Batch performs c+D plus
per-level sort/dedup work and scratch allocation. With four leaves, changing C
costs 7/3/3 calls. Changing C and D costs 7/6/4 calls.

For b record bytes, full hashing processes n(b+1)+65(n-1) input bytes.
For n=4,b=1 this is 203 bytes. One path processes 2+2*65=132 bytes.
Padding and compression add internal work. Useful digest storage is
32(2n-1), or 224 bytes at n=4. This heap layout reserves one extra 32-byte slot.
Record storage, Vec metadata, allocator overhead, batch scratch and snapshots
add memory. Hash calls and bytes are measured-independent algorithm counts.

Use Full for construction/dense changes, Eager for sparse changes or intermediate
roots, and Batch when one root per burst permits shared recomputation. Consult
measurements/README.md for the actual workload-dependent selections.

A build-cache action key must identify its computation version, toolchain,
command, relevant environment and complete ordered dependencies. Encode field
boundaries canonically. Publish a result only for the input snapshot used to
compute it. Content identity and action identity serve different purposes.

## Run

From this directory:

```sh
rustc --edition=2024 --test src/lib.rs -o /tmp/topic036-tests
/tmp/topic036-tests
python3 measurements/run.py --out /tmp/topic036-results
python3 measurements/summarize.py /tmp/topic036-results
```

From the workspace: `cargo test -p topic036-merkle-trees --all-targets`.
The harness's default invocation is a small correctness-checked smoke workload.

## Evidence boundaries and revisits

Portable allocating SHA-256 supports a dependency-free lesson replay. Check it
against Python hashlib before timing. Use a maintained crypto library in a
production application. Timings include this portable implementation's costs.
Independent correctness checks establish bounded agreement, not cryptographic
validation. Current timing excludes discovery of changed files, network/disk
repair, proof production, concurrency, and cache lookup. Lifecycle includes
construction, one update and destruction; update timing uses an existing tree.

The catalog's Topic 036 has no coverage array and ALGORITHM_INDEX.md is absent.
Explicit revisits: arbitrary-size RFC tree shape; append-only consistency
proofs; sparse Merkle maps; high fanout; authenticated DAGs; persistent snapshots;
production hash backends; action-cache execution and distributed anti-entropy.

## Primary sources

- [RFC 9162 section 2.1](https://www.rfc-editor.org/rfc/rfc9162.html#section-2.1): tree encoding and inclusion proofs.
- [NIST FIPS 180-4](https://csrc.nist.gov/pubs/fips/180-4/upd1/final): SHA-256 algorithm.
- [Bazel remote caching](https://bazel.build/remote/caching): action cache, declared inputs and content-addressable storage.
