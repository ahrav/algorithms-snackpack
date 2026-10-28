//! Run the five-vertex dependency update.
use topic_032_topological_order_and_incremental_dags::{Evaluator, Graph, Update};
fn main() {
    let g = Graph::new(5, &[(0, 2), (1, 2), (2, 3), (2, 4)]).unwrap();
    println!("order: {:?}", g.order());
    let mut e = Evaluator::new(g, vec![2, 3, 0, 1, 2]).unwrap();
    println!("before: {:?}", e.values());
    let work = e.update(&[(0, 4)], Update::Heap).unwrap();
    println!("after: {:?}; recomputed {}", e.values(), work.evaluated);
    assert_eq!(e.values(), [4, 3, 7, 8, 9]);
}
