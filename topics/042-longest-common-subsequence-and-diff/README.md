# Longest common subsequence and diff

Return a minimum insertion/deletion script over exact bytes. Each operation
consumes or emits its recorded byte; replay consumes the whole source and emits
the exact target. Replacement costs two. Ties permit different optimal scripts.
Lengths and score storage must fit addressable memory. Allocation follows Rust's
allocator policy. This teaching API requires callers to bound hostile inputs.

For `ABCABBA` to `CBABAC`, one LCS is `CBBA`, length 4. Delete 3 and insert 2.

```text
BEFORE  source: A B C A B B A     target: C B A B A C
DURING  source indices 2,4,5,6   target indices 0,1,3,4
AFTER   -A -B C -A B +A B A +C  -> C B A B A C
```

| Candidate | Mechanism | Time | Score storage | Selection constraint |
|---|---|---|---|---|
| Matrix | prefix LCS scores and traceback | O(mn) | O(mn) | small inputs, simple reconstruction |
| Hirschberg | forward/reverse rows and recursive splits | O(mn) | O(min(m,n)) | memory budget; rows released before recursion |
| Myers | furthest reachable position per edit count and diagonal | O((m+n)(D+1)) | O(D²) compact history | similar inputs; large D raises time and memory |

Here m and n are byte lengths, L is LCS length, and D=m+n-2L is minimum
insertion/deletion count. In the example m=7,n=6,L=4,D=5: matrix visits 42
cells and stores 56 scores, or 448 payload bytes with 64-bit usize. Myers'
complete history layers through D=5 would contain 21 scores. Our last layer
can terminate early; allocated capacity is counted. Vector headers, input,
matching positions, allocator overhead and O(m+n) script storage are additional.

Hirschberg chooses a target split maximizing forward[j]+reverse[n-j]. For
source split `ABC | ABBA`, target split `C | BABAC` permits 1+3=4 kept bytes.
Myers uses diagonal k=x-y, where x and y count consumed source and target bytes.
The first frontier reaching both ends has minimum D. Repeated tokens admit
multiple optima; exact minimum cost does not specify review-friendly hunks.

```sh
cargo test -p topic-042-lcs-diff
cargo run --release -p topic-042-lcs-diff --example compare -- matrix sparse 256 10
cargo run --release -p topic-042-lcs-diff --example compare -- hirschberg sparse 256 10
cargo run --release -p topic-042-lcs-diff --example compare -- myers sparse 256 10
```

Each run checks the script against a separate forward edit-graph distance and
exact replay before timing. Full experiment and results: [benchmark](BENCHMARK.md)
and [measurements](measurements/README.md).

Coverage audit: the authoritative catalog has no coverage array for Topic 042;
ALGORITHM_INDEX.md is absent. Implemented: exact full-table LCS, Hirschberg and
Myers compact-history scripts. Explicit revisits: Hunt-Szymanski sparse matching,
bit-parallel length-only kernels, a symmetric length-one Hirschberg base case,
linear-space Myers traceback, patience and histogram presentation, line/Unicode
tokenization, preprocessing/common-prefix trimming and production-library
comparison. Unit substitution distance belongs to Topic 041; exact reconstruction
and presentation objectives stay distinct.

Primary sources: [Hirschberg 1975](https://www.cs.emory.edu/~cheung/Courses/253/Syllabus/DynProg/Docs/Hirschberg%3DLinear-space-LCS.pdf),
[Myers 1986](https://gi.cebitec.uni-bielefeld.de/_media/teaching/2013summer/818myers-1986-difference-algorithm.pdf),
and [Git diff algorithm options](https://git-scm.com/docs/diff-options).
