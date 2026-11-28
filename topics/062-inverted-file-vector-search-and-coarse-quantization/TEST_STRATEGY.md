# Test strategy

The exact reference visits original rows and full-sorts distance/ID pairs.
A second model independently sorts centroids, assigns rows, reconstructs
coordinates, computes distances in position space and full-sorts each score.
It does not use lookup tables, heap replacement, the index's assignments or
the index's tie handling.

| Invariant | Inputs and checks | Plausible violation |
| --- | --- | --- |
| Exact order and widened coordinates | Original row order breaks ties, including duplicate vectors. Extreme i16 inputs preserve squared sums. | Select arbitrary tie IDs or subtract into i16. |
| Assignment and probe scope | Duplicate and empty lists, no probes, oversized probe budgets and boundary queries. | Pick a different centroid on a tie or silently scan an extra list. |
| Residual score matches reconstruction | Position-space scoring over 24 queries, five probe counts and three result widths. | Omit the centroid from the query residual or use the wrong block code. |
| Exact rerank stays inside PQ candidates | Compare four shortlist capacities, including zero, underfill and all candidates. | Expand to an unopened list or order survivors by compressed score. |
| Exact scan and shortlist limit | All raw probes match the sorted full oracle on every benchmark query. Exact reranking agrees with full-sorting the retained originals and preserves candidate-coverage recall. | Lose an eligible entry or truncate the compressed pool to k before reranking. |

The focused case isolates routing and compressed ties. Measurements check
the result and scanned-count digest against the untimed run. These are
finite checks under the fixed integer and static-corpus contract.
