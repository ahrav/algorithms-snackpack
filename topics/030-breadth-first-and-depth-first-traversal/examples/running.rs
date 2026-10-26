//! The six-vertex running example.
use topic030_traversal::{Graph, UNREACHABLE, bfs_deque, dfs_frames};
fn main() {
    let g = Graph::new(vec![
        vec![1, 2],
        vec![3],
        vec![3],
        vec![1, 4],
        vec![],
        vec![],
    ])
    .expect("valid graph");
    let d = bfs_deque(&g, &[0]);
    assert_eq!(d, [0, 1, 1, 2, 3, UNREACHABLE]);
    let dfs = dfs_frames(&g, &[0]);
    assert_eq!(dfs.preorder, [0, 1, 3, 4, 2]);
    assert_eq!(dfs.finish, [4, 3, 1, 2, 0]);
    assert!(dfs.cyclic);
    println!(
        "distances={d:?} preorder={:?} finish={:?} cyclic={}",
        dfs.preorder, dfs.finish, dfs.cyclic
    );
}
