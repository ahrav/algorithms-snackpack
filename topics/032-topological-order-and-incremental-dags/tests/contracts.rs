//! Admission, ordering and incremental-value contracts.
use topic_032_topological_order_and_incremental_dags::{
    Error, Evaluator, Graph, Ordering, Update, oracle,
};

fn valid(n: usize, edges: &[(usize, usize)], order: &[usize]) {
    assert_eq!(order.len(), n);
    let mut rank = vec![usize::MAX; n];
    for (i, &u) in order.iter().enumerate() {
        assert!(u < n);
        assert_eq!(rank[u], usize::MAX);
        rank[u] = i;
    }
    for &(u, v) in edges {
        assert!(rank[u] < rank[v]);
    }
}
#[test]
fn exact_example_and_shared_consumer() {
    let edges = [(0, 2), (1, 2), (2, 3), (2, 4)];
    let g = Graph::new(5, &edges).unwrap();
    for ordering in [Ordering::Fifo, Ordering::Minimum, Ordering::Dfs] {
        valid(5, &edges, &g.topological(ordering).unwrap());
    }
    for update in [Update::Full, Update::Dirty, Update::Heap] {
        let mut e = Evaluator::new(g.clone(), vec![2, 3, 0, 1, 2]).unwrap();
        assert_eq!(e.values(), [2, 3, 5, 6, 7]);
        e.update(&[(0, 4), (1, 6), (0, 5)], update).unwrap();
        assert_eq!(e.values(), [5, 6, 11, 12, 13]);
        e.update(&[(0, 4), (1, 3)], update).unwrap();
        assert_eq!(e.values(), [4, 3, 7, 8, 9]);
    }
}
#[test]
fn rejected_inputs_preserve_state() {
    assert_eq!(Graph::new(1, &[(0, 1)]), Err(Error::Vertex));
    assert_eq!(Graph::new(2, &[(0, 1), (0, 1)]), Err(Error::Duplicate));
    assert_eq!(Graph::new(1, &[(0, 0)]), Err(Error::Cycle));
    assert_eq!(Graph::new(3, &[(0, 1), (1, 2), (2, 1)]), Err(Error::Cycle));
    let g = Graph::new(3, &[(0, 1), (1, 2)]).unwrap();
    assert!(matches!(
        Evaluator::new(g.clone(), vec![]),
        Err(Error::Length)
    ));
    for mode in [Update::Full, Update::Dirty, Update::Heap] {
        let mut e = Evaluator::new(g.clone(), vec![1, 1, 1]).unwrap();
        assert_eq!(
            e.update(&[(0, 9), (3, 4)], mode).unwrap_err(),
            Error::Vertex
        );
        assert_eq!(e.bases(), [1, 1, 1]);
        assert_eq!(e.values(), [1, 2, 3]);
        assert_eq!(e.insert(2, 0), Err(Error::Cycle));
        assert_eq!(e.values(), [1, 2, 3]);
        assert_eq!(e.insert(0, 1), Err(Error::Duplicate));
        assert_eq!(e.insert(4, 0), Err(Error::Vertex));
        e.update(&[(0, 2)], mode).unwrap();
        assert_eq!(e.values(), [2, 3, 4]);
    }
}
#[test]
fn forward_backward_insertions_and_cycle_rejection() {
    let mut g = Graph::new(4, &[(0, 2)]).unwrap();
    g.insert(3, 1).unwrap();
    valid(4, &[(0, 2), (3, 1)], g.order());
    let before = g.clone();
    assert_eq!(g.insert(1, 3), Err(Error::Cycle));
    assert_eq!(g, before);
    let mut e = Evaluator::new(g, vec![1; 4]).unwrap();
    e.insert(0, 3).unwrap();
    assert_eq!(e.values(), oracle(4, &[(0, 2), (3, 1), (0, 3)], &[1; 4]));
}
#[test]
fn empty_noop_wrapping_and_cancellation() {
    let g = Graph::new(0, &[]).unwrap();
    for o in [Ordering::Fifo, Ordering::Dfs, Ordering::Minimum] {
        assert!(g.topological(o).unwrap().is_empty());
    }
    let mut e = Evaluator::new(g, vec![]).unwrap();
    assert_eq!(e.update(&[], Update::Heap).unwrap().evaluated, 0);
    for m in [Update::Full, Update::Dirty, Update::Heap] {
        let mut e = Evaluator::new(
            Graph::new(4, &[(0, 2), (1, 2), (2, 3)]).unwrap(),
            vec![1, 2, 0, 0],
        )
        .unwrap();
        e.update(&[(0, 0), (1, 3)], m).unwrap();
        assert_eq!(e.values(), [0, 3, 3, 3]);
        e.update(&[(0, u64::MAX), (1, 2)], m).unwrap();
        assert_eq!(e.values(), [u64::MAX, 2, 1, 1]);
        let work = e.update(&[(0, u64::MAX)], Update::Heap).unwrap();
        assert_eq!(work.evaluated, 1);
        assert_eq!(work.scheduled_edges, 0);
    }
}
#[test]
fn exhaustive_small_dags_and_update_sequences() {
    let pairs = [(3, 2), (3, 1), (3, 0), (2, 1), (2, 0), (1, 0)];
    for mask in 0..64_u64 {
        let edges: Vec<_> = pairs
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, &edge)| edge)
            .collect();
        let g = Graph::new(4, &edges).unwrap();
        for o in [Ordering::Fifo, Ordering::Dfs, Ordering::Minimum] {
            valid(4, &edges, &g.topological(o).unwrap());
        }
        for mode in [Update::Full, Update::Dirty, Update::Heap] {
            let mut e = Evaluator::new(g.clone(), vec![0; 4]).unwrap();
            for step in 0..48_usize {
                let edit = [
                    (
                        step % 4,
                        if step % 7 == 0 {
                            u64::MAX
                        } else {
                            u64::try_from(step).unwrap()
                        },
                    ),
                    ((step + 1) % 4, 1),
                ];
                e.update(&edit, mode).unwrap();
                assert_eq!(e.values(), oracle(4, &edges, e.bases()));
            }
        }
    }
}
#[test]
fn long_path_uses_explicit_traversal_state() {
    let n = 16_384;
    let edges: Vec<_> = (1..n).map(|i| (i, i - 1)).collect();
    let g = Graph::new(n, &edges).unwrap();
    valid(n, &edges, &g.topological(Ordering::Dfs).unwrap());
    let mut e = Evaluator::new(g, vec![1; n]).unwrap();
    e.update(&[(n - 1, 2)], Update::Heap).unwrap();
    assert_eq!(e.values(), oracle(n, &edges, e.bases()));
    assert_eq!(e.values()[0], 16_385);
}

#[test]
fn every_small_insertion_matches_independent_reachability() {
    let pairs = [(3, 2), (3, 1), (3, 0), (2, 1), (2, 0), (1, 0)];
    for mask in 0..64_u64 {
        let edges: Vec<_> = pairs
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, &edge)| edge)
            .collect();
        let g = Graph::new(4, &edges).unwrap();
        let mut reach = [[false; 4]; 4];
        for &(u, v) in &edges {
            reach[u][v] = true;
        }
        for k in 0..4 {
            for u in 0..4 {
                for v in 0..4 {
                    reach[u][v] |= reach[u][k] && reach[k][v];
                }
            }
        }
        for &u in g.order() {
            for &v in g.order() {
                let mut candidate = g.clone();
                let result = candidate.insert(u, v);
                if edges.contains(&(u, v)) {
                    assert_eq!(result, Err(Error::Duplicate));
                    assert_eq!(candidate, g);
                } else if u == v || reach[v][u] {
                    assert_eq!(result, Err(Error::Cycle));
                    assert_eq!(candidate, g);
                } else {
                    assert_eq!(result, Ok(()));
                    let mut next = edges.clone();
                    next.push((u, v));
                    valid(4, &next, candidate.order());
                }
            }
        }
    }
}
