//! Executable dominance contract checks.
use topic034::{bitsets, chk, insert, parents};
fn main() {
    let mut graph = vec![
        vec![1],
        vec![2, 3],
        vec![4],
        vec![4],
        vec![5],
        vec![],
        vec![4],
    ];
    let mut d = bitsets(&graph, 0).0;
    println!("before {:?}", chk(&graph, 0).0);
    insert(&mut graph, 0, &mut d, 0, 4);
    println!("after  {:?}", parents(&d, 0));
    assert_eq!(parents(&d, 0), chk(&graph, 0).0);
}
