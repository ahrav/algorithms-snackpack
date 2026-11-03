# Test strategy

Scope: nonempty arbitrary-byte patterns, complete overlapping offsets,
prefix-table preprocessing, resident DFA and persistent KMP streams.
Contract and failure classes: empty-pattern rejection, absent/empty/short text,
overlaps, wrong borders, chunk loss, duplicate emission and offset overflow.
Primary technique: differential testing against independent slice equality.
Secondary techniques: exhaustive small binary inputs, brute-force border
enumeration, every two-chunk split, one-byte chunks and named boundary cases.
Oracle: `oracle` uses slice windows and equality without calling a candidate.
Input/model/schedule domain and bounds: binary text lengths 0..8 and pattern
lengths 1..4; all byte values in a separate fixture; deterministic benchmarks.
Replay assets: `src/lib.rs`, `tests/contracts.rs`, benchmark and Linux runner.
What a green run establishes: agreement on tested complete output vectors and
atomic rejection for the injected offset boundary.
Known gaps and delegated skills: finite tests support the implementation;
the aggregate-work argument is separate. Allocation failure follows Rust's
allocator behavior. Unicode transformation and production library comparison
are revisit scope. `string-matcher-selection` owns production selection.
Concrete tests to add: transformed-text mappings if that API is introduced.

## Invariant audit

| Claim and scope | Plausible violation | Observation and independent oracle | Smallest discriminator |
|---|---|---|---|
| Complete ordered overlaps | Reset q to zero after a match | Exact vector equals independent slice-window vector | `aaa` in `aaaaa`: `[0,1,2]` |
| Border correctness | Use preceding pi without failure traversal | Full pi vector equals brute-force prefix/suffix equality | `abaa`: `[0,0,1,1]`; multi-step `ababaa`: `[0,0,1,2,3,1]` |
| Stream continuation | Reset state at each chunk | Concatenated stream output equals resident oracle after every split | `aaa` split across one-byte chunks |
| Empty-chunk progress | Emit a previous match twice | Exact ordered vectors with inserted empty chunks | Empty chunk after a completed match |
| DFA accepting row | Clear state after acceptance | Exact vector equals independent oracle | `aaa` in `aaaaa` |
| Rejection before mutation | Update q or absolute offset before overflow check | `None`, followed by exact q/offset equality | consumed=usize::MAX, push one byte |

Mutation backstop: each listed wrong behavior reaches an exact assertion.
No mutation-testing campaign was run. Binary oracle tests preserve order and
multiplicity; canonicalization cannot hide duplicates or missed overlaps.
