//! Runnable merge and join contract checks.
use merge_lab::{Row, count_join, linear_merge, parallel_merge, replay_join};

fn main() {
    let a = [(1, 1), (5, 2), (5, 3), (9, 4)].map(|(key, id)| Row { key, id });
    let b = [(2, 10), (5, 11), (5, 12), (5, 13), (8, 14)].map(|(key, id)| Row { key, id });
    assert_eq!(linear_merge(&a, &b), parallel_merge(&a, &b, 2));
    assert_eq!(count_join(&a, &b), 6);
    println!("merge: {:?}", linear_merge(&a, &b));
    println!("join: {:?}", replay_join(&a, &b));
    println!("row_bytes: {}", std::mem::size_of::<Row>());
}
