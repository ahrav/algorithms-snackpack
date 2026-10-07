//! Runnable merge and join contract checks.
use merge_lab::{Row, co_rank, count_join, replay_join};

#[test]
fn merge_partitions_do_not_define_join_partitions() {
    let a = [Row { key: 5, id: 2 }, Row { key: 5, id: 3 }];
    let b = [Row { key: 5, id: 11 }, Row { key: 5, id: 12 }];
    let (i, j) = co_rank(2, &a, &b);
    assert_eq!((i, j), (2, 0));
    assert_eq!(replay_join(&a, &b).len(), 4);
    assert_eq!(
        replay_join(&a[..i], &b[..j]).len() + replay_join(&a[i..], &b[j..]).len(),
        0
    );
}

#[test]
fn count_can_budget_output_before_materializing() {
    let a = vec![Row { key: 5, id: 2 }; 512];
    let b = vec![Row { key: 5, id: 11 }; 512];
    assert_eq!(count_join(&a, &b), 262_144);
    let budget = 100_000;
    assert!(count_join(&a, &b) > budget);
}
