//! Running example for stable digit partitioning.
use radix_partitioning::{Digit, Row, scatter};
fn main() {
    let input: Vec<_> = [6, 1, 7, 4, 1, 2, 5, 0]
        .into_iter()
        .enumerate()
        .map(|(i, key)| Row {
            key,
            id: i as u64,
            payload: [],
        })
        .collect();
    let result = scatter(&input, Digit::new(0, 2));
    assert_eq!(result.bounds, [0, 2, 5, 7, 8]);
    assert_eq!(
        result.rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        [3, 7, 1, 4, 6, 0, 5, 2]
    );
    println!(
        "bounds {:?}; source identities {:?}",
        result.bounds,
        result.rows.iter().map(|r| r.id).collect::<Vec<_>>()
    );
}
