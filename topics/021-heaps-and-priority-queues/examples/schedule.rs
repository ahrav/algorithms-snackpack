//! A keyed scheduler preserves FIFO birth across updates.
use topic021_heaps_and_priority_queues::{Kind, Queue};
fn main() {
    let mut queue = Queue::from_priorities(Kind::Four, &[5, 3, 3, 8]);
    queue.set(0, 1).expect("valid ID");
    let ids: Vec<_> = std::iter::from_fn(|| queue.pop()).map(|e| e.id).collect();
    assert_eq!(ids, [0, 1, 2, 3]);
    println!("pop order: A B C D");
}
