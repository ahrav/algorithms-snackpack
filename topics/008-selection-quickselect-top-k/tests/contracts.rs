//! Independent-oracle contracts and partition regressions.
use std::collections::BTreeMap;
use top_k_portfolio::{CANDIDATES, select_three_way};

// Independent ordered-count oracle. Does not partition, heapify, or use slice sort.
fn oracle(input: &[i64], k: usize) -> Option<Vec<i64>> {
    if k > input.len() {
        return None;
    }
    let mut counts = BTreeMap::new();
    for &value in input {
        *counts.entry(value).or_insert(0_usize) += 1;
    }
    Some(
        counts
            .into_iter()
            .flat_map(|(v, n)| std::iter::repeat_n(v, n))
            .take(k)
            .collect(),
    )
}

fn check(input: &[i64]) {
    let original = input.to_vec();
    for k in 0..=input.len() + 1 {
        let expected = oracle(input, k);
        for (name, f) in CANDIDATES {
            assert_eq!(f(input, k), expected, "{name}, k={k}, input={input:?}");
            assert_eq!(input, original);
        }
    }
    for rank in 0..input.len() {
        let mut a = input.to_vec();
        let _ = select_three_way(&mut a, rank).unwrap();
        let nth = a[rank];
        assert!(a[..rank].iter().all(|&x| x <= nth));
        assert!(a[rank + 1..].iter().all(|&x| x >= nth));
        assert_eq!(oracle(&a, a.len()), oracle(input, input.len()));
        assert_eq!(Some(nth), oracle(input, rank + 1).unwrap().last().copied());
    }
}

#[test]
fn exhaustive_duplicates_and_boundaries() {
    for length in 0_u32..=7 {
        for code in 0..3_usize.pow(length) {
            let mut value = code;
            let a: Vec<i64> = (0..length)
                .map(|_| {
                    let x = match value % 3 {
                        0 => -1,
                        1 => 0,
                        _ => 1,
                    };
                    value /= 3;
                    x
                })
                .collect();
            check(&a);
        }
    }
}

#[test]
fn integer_extremes_and_running_example() {
    check(&[9, 1, 7, 3, 3, 8, 2, 6]);
    check(&[i64::MAX, i64::MIN, 0, i64::MIN, i64::MAX, -1, 1]);
    for (name, f) in CANDIDATES {
        assert_eq!(
            f(&[9, 1, 7, 3, 3, 8, 2, 6], 3),
            Some(vec![1, 2, 3]),
            "{name}"
        );
        assert_eq!(f(&[1], usize::MAX), None);
    }
}

#[test]
fn seeded_shapes() {
    let mut state = 0x91a0_8200_8173_u64;
    for n in [1, 2, 3, 8, 31, 128, 1024] {
        for _ in 0..20 {
            let a: Vec<i64> = (0..n)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    i64::from_ne_bytes(state.to_ne_bytes())
                })
                .collect();
            for k in [0, 1, n / 2, n - 1, n, n + 1] {
                let expected = oracle(&a, k);
                for (name, f) in CANDIDATES {
                    assert_eq!(f(&a, k), expected, "{name}");
                }
            }
        }
    }
}

#[test]
fn equal_band_is_one_partition() {
    let mut a = vec![5; 1024];
    let s = select_three_way(&mut a, 511).unwrap();
    assert_eq!(s.partitions, 1);
    assert_eq!(s.classified, 1024);
    assert_eq!(s.swaps, 0);
    assert!(!s.fallback);
}

#[test]
fn invalid_rank_does_not_mutate() {
    let mut a = [3, 1, 2];
    for rank in [3, 4, usize::MAX] {
        assert_eq!(select_three_way(&mut a, rank), None);
        assert_eq!(a, [3, 1, 2]);
    }
    assert_eq!(select_three_way(&mut [], 0), None);
    assert_eq!(select_three_way(&mut [], usize::MAX), None);
}

#[test]
fn organ_pipe_scan_budget_and_fallback() {
    let a: Vec<i64> = (0..4096).map(|i| i64::from(i.min(4095 - i))).collect();
    let expected = oracle(&a, 2048).unwrap();
    let mut b = a.clone();
    let s = select_three_way(&mut b, 2047).unwrap();
    assert!(s.classified <= a.len() * 8);
    assert!(
        s.fallback,
        "adversarial organ pipe must exercise the fallback"
    );
    assert_eq!(b[2047], expected[2047]);
    assert!(b[..2047].iter().all(|&x| x <= b[2047]));
    assert!(b[2048..].iter().all(|&x| x >= b[2047]));
    assert_eq!(oracle(&b, b.len()), oracle(&a, a.len()));
}
