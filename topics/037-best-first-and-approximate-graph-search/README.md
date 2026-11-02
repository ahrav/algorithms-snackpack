# Best-first and approximate graph search

Return the nearest min(k,N) vector IDs under squared Euclidean distance and
stable ID ties. Exact scan scores all vectors into a sorted k-entry buffer.
Single-layer search uses a minimum frontier, maximum retained heap and visited
bits. Width bounds retained results; pending nodes survive retained-set eviction.
A frontier distance supplies no lower bound on undiscovered points.

```text
Before: q=0, entry A. Squared distances: A=100 B=4 C=9 D=0.
Edges: A--B, A--C, C--D.
During: width 1 keeps B; width 2 keeps B and C.
After: width 1 returns B; width 2 expands C and reaches D.
```

The rendered/inspected figure is retained externally with VISUAL_REVIEW.json.

## Contract and example

Points and queries have equal dimension 1..1024 and integer coordinates
-10000..10000, with i64 accumulation. Empty graphs accept empty queries.
Self matches and distinct IDs with equal coordinates remain eligible. Links may
repeat or cycle. Construction rejects invalid IDs and shapes. Search validates
query, entry and width, returning sorted distinct results. Disconnection can
underfill. k=0 returns empty. Width must be at least min(k,N) for nonempty output.

For q=0, A=10,B=2,C=3,D=0 and undirected edges A-B,A-C,C-D,
width 1 returns B; width 2 keeps C and reaches D. An integration test checks this.

## Alternatives and costs

Implemented: exact flat scan and widths 4/16/64 on one deterministic graph.
HNSW adds hierarchy and entry descent. NSG invests in sparse connected topology.
DiskANN adds storage-aware search. Their full construction/execution remains
revisit scope. This lab measures single-layer search, not those engines.

N is corpus count, d dimension, k requested count, V scored nodes, X expanded
nodes, M maximum degree, F maximum frontier size and ef retained width.
Flat costs O(Nd+Nk), with O(k) result storage. Graph costs
O(N+Vd+XM+V log F+V log ef), including visited initialization, with
O(N+F+ef) query storage. Construction fully ranks neighbors per vector,
O(N squared times (d+log N)), and adds ring links. Logical vector payload is
4Nd bytes; links use native usize plus Vec overhead. Capacities/RSS need separate
measurement. Width changes cannot repair a disconnected component.

If build takes B and scan/graph queries take ts/tg with ts>tg, amortization needs
Q>B/(ts-tg) queries. When tg>=ts query savings do not pay B. Select exact for
required exactness or when qualified graph options lose. Select graph only with
measured quality, query savings and acceptable build/memory costs.

## Run

```bash
cargo test -p topic037-graph-search
cargo bench -p topic037-graph-search --bench search
cd topics/037-best-first-and-approximate-graph-search
python3 measurements/run.py --out /tmp/topic037-run
python3 measurements/summarize.py /tmp/topic037-run
```

The bare bench executes case 0/flat. The Python runner supplies the full matrix,
oracle, process schedule and environment/source checks. See BENCHMARK.md.

## Sources and scope

- [HNSW, Algorithms2/5](https://arxiv.org/pdf/1603.09320).
- [hnswlib parameters](https://github.com/nmslib/hnswlib/blob/master/ALGO_PARAMS.md).
- [NSG](https://arxiv.org/abs/1707.00143).
- [DiskANN](https://proceedings.neurips.cc/paper/2019/file/09853c7fb1d3f8ee67a61b6bf4a7f8e6-Paper.pdf).

Catalog origin/main has Topic037 but no coverage array or ALGORITHM_INDEX.md.
Explicit revisit: full HNSW/NSG/Vamana construction, build seeds/orders, filters,
fallback, deletion/churn, float/quantized scoring, reusable visited storage,
wider k, layout, cold storage, concurrent load, and allocation-bounded retained
buffers (pre-sized heap with in-place root replacement, pop-before-insert scan
buffer, method parsed once in the bench).
