# Edit distance and automaton-guided lookup

Return all dictionary IDs within a global byte Levenshtein threshold. Insertion,
deletion and substitution cost one. Empty terms and queries are valid. Duplicate
terms preserve separate IDs. Results follow input order. Adjacent transposition
costs two; Unicode normalization and grapheme segmentation require caller policy.

For query `cat`, dictionary `[cat, cart, cut, dog]`, and threshold one, return
IDs `[0, 1, 2]`. The implementation compares arbitrary bytes.

```text
BEFORE                  DURING                 AFTER
query prefixes:         shared dictionary      terminal distances:
empty c ca cat          prefix ca              cat=0 cart=1 cut=1
row [0,1,2,3]           row [2,1,0,1]           accept all three
                                               do row [2,2,2,3]
                                               min=2 > k: prune dog
```

The external lesson includes a rendered, inspected version of this diagram.

## State and candidates

Let D(i,j) be the cheapest edits between a dictionary prefix of length i and
query prefix of length j. Each cell takes the minimum of deletion, insertion,
and diagonal alignment plus zero/equality or one/substitution. Boundaries are
D(i,0)=i and D(0,j)=j. A rolling row returns distance without traceback.

- `distance`: full-row baseline, one row allocation per term.
- `bounded`: rejects length differences above k, computes the diagonal band,
  and returns the exact distance when it is at most k. Active-band boundaries
  discard stale values. Thresholds as large as the longer input use full rows.
- `Myers`: prepares byte equality masks once per query; one-word difference
  updates support lengths 1..64. Empty patterns return term length; longer
  patterns dispatch to full rows. The low bit in `(ph << 1) | 1` charges for
  the global text prefix. Clearing it changes the alignment contract.
- `Trie`: immutable prefix sharing, iterative traversal, a row per evaluated
  edge, and subtree pruning when every row cell exceeds k. Terminal nodes emit
  all stored IDs. Sorting restores input order. The last cell alone cannot
  prune: prefix c has distance two to cat, but its descendant cat has zero.

Each completion of a prefix must cross some current row entry. Nonnegative
future edit costs justify the row-minimum lower bound.

## Costs and selection

N is dictionary entries, m query length, L typical term length, R output IDs,
and E evaluated trie edges. Full scanning uses O(NmL) cells and O(m) distance
workspace. Banded scanning uses O(m + L*min(m,2k+1)) work per eligible term in
this implementation, with O(m) workspace. One-word Myers uses O(m+256) query
preparation and O(NL) word updates for 1<=m<=64. Trie lookup uses O(Em) row work,
plus traversal and O(R log R) output sorting. The iterative stack retains rows
for pending nodes; its memory depends on the frontier, and can exceed O(mL).

For the running example, total term length is 13. The baseline evaluates 39
interior cells. Myers performs 13 word updates after preparation. The trie
examines nine edges. At m=L=48, k=1, full rows have 2304 cells per pair while
the band has at most 144 interior cells, plus initialization. These counts
bound work; measured time includes additional operations and allocation.

For Q queries, trie construction B, scan time S and trie query time T,
compare Q*S with B+Q*T. At Q=1 these are S and B+T; at Q=100 they are 100S and
B+100T. Construction amortizes when Q>B/(S-T), provided S>T. Read the measured
workload table before choosing. Dense output and broad thresholds can reduce
pruning. Tiny or frequently rebuilt dictionaries can favor scanning.

## Replay

From the repository root:

```sh
cargo test -p topic-041-edit
cargo run -p topic-041-edit --example edit
python3 topics/041-edit-distance-and-automaton-lookup/measurements/run.py
```

Expected example: `matching IDs=[0, 1, 2], evaluated edges=9`.
The runner requires Linux, Rust, Python 3, taskset and objdump. It compiles
path-limited inputs directly with rustc and records its flags and digests.
Run `--preflight` for two small contract workloads. Full raw evidence belongs
outside Git. Native output directories are scratch.

## Scope and revisit work

Implemented: global unit-cost byte distance, threshold bands, prepared one-word
Myers with a long-query fallback, and complete trie lookup. Revisit compiled
Levenshtein DFA construction and FST intersection, blocked Myers, Unicode units,
transpositions, weighted edits, substring matching, traceback, cold memory,
mixed dictionary generations, and production corpus comparisons separately.

The authoritative catalog at the starting revision has 54 topics and no
coverage arrays. ALGORITHM_INDEX.md is absent. The existing metadata pending
entry owns that repair; this topic preserves its identity and records scope.

## Primary sources

- [Wagner and Fischer, recurrence](https://computingbiology.github.io/docs/wagner1974.pdf).
- [Ukkonen, threshold-sensitive algorithms](https://www.cs.helsinki.fi/u/ukkonen/InfCont85.PDF).
- [Myers, bit-vector distance](https://doi.org/10.1145/316542.316550).
- [Pinned Rust-Bio search implementation](https://github.com/rust-bio/rust-bio/blob/2d7094091f5c7ae295511eeba17f33819d990d5d/src/pattern_matching/myers/simple.rs).
- [Windsor, edit automata and dictionary traversal](https://blog.aaw.io/2017/08/25/levenshtein-automata.html).

The Myers publisher PDF returned HTTP 403. The linked implementation and the
independent executed oracle support the implemented boundary and recurrence.
