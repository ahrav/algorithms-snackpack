use topic033::{Graph, canonical, certify, example, gabow, kosaraju, oracle, tarjan};

fn check(g: &Graph) {
    let expected = oracle(g);
    for f in [tarjan, kosaraju, gabow] {
        let id = f(g);
        assert_eq!(canonical(&id), expected, "graph={g:?}");
        assert!(certify(g, &id));
    }
}

#[test]
fn all_directed_graphs_through_four_vertices() {
    for n in 0..=4 {
        for mask in 0_u64..1 << (n * n) {
            let mut g = vec![vec![]; n];
            for (v, edges) in g.iter_mut().enumerate() {
                for w in 0..n {
                    if mask & (1 << (v * n + w)) != 0 {
                        edges.push(w);
                    }
                }
            }
            check(&g);
        }
    }
}

#[test]
fn active_finished_sibling_and_completed_target() {
    // DFS frame 1 has returned but 1 is unresolved when edge 2->1 is scanned.
    check(&vec![vec![1, 2], vec![0], vec![1]]);
    // 0 is already emitted before the later root's edge into it.
    check(&vec![vec![], vec![0]]);
    check(&example());
    check(&vec![vec![0, 1, 1], vec![0, 0], vec![]]);
}

#[test]
fn certificate_rejects_splits_merges_and_malformed_labels() {
    let g = example();
    assert!(!certify(&g, &[0, 0, 0, 0, 0, 0, 0]));
    assert!(!certify(&g, &[0, 1, 2, 3, 4, 5, 6]));
    assert!(!certify(&g, &[0, 0]));
    assert!(!certify(&g, &[0, 0, 0, 3, 3, 5, 6]));
    assert!(!certify(&g, &[99; 7]));
}

#[test]
fn deep_chain_and_cycle_use_heap_frames() {
    let n = 100_000;
    let mut g: Graph = (0..n)
        .map(|v| if v + 1 < n { vec![v + 1] } else { vec![] })
        .collect();
    for f in [tarjan, kosaraju, gabow] {
        let id = f(&g);
        assert!(certify(&g, &id));
        assert_eq!(*id.iter().max().unwrap(), n - 1);
    }
    g[n - 1].push(0);
    for f in [tarjan, kosaraju, gabow] {
        let id = f(&g);
        assert!(certify(&g, &id));
        assert!(id.iter().all(|&c| c == 0));
    }
}

#[test]
fn graph_edit_rebuild_changes_partition() {
    let mut g = example();
    check(&g);
    g[6].push(1);
    for f in [tarjan, kosaraju, gabow] {
        assert_eq!(canonical(&f(&g)), vec![0; 7]);
    }
    g[6].pop();
    g[4].retain(|&w| w != 3);
    check(&g);
    assert_ne!(tarjan(&g)[3], tarjan(&g)[4]);
}

#[test]
#[should_panic(expected = "valid(g)")]
fn invalid_destination_rejected() {
    let _ = tarjan(&vec![vec![1]]);
}

#[test]
#[should_panic(expected = "dense")]
fn canonical_rejects_labels_outside_vertex_range() {
    let _ = canonical(&[10, 10]);
}
