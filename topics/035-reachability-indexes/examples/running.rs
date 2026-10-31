//! Six-vertex running example.
use topic035::{Closure, Dag, Intervals, Scratch, search};
fn main() {
    let g = Dag::new(vec![
        vec![1, 2],
        vec![3],
        vec![3, 4],
        vec![5],
        vec![5],
        vec![],
    ])
    .unwrap();
    let f = Intervals::new(&g);
    let c = Closure::new(&g, 1024).unwrap();
    let mut s = Scratch::new(g.len());
    for u in 0..g.len() {
        let expected = g.oracle(u);
        for (v, &e) in expected.iter().enumerate() {
            assert_eq!(search(&g, &mut s, u, v, None), e);
            assert_eq!(search(&g, &mut s, u, v, Some(&f)), e);
            assert_eq!(c.reaches(u, v), e);
        }
    }
    println!(
        "36 query pairs agree; 0->5=true; 1->4=false; closure={} bytes; intervals={} bytes; empty={}",
        c.bytes(),
        f.bytes(),
        g.is_empty()
    );
}
