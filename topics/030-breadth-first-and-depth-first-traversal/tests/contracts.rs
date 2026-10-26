//! Bounded independent-oracle traversal contracts.
#[path = "../src/lib.rs"]
#[allow(dead_code)]
mod traversal;
use traversal::{
    Csr, Dfs, Graph, UNREACHABLE, bfs_csr, bfs_deque, bfs_layers, bfs_vec, dfs_events, dfs_frames,
    distance_oracle,
};

fn recursive(g: &Graph, roots: &[usize]) -> Dfs {
    fn visit(g: &Graph, u: usize, colors: &mut [u8], out: &mut Dfs) {
        colors[u] = 1;
        out.preorder.push(u);
        for &v in g.neighbors(u) {
            match colors[v] {
                0 => visit(g, v, colors, out),
                1 => out.cyclic = true,
                _ => {}
            }
        }
        colors[u] = 2;
        out.finish.push(u);
    }
    let mut out = Dfs {
        preorder: vec![],
        finish: vec![],
        cyclic: false,
    };
    let mut colors = vec![0; g.len()];
    for &r in roots {
        if r < g.len() && colors[r] == 0 {
            visit(g, r, &mut colors, &mut out);
        }
    }
    out
}
fn check(g: &Graph, sources: &[usize]) {
    let expected = distance_oracle(g, sources);
    let csr = Csr::new(g);
    for got in [
        bfs_deque(g, sources),
        bfs_vec(g, sources),
        bfs_csr(&csr, sources),
        bfs_layers(&csr, sources),
    ] {
        assert_eq!(got, expected);
    }
    let reference = recursive(g, sources);
    assert_eq!(dfs_frames(g, sources), reference);
    assert_eq!(dfs_events(g, sources), reference);
}
#[test]
fn exhaustive_three_vertices() {
    for bits in 0..512 {
        let mut adj = vec![vec![]; 3];
        for (u, row) in adj.iter_mut().enumerate() {
            for v in 0..3 {
                if bits & (1 << (u * 3 + v)) != 0 {
                    row.push(v);
                }
            }
        }
        let g = Graph::new(adj).unwrap();
        for roots in [
            &[][..],
            &[0][..],
            &[1][..],
            &[2][..],
            &[0, 2, 0, 9][..],
            &[0, 1, 2][..],
        ] {
            check(&g, roots);
        }
    }
}
#[test]
fn empty_invalid_duplicates_and_running_example() {
    assert!(Graph::new(vec![vec![1]]).is_err());
    check(&Graph::new(vec![]).unwrap(), &[0]);
    let g = Graph::new(vec![
        vec![1, 2],
        vec![3],
        vec![3],
        vec![1, 4],
        vec![],
        vec![],
    ])
    .unwrap();
    check(&g, &[0]);
    assert_eq!(bfs_vec(&g, &[0]), vec![0, 1, 1, 2, 3, UNREACHABLE]);
    assert_eq!(
        dfs_frames(&g, &[0]),
        Dfs {
            preorder: vec![0, 1, 3, 4, 2],
            finish: vec![4, 3, 1, 2, 0],
            cyclic: true
        }
    );
    check(
        &Graph::new(vec![vec![0, 1, 1], vec![1], vec![]]).unwrap(),
        &[0, 0, 99],
    );
}
#[test]
fn seeded_graphs() {
    let mut seed = 42_u64;
    for n in 1..=80 {
        let mut adj = vec![vec![]; n];
        for row in &mut adj {
            for _ in 0..5 {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let v = usize::try_from(seed >> 32).unwrap() % n;
                row.push(v);
            }
        }
        check(&Graph::new(adj).unwrap(), &[0, n - 1]);
    }
}
#[test]
fn deep_path_and_cross_edge_are_distinct_from_back_edge() {
    let n = 50_000;
    let mut adj = vec![vec![]; n];
    for (i, row) in adj.iter_mut().enumerate().take(n - 1) {
        row.push(i + 1);
    }
    let g = Graph::new(adj).unwrap();
    let frames = dfs_frames(&g, &[0]);
    assert!(!frames.cyclic);
    assert_eq!(frames.preorder.len(), n);
    assert_eq!(frames.finish[0], n - 1);
    assert_eq!(dfs_events(&g, &[0]), frames);
    let diamond = Graph::new(vec![vec![1, 2], vec![3], vec![3], vec![]]).unwrap();
    assert!(!dfs_frames(&diamond, &[0]).cyclic);
    check(&diamond, &[0]);
}
