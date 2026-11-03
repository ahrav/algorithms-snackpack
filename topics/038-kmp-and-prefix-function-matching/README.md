# KMP and prefix-function matching

Find every overlapping occurrence of a nonempty byte pattern. Offsets count
bytes. Empty patterns return `None`; empty text and `m > n` return no matches.
Inputs may include every byte value. Unicode normalization and case folding
require a separate transformation contract.

The running pattern `ababaca` occurs at `[2, 8]` in `abababacababaca`.
Its prefix function is `[0, 0, 1, 2, 3, 0, 1]`. Each entry gives the longest
proper prefix that is also a suffix of the prefix ending at that entry.

```text
BEFORE: consumed ababa, q=5; incoming b differs from pattern[5]=c.
DURING: q=pi[4]=3 retains aba; the same b extends it to q=4.
AFTER:  text[8] completes a match at 2; q=pi[6]=1 retains a.
        Continuing completes the second match at 8.
```

`q` is the matched-prefix length. Following failure links changes the pattern
state while the current text byte stays available. After a complete match,
following the final failure link preserves overlaps. `Stream` keeps `q` and
the absolute byte count across chunks. Empty chunks emit nothing; ending a
stream needs no flush. Absolute-offset overflow returns `None` before mutation.

## Alternatives and cost

Let `n` be text length, `m > 0` pattern length, `z` output count and
`A=max(0,n-m+1)` legal starts. Direct scanning uses up to `A*m` byte
comparisons. For the example, `A=9`, the bound is 63, and the actual count is 30.
KMP builds `m` prefix entries and uses aggregate `O(m+n+z)` work. The example's
scaling terms are `7+15+2`; this sum is not an exact instruction count.
Each successful extension raises state by one; fallback strictly decreases it.
Across the scan, total decreases are bounded by total increases. One byte may
still trigger several fallbacks.

The dense deterministic finite automaton (DFA) stores `(m+1)*256` transitions,
including an accepting row for overlaps. Preparation takes `O(256*m)` work;
search takes one table transition per byte plus output. At `m=7` and eight-byte
entries, the table occupies 16,384 bytes. KMP's prefix entries occupy 56 bytes,
plus seven owned pattern bytes. Output vectors and allocation metadata are extra.

Direct scanning is a useful tiny-input baseline. KMP suits forward-only chunks
and compact state. DFA suits workloads where measured transition cost pays
for its preparation and table. Repeated-prefix near misses and dense overlaps
make direct scanning expensive. Dense preparation can dominate short queries.
Read [measurements](measurements/README.md) for measured selections and limits.

## Run

From the repository root:

```sh
cargo test -p topic038-kmp-and-prefix-function-matching
cargo bench -p topic038-kmp-and-prefix-function-matching --bench matching -- near kmp total
cd topics/038-kmp-and-prefix-function-matching
python3 measurements/run.py
```

The Linux runner needs `rustc`, `taskset`, `objdump` and Python 3. It creates
`native/` with environment, source hashes, linked disassembly, correctness
results and raw timings. The benchmark accepts workload `tiny`, `random`,
`large`, `near`, `dense` or `binary`; candidate `direct`, `kmp` or `dfa`;
boundary `total` or `prepared`. The doctest expects `[2, 8]`.

## Sources and revisit scope

- [Knuth, Morris and Pratt, 1977](https://www.cs.jhu.edu/~misha/Spring23/Knuth77.pdf):
  linear matching and failure-state reuse.
- [Sedgewick and Wayne's dense KMP](https://raw.githubusercontent.com/kevin-wayne/algs4/master/src/main/java/edu/princeton/cs/algs4/KMP.java):
  alphabet-sized transition tables. This artifact independently implements an
  all-overlap byte contract.
- [Archive of Formal Proofs](https://isa-afp.org/entries/KnuthMorrisPratt.html):
  a separately verified algorithm; finite tests here remain bounded evidence.
- [Rust string APIs](https://doc.rust-lang.org/std/primitive.str.html#method.match_indices):
  disjoint matching and byte-offset semantics.

The authoritative catalog currently has no coverage arrays or
`ALGORITHM_INDEX.md`. This visit covers prefix construction, failure reuse,
overlaps, chunk state and dense tables. Topic 039 owns Boyer-Moore, Two-Way
and adaptive search. Unicode transformations, optimized failure links,
compressed alphabets, library comparisons, cold startup, callback output and
large-pattern table pressure remain explicit revisit scope.
