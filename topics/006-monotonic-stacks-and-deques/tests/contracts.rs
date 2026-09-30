//! Independent-model and boundary checks over a finite contract.
use monotonic_stacks_deques::{
    next_greater_scan, next_greater_stack, next_greater_stack_counted, window_blocks,
    window_blocks_counted, window_deque, window_deque_counted, window_scan,
};

#[test]
fn running_example_and_counts() {
    let a = [5, 3, 3, 4, 2, 6, 1, 6];
    assert_eq!(window_scan(&a, 3), Some(vec![0, 3, 3, 5, 5, 7]));
    let (out, counts) = window_deque_counted(&a, 3).unwrap();
    assert_eq!(Some(out), window_scan(&a, 3));
    assert_eq!(counts.comparisons, 10);
    assert_eq!(counts.back_pops, 6);
    let expected = vec![
        Some(5),
        Some(3),
        Some(3),
        Some(5),
        Some(5),
        None,
        Some(7),
        None,
    ];
    assert_eq!(next_greater_stack(&a), expected);
    assert_eq!(next_greater_stack_counted(&a).1.comparisons, 12);
}

#[test]
fn duplicates_have_distinct_contracts() {
    assert_eq!(next_greater_stack(&[3, 3, 4]), vec![Some(2), Some(2), None]);
    assert_eq!(window_deque(&[6, 1, 6], 3), Some(vec![2]));
    assert_eq!(window_blocks(&[6, 1, 6], 3), Some(vec![2]));
    assert_eq!(window_deque(&[4, 4, 4, 4], 2), Some(vec![1, 2, 3]));
}

#[test]
fn expiration_and_retained_second_best() {
    let a = [9, 8, 7, 6];
    assert_eq!(window_deque(&a, 2), Some(vec![0, 1, 2]));
    assert_eq!(window_deque_counted(&a, 2).unwrap().1.peak, 2);
    assert_eq!(window_deque_counted(&a, 2).unwrap().1.front_pops, 2);
}

#[test]
fn invalid_widths_and_extreme_values() {
    for a in [&[][..], &[i64::MIN, 0, i64::MAX][..]] {
        for w in [0, a.len() + 1, usize::MAX] {
            assert_eq!(window_scan(a, w), None);
            assert_eq!(window_deque(a, w), None);
            assert_eq!(window_blocks(a, w), None);
        }
    }
    assert_eq!(next_greater_stack(&[]), vec![]);
    assert_eq!(window_blocks(&[i64::MIN, i64::MAX], 1), Some(vec![0, 1]));
}

#[test]
fn removal_burst_is_not_constant() {
    let mut a: Vec<i64> = (1..=64).rev().collect();
    a.push(100);
    let c = window_deque_counted(&a, 64).unwrap().1;
    assert_eq!(c.max_burst, 63);
    assert_eq!(c.peak, 64);
    assert_eq!(next_greater_stack_counted(&a).1.max_burst, 64);
}

#[test]
fn exhaustive_small_arrays_agree_with_independent_oracles() {
    let mut arrays = 0;
    let mut windows = 0;
    for len in 0..=7_u32 {
        for mut word in 0..3_usize.pow(len) {
            let mut a = vec![0; len as usize];
            for value in &mut a {
                *value = [-1, 0, 1][word % 3];
                word /= 3;
            }
            arrays += 1;
            assert_eq!(next_greater_stack(&a), next_greater_scan(&a));
            for w in 0..=a.len() + 1 {
                windows += 1;
                let oracle = window_scan(&a, w);
                assert_eq!(window_deque(&a, w), oracle, "{a:?}, width {w}");
                assert_eq!(window_blocks(&a, w), oracle, "{a:?}, width {w}");
                if let Some((out, counts)) = window_deque_counted(&a, w) {
                    assert_eq!(Some(out), oracle);
                    assert!(counts.peak <= w);
                    assert_eq!(counts.pushes, a.len());
                    assert!(counts.back_pops + counts.front_pops <= a.len());
                    assert!(counts.comparisons <= a.len().saturating_mul(2));
                    assert_eq!(window_blocks_counted(&a, w).unwrap().0, oracle.unwrap());
                }
            }
        }
    }
    assert_eq!(arrays, 3_280);
    assert_eq!(windows, 27_884);
}
