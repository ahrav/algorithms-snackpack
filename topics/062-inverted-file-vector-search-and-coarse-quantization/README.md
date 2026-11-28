# Inverted-file vector search and coarse quantization

Find up to k nearest original row IDs under squared Euclidean distance and
break ties by smaller ID. The lab uses eight signed 16-bit coordinates and
64-bit distances. The input position is the stable row ID.

Build inverted lists by assigning each vector to its nearest supplied
centroid. Break assignment ties by centroid position. At query time, score
the centroids exactly and open the nearest requested groups. Scan each
selected list completely. Exhaustive raw-list search equals flat search
under this contract.

| Path | Work saved | Failure case |
| --- | --- | --- |
| Flat | No index routing or compressed scoring | Reads every original per query |
| IVF raw | Reads only vectors in opened lists | The nearest vector can belong to a list whose centroid ranks lower |
| IVF residual product quantization (PQ) | Looks up scores from four codes per vector | Reconstruction can shift ranking, including at full probe width |
| IVF PQ with exact rerank | Reads originals only for a fixed compressed shortlist | Exact scoring cannot retrieve absent IDs |

Each PQ code selects one of nine representatives for one two-coordinate block
of the vector minus its centroid. The query remains uncompressed. For each
opened list, compute the squared distance from each query-residual block
to each codeword. Sum one lookup per block to score each entry. This
asymmetric distance matches exact distance to the reconstructed coordinates.

## Worked case

Query x=5 with centroids at 0 and 10 and row coordinates [0, 2, 3, 6, 10].
Other coordinates are zero. Flat search returns ID 3 at squared distance 1.
One probe opens list 0 on the centroid tie. Raw search returns ID 2 at distance
4. PQ maps IDs 0, 1 and 2 to the same residual code and returns ID 0. Rerank
with two candidates and ID 1 wins. With three candidates ID 2 wins. ID 3 only
reaches ranking when search opens both lists.

## Cost and choice

For N rows of d coordinates, flat work is Nd coordinate contributions.
With L centroids, p probes and S total rows in those p lists, raw IVF scores
(L+S)d coordinate contributions. The worked case uses N=5, d=8, L=2, p=1 and
S=3. Both flat and raw IVF score 40 coordinate contributions before selection.
Estimate S with actual query-weighted list lengths. The uniform estimate
pN/L gives 2.5 rows in this example.

With M blocks and C codewords per block, PQ adds pCd table setup contributions,
SM lookups and, for R reranked rows, Rd raw contributions. The worked case
with M=4, C=9 and R=2 uses 72 table contributions, 12 lookups and 16 re-score
contributions. Separately account for the bounded selection heaps and raw
vector fetches. The experiment stores four code bytes and 16 original
coordinate bytes per row, before ID, array and centroid overhead.

Start with flat when exact coverage is required or routing overhead dominates.
Use an IVF candidate only when its measured recall, actual scans and cost meet
the workload requirements. Increase probes to add lists. Increase shortlist
size to keep more candidates from the same lists. Exact reranking achieves
the exact-top-k coverage of its input shortlist when it applies the same
metric and total order as the global oracle.

Fixed centroids and codebooks isolate query behavior. Train both on
representative data in production and measure training cost, distribution
shift, bytes and cold reads separately.

```bash
cargo run --release -p inverted-file-vector-search-and-coarse-quantization --example demo
cargo test -p inverted-file-vector-search-and-coarse-quantization
cargo bench -p inverted-file-vector-search-and-coarse-quantization --bench search --no-run
```

See [benchmark plan](BENCHMARK.md), [test strategy](TEST_STRATEGY.md) and
[measurements](measurements/README.md).

Primary sources include [Jégou, Douze and Schmid](https://doi.org/10.1109/TPAMI.2010.57),
[Faiss index families](https://github.com/facebookresearch/faiss/wiki/Faiss-indexes)
and [Faiss table implementation notes](https://github.com/facebookresearch/faiss/wiki/Implementation-notes).
