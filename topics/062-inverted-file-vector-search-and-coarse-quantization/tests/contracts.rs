//! Independent full-sort, position-space reconstruction and assignment checks.

use inverted_file_vector_search_and_coarse_quantization::{Hit, Index, Mode, Vector, reference};

fn dist(a: &[i32; 8], b: &[i32; 8]) -> i64 {
    let mut sum = 0;
    for i in 0..8 {
        sum += i64::from(a[i] - b[i]).pow(2);
    }
    sum
}

fn order(mut hits: Vec<Hit>, k: usize) -> Vec<Hit> {
    hits.sort_unstable();
    hits.truncate(k);
    hits
}

#[test]
fn every_path_agrees_with_independent_reconstruction() {
    let centers = [[0; 8], [8; 8], [0; 8], [-8; 8]];
    let rows: Vec<Vector> = (-31_i16..=34)
        .map(|x| [x, x % 6, x / 3, -x, x % 8, 5, x % 7, x / 2])
        .chain([[0; 8], [0; 8], [i16::MIN; 8], [i16::MAX; 8]])
        .collect();
    let index = Index::new(rows.clone(), centers.to_vec());
    let dims = centers.map(|c| c.map(i32::from));
    for q in rows.iter().step_by(3) {
        let q32 = q.map(i32::from);
        let routed = order(
            dims.iter()
                .enumerate()
                .map(|(i, c)| (dist(c, &q32), i))
                .collect(),
            dims.len(),
        );
        for p in [0, 1, 2, 4, 7] {
            let scored: Vec<_> = rows
                .iter()
                .enumerate()
                .filter_map(|(id, row)| {
                    let row = row.map(i32::from);
                    let list = (0..dims.len())
                        .min_by_key(|&j| (dist(&dims[j], &row), j))
                        .unwrap();
                    if !routed.iter().take(p).any(|&(_, j)| j == list) {
                        return None;
                    }
                    let decoded = std::array::from_fn(|dim| {
                        let center = dims[list][dim];
                        [center, center - 12, center + 12]
                            .into_iter()
                            .min_by_key(|&v| (row[dim] - v).abs())
                            .unwrap()
                    });
                    Some(((dist(&q32, &row), id), (dist(&q32, &decoded), id)))
                })
                .collect();
            for k in [1, 5, 77] {
                let raw = order(scored.iter().map(|&(h, _)| h).collect(), k);
                let codes: Vec<_> = scored.iter().map(|&(_, h)| h).collect();
                let got = index.search(q, k, Mode::Raw(p));
                assert_eq!(got.scanned, scored.len());
                assert_eq!(got.hits, raw);
                assert_eq!(
                    index.search(q, k, Mode::Pq(p)).hits,
                    order(codes.clone(), k)
                );
                assert_eq!(index.search(q, k, Mode::Flat).hits, reference(&rows, q, k));
                for r in [0, 2, 9, 100] {
                    let shortlist = order(codes.clone(), r);
                    let exact = order(
                        shortlist
                            .iter()
                            .map(|&(_, id)| (dist(&q32, &rows[id].map(i32::from)), id))
                            .collect(),
                        k,
                    );
                    let got = index.search(
                        q,
                        k,
                        Mode::Rerank {
                            probes: p,
                            shortlist: r,
                        },
                    );
                    assert_eq!(got.hits, exact);
                    assert_eq!(got.reranked, shortlist.len());
                }
            }
        }
    }
}

#[test]
fn shortlist_and_probe_misses_are_distinct() {
    fn axis(x: i16) -> Vector {
        [x, 0, 0, 0, 0, 0, 0, 0]
    }
    let index = Index::new(
        [0, 2, 3, 6, 10].map(axis).to_vec(),
        [0, 10].map(axis).to_vec(),
    );
    let q = axis(5);
    assert_eq!(index.search(&q, 1, Mode::Raw(1)).hits, [(4, 2)]);
    assert_eq!(index.search(&q, 1, Mode::Raw(2)).hits, [(1, 3)]);
    assert_eq!(index.search(&q, 1, Mode::Pq(1)).hits, [(25, 0)]);
    for (shortlist, expected) in [(2, (9, 1)), (3, (4, 2))] {
        assert_eq!(
            index
                .search(
                    &q,
                    1,
                    Mode::Rerank {
                        probes: 1,
                        shortlist
                    }
                )
                .hits,
            [expected]
        );
    }
}

#[test]
fn empty_requests_underfill_and_extreme_coordinates() {
    let empty = Index::new(vec![], vec![[0; 8]]);
    assert!(empty.search(&[0; 8], 8, Mode::Raw(1)).hits.is_empty());
    let index = Index::new(vec![[i16::MIN; 8]], vec![[i16::MAX; 8]]);
    assert_eq!(
        index.search(&[i16::MAX; 8], 9, Mode::Flat).hits,
        [(34_358_689_800, 0)]
    );
    assert!(index.search(&[0; 8], 0, Mode::Flat).hits.is_empty());
    assert!(index.search(&[0; 8], 1, Mode::Raw(0)).hits.is_empty());
}
