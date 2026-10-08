//! External API contract regressions.
use radix_partitioning::{Digit, Row, cycles_in_place, scatter};
#[test]
fn caller_owned_in_place_contract() {
    let mut rows: Vec<_> = [3, 0, 2, 1, 0, 3]
        .into_iter()
        .enumerate()
        .map(|(i, key)| Row {
            key,
            id: i as u64,
            payload: [key + 99],
        })
        .collect();
    let original = rows.clone();
    let bounds = cycles_in_place(&mut rows, Digit::new(0, 2));
    assert_eq!(bounds, [0, 2, 3, 4, 6]);
    for b in 0..4 {
        assert!(
            rows[bounds[b]..bounds[b + 1]]
                .iter()
                .all(|r| r.key == b as u64)
        );
    }
    rows.sort_by_key(|r| r.id);
    assert_eq!(rows, original);
}
#[test]
fn stable_digit_order_does_not_sort_keys() {
    let rows = [
        Row {
            key: 4,
            id: 0,
            payload: [],
        },
        Row {
            key: 0,
            id: 1,
            payload: [],
        },
    ];
    assert_eq!(scatter(&rows, Digit::new(0, 2)).rows, rows);
}
