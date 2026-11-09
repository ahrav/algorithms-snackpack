# Multi-pattern matching and automata

Find every overlapping match of a fixed set of nonempty byte patterns. Preserve
original pattern IDs, including duplicates. `Hit` uses half-open byte offsets.
Emit in increasing end position; ties have unspecified order. Empty pattern
sets succeed. Empty patterns return their first index before output. Allocation
failure follows ordinary Rust allocation behavior. The stream cursor checks
length overflow before consuming a chunk; callbacks may panic after partial output.

## Running example

Patterns `he`, `she`, `hers`, `his`, `he` in `ushers` yield IDs 1 at [1,4),
0 and 4 at [2,4), and 2 at [2,6). At `ush` the current prefix is `sh`; `e`
completes `she` and its output suffix `he`. Failure from `she` to `he` permits
continuing with `r,s` to finish `hers`. Each terminal stores only its own IDs.
Output links enumerate terminal suffixes. A cursor carries state and offset
across chunks, including empty chunks.

## Candidates and cost model

`n` is input bytes, `r` patterns, `P` summed pattern bytes, `S` states, `d`
maximum outgoing degree, `z` emitted records, and `R` searches per generation.
The example has n=6, r=5, P=14, S=10, d=2, z=4.

| Candidate | Work | Storage and lifecycle |
|---|---|---|
| direct | n*r eligibility checks; worst-case O(n*P+z) | borrowed patterns in scan; owned clone in build endpoint |
| sparse | O(n*d+z), linear edge lookup, failure/output links | O(S+r) metadata with output IDs stored once |
| dense | n completed transitions plus output enumeration | sparse metadata plus 256*S usize slots |

The running example's dense table has 10*256*8=20,480 payload bytes on 64-bit
hosts. Other metadata, allocator cost and construction scratch are extra.
Dense construction adds O(256*S) to sparse construction. Shared prefixes save
states. Sparse lookup costs depend on fanout. Output enumeration costs at least
O(z); duplicate IDs contribute to z. Collecting four Hits needs 96 payload bytes
on these targets. The experiment consumes each Hit through a digest callback.

Lifetime cost is B+R*Q, where B is construction including destruction and Q is
one reused scan. For R=10 compare B_direct+10*Q_direct, B_sparse+10*Q_sparse,
and B_dense+10*Q_dense. If Q_sparse>Q_dense, dense pays back when
R>(B_dense-B_sparse)/(Q_sparse-Q_dense). This is a workload projection.

## Run

```sh
cargo test -p topic043
cargo run --release -p topic043 --example experiment -- example
python3 topics/043-multi-pattern-matching-and-automata/measurements/run.py
```

The Python runner creates an external temporary evidence directory and prints
its path. It needs Linux, Python 3, rustc and taskset. It builds exact local
source with native target features. See BENCHMARK.md and measurements/README.md.

Use direct scans for small or short-lived sets when measured lifetime cost wins,
and for dense-output workloads where output enumeration dominates: x86 case9
(z=130,576) selected direct on the reused scan endpoint itself. Use dense
transitions when saved query time pays for construction and table storage; dense
won19 of22 scan cells. Sparse won no scan or build cell in this screen, so it is
the fallback when the dense payload 256*S*sizeof(usize) exceeds the memory budget.
Consult measured per-cell selections before choosing a runtime winner.

Production work should compare a maintained library with the exact API contract.
`aho-corasick` 1.1.5 offers overlapping resident matching under Standard semantics;
its high-level stream iterator selects non-overlapping matches. Leftmost selection
requires a different contract. Unicode transformation and source-span recovery
also require separate design. Revisit scope: library comparison, compressed
alphabets, vector prefilters, recoverable quotas, Unicode, leftmost selection,
and concurrent generations. The current catalog has no coverage arrays or
ALGORITHM_INDEX.md; this explicit scope preserves the variants for follow-up.

## Primary sources

- [Aho and Corasick, 1975](https://cr.yp.to/bib/1975/aho.pdf): transition,
  failure and output functions.
- [MatchKind 1.1.5](https://docs.rs/aho-corasick/1.1.5/aho_corasick/enum.MatchKind.html):
  overlap and leftmost semantics.
- [AhoCorasick 1.1.5](https://docs.rs/aho-corasick/1.1.5/aho_corasick/struct.AhoCorasick.html):
  resident and streaming APIs.
