//! Executable dominance contract checks.
use topic034::{Graph, bitsets, chk, expand, insert, oracle, parents, workload};
fn check(g: &Graph, root: usize) {
    let expected = oracle(g, root);
    let d = bitsets(g, root).0;
    assert_eq!(d, expected);
    let p = chk(g, root).0;
    assert_eq!(expand(&p), expected);
    assert_eq!(parents(&d, root), p);
}
#[test]
fn exhaustive_three_vertices_with_every_root_and_insertion() {
    for mask in 0_u32..512 {
        let mut g = vec![vec![]; 3];
        for (u, row) in g.iter_mut().enumerate() {
            for v in 0..3 {
                if mask & (1 << (u * 3 + v)) != 0 {
                    row.push(v);
                }
            }
        }
        for root in 0..3 {
            check(&g, root);
            for u in 0..3 {
                for v in 0..3 {
                    let mut h = g.clone();
                    let mut d = bitsets(&h, root).0;
                    insert(&mut h, root, &mut d, u, v);
                    assert_eq!(d, oracle(&h, root));
                    check(&h, root);
                }
            }
        }
    }
}
#[test]
fn boundaries_and_word_boundaries() {
    for n in [0, 1, 2, 63, 64, 65, 127, 129] {
        for shape in ["chain", "joins", "cycle", "unreachable", "permuted"] {
            check(&workload(shape, n), 0);
        }
    }
    check(&vec![vec![1, 1], vec![1, 2], vec![0], vec![2]], 0);
    check(&vec![vec![], vec![0]], 9);
}
#[test]
fn update_sequence_and_deletion_requires_reset() {
    let mut g = workload("chain", 65);
    let mut d = bitsets(&g, 0).0;
    for (u, v) in [(0, 32), (10, 64), (64, 10), (64, 64), (0, 64)] {
        insert(&mut g, 0, &mut d, u, v);
        assert_eq!(d, oracle(&g, 0));
    }
    let mut h = vec![vec![1, 2], vec![2], vec![3], vec![2]];
    let stale = bitsets(&h, 0).0;
    h[0].retain(|&v| v != 2);
    assert_eq!((stale[1][0] & stale[3][0]) | (1 << 2), stale[2][0]);
    assert_eq!(stale[2][0] | (1 << 3), stale[3][0]);
    let fresh = bitsets(&h, 0).0;
    assert_ne!(stale, fresh);
    check(&h, 0);
}
#[test]
fn insert_rejects_invalid_endpoints_before_mutating() {
    let mut g: Graph = vec![vec![1], vec![], vec![]];
    let before = g.clone();
    let mut d = bitsets(&g, 0).0;
    let sets = d.clone();
    for (u, v) in [(2, 3), (0, 3), (3, 0)] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            insert(&mut g, 0, &mut d, u, v);
        }));
        assert!(r.is_err(), "insert({u}, {v}) must panic");
        assert_eq!(g, before, "insert({u}, {v}) mutated the graph");
        assert_eq!(d, sets, "insert({u}, {v}) mutated the sets");
    }
}
