//! Independent full-sort and sorted-frontier checks.
use topic037_graph_search::{Graph, Neighbor};

fn oracle(g: &Graph, q: &[i32], k: usize) -> Vec<Neighbor> {
    let mut scores = Vec::new();
    for (id, p) in g.points().iter().enumerate() {
        let mut total = 0;
        for j in 0..q.len() {
            total += (i64::from(p[j]) - i64::from(q[j])).pow(2);
        }
        scores.push((total, id));
    }
    scores.sort_unstable();
    scores.truncate(k);
    scores
}

fn reference(g: &Graph, q: &[i32], k: usize, ef: usize) -> Vec<Neighbor> {
    let scores = oracle(g, q, g.points().len());
    let score = |id| *scores.iter().find(|x| x.1 == id).unwrap();
    let mut pending = vec![score(0)];
    let mut retained = pending.clone();
    let mut visited = std::collections::BTreeSet::from([0]);
    while !pending.is_empty() {
        pending.sort_unstable();
        let current = pending.remove(0);
        retained.sort_unstable();
        if retained.len() == ef && current > *retained.last().unwrap() {
            break;
        }
        for &id in &g.edges()[current.1] {
            if !visited.insert(id) {
                continue;
            }
            let item = score(id);
            if retained.len() < ef || item < *retained.last().unwrap() {
                pending.push(item);
                retained.push(item);
                retained.sort_unstable();
                retained.truncate(ef);
            }
        }
    }
    retained.sort_unstable();
    retained.truncate(k);
    retained
}

#[test]
fn width_retains_required_worse_route() {
    let g = Graph::new(
        vec![vec![10], vec![2], vec![3], vec![0]],
        vec![vec![1, 2], vec![0], vec![0, 3], vec![2]],
    )
    .unwrap();
    assert_eq!(g.search(&[0], 1, 1, 0).unwrap().0, vec![(4, 1)]);
    assert_eq!(g.search(&[0], 1, 2, 0).unwrap().0, vec![(0, 3)]);
}

#[test]
fn graph_and_scan_agree_with_independent_models() {
    for n in 1..30 {
        let points = (0..n)
            .map(|i| {
                vec![
                    i32::try_from(i * 17 % 31).unwrap(),
                    i32::try_from(i * 7 % 23).unwrap(),
                ]
            })
            .collect();
        let edges = (0..n)
            .map(|i| vec![(i + 1) % n, (i + n - 1) % n, i, (i + 3) % n, (i + 1) % n])
            .collect();
        let g = Graph::new(points, edges).unwrap();
        for q in [vec![0, 0], vec![9, 17], vec![-12, 30]] {
            for k in [1, 4, 40] {
                assert_eq!(g.exact(&q, k).unwrap().0, oracle(&g, &q, k));
                for width in [k.min(n), n] {
                    let got = g.search(&q, k, width, 0).unwrap();
                    assert_eq!(got.0, reference(&g, &q, k, width));
                    assert!(got.1.distances <= n);
                    if width == n {
                        assert_eq!(got.0, oracle(&g, &q, k));
                    }
                }
            }
        }
    }
}

#[test]
fn empty_ties_disconnection_and_input_contracts() {
    let empty = Graph::new(vec![], vec![]).unwrap();
    assert!(empty.exact(&[], 4).unwrap().0.is_empty());
    assert!(empty.search(&[], 4, 0, 0).unwrap().0.is_empty());
    let g = Graph::new(
        vec![vec![1], vec![-1], vec![0]],
        vec![vec![1, 1, 0], vec![0], vec![]],
    )
    .unwrap();
    assert_eq!(g.exact(&[0], 9).unwrap().0, vec![(0, 2), (1, 0), (1, 1)]);
    assert_eq!(g.search(&[0], 3, 3, 0).unwrap().0, vec![(1, 0), (1, 1)]);
    assert!(g.search(&[0], 2, 1, 0).is_err());
    assert!(g.search(&[0], 1, 1, 3).is_err());
    assert!(g.exact(&[0, 1], 1).is_err());
    assert!(g.exact(&[10001], 1).is_err());
    assert!(Graph::new(vec![vec![i32::MIN]], vec![vec![]]).is_err());
    assert!(Graph::new(vec![vec![0]], vec![vec![1]]).is_err());
    assert!(Graph::new(vec![vec![]], vec![vec![]]).is_err());
    assert!(Graph::new(vec![vec![1], vec![1, 2]], vec![vec![], vec![]]).is_err());
    let bound = Graph::new(vec![vec![-10000; 1024]], vec![vec![]]).unwrap();
    assert_eq!(
        bound.exact(&vec![10000; 1024], 1).unwrap().0[0].0,
        409_600_000_000
    );
}
