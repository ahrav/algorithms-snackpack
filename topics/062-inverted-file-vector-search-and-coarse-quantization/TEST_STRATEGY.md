# Test strategy

The exact reference visits original rows and full-sorts distance/ID pairs.
A second model independently sorts centroids, assigns rows, reconstructs
coordinates, computes distances in position space and full-sorts each score.
It does not use lookup tables, heap replacement, the index's assignments or
the index's tie handling.

| Claim and scope | Plausible violation | Observation and comparator | Oracle and independence | Smallest discriminating case | Mutation backstop |
| --- | --- | --- | --- | --- | --- |
| Exact order and widened coordinates. Flat search over 70 rows with duplicate vectors and extreme i16 rows, k of 1, 5 and 77. | Select arbitrary tie IDs or subtract into i16. | Full ordered distance/ID vectors, compared for exact equality. | Library full-sort `reference` and one hand-computed distance. Neither uses a heap. | One row at i16::MIN queried at i16::MAX must score 34,358,689,800. The duplicate zero rows must rank by ID. | Prefer the larger ID on equal flat distance. Subtract with wrapping i16. |
| Assignment and probe scope. Duplicate and empty lists, probe counts 0, 1, 2, 4 and an oversized 7, and a query on a centroid tie. | Pick a different centroid on a tie or silently scan an extra list. | Scanned-entry count and full raw hit vectors per probe count, compared for exact equality. | Test-local model that sorts centroid distance/index pairs and assigns rows by the same pair. | Query x=5 between centroids 0 and 10 with one probe opens list 0 only and returns ID 2 at distance 4. | Assign to the last nearest centroid. Prefer the larger centroid index at query time. Open one extra list. |
| Residual score matches reconstruction. 24 queries, five probe counts and three result widths. | Omit the centroid from the query residual or use the wrong block code. | Full PQ score/ID vectors, compared for exact equality. | Test-local position-space distance to coordinates rebuilt from the centroid and the nearest of three offsets, with no lookup table. | Any list with a non-zero centroid. In the focused case IDs 0, 1 and 2 share one code and ID 0 wins at compressed distance 25. | Build the table without the centroid. Reverse the block order of a code. Prefer the larger ID on equal compressed score. |
| Exact rerank stays inside PQ candidates. Shortlist capacities 0, 2, 9 and 100, covering zero, underfill and all candidates. | Expand to an unopened list or order survivors by compressed score. | Rerank hit vectors and the reranked count, compared for exact equality. | Test-local full sort of compressed scores, truncation to the shortlist, then a full sort of exact distances. | Focused case with one probe: shortlist 2 returns ID 1 at distance 9 and shortlist 3 returns ID 2 at distance 4. | Open one extra list for rerank. Order survivors by compressed score. |
| Exact scan and shortlist limit. Every benchmark query in the untimed quality mode. | Lose an eligible entry or truncate the compressed pool to k before reranking. | Flat and all-probe raw hit vectors, rerank hit vectors and candidate-coverage counts, compared for exact equality. | Library full-sort `reference` over all rows and over the retained originals. | The 256-row tiny cell. In the focused case one probe with k=1 must return ID 2, the last entry of list 0. | Skip the last entry of each raw list. Truncate the compressed pool to k. |

Each mutation in the last column was applied alone to `src/lib.rs`. Every
one fails `cargo test --test contracts`. The last row's mutations also fail the
assertions of the benchmark quality mode.

The focused case isolates routing and compressed ties. Measurements check
the result and scanned-count digest against the untimed run. These are
finite checks under the fixed integer and static-corpus contract.
