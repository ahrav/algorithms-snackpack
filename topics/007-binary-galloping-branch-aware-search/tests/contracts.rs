//! Independent-model agreement and boundary-discriminating checks.
use lower_bound_portfolio::{CANDIDATES, oracle};

fn check(a: &[u64], key: u64) {
    let expected = oracle(a, key);
    for (name, f) in CANDIDATES {
        let p = f(a, key);
        assert_eq!(p, expected, "{name}: {a:?}, key={key}");
        assert!(a[..p].iter().all(|&v| v < key));
        assert!(a[p..].iter().all(|&v| v >= key));
    }
}

#[test]
fn empty_singletons_duplicates_and_extrema() {
    for a in [
        vec![],
        vec![0],
        vec![u64::MAX],
        vec![4, 4],
        vec![4; 17],
        vec![0, 0, 4, 4, 4, u64::MAX, u64::MAX],
    ] {
        for key in [0, 1, 3, 4, 5, u64::MAX] {
            check(&a, key);
        }
    }
}

#[test]
fn running_example_first_duplicate() {
    let a = [2, 4, 4, 4, 9, 13, 18, 21];
    for key in 0..=22 {
        check(&a, key);
    }
}

fn enumerate(a: &mut Vec<u64>, remaining: usize, minimum: u64, count: &mut usize) {
    if remaining == 0 {
        *count += 1;
        for key in [0, 1, 2, 3, 4, u64::MAX] {
            check(a, key);
        }
        return;
    }
    for v in minimum..=3 {
        a.push(v);
        enumerate(a, remaining - 1, v, count);
        a.pop();
    }
}

#[test]
fn exhaustive_sorted_arrays() {
    let mut count = 0;
    for n in 0..=10 {
        enumerate(&mut Vec::new(), n, 0, &mut count);
    }
    assert_eq!(count, 1001);
}

#[test]
fn power_of_two_brackets_and_tails() {
    for n in [2, 3, 7, 8, 9, 15, 16, 17, 127, 128, 129, 1023, 1024, 1025] {
        let a: Vec<u64> = (0..n).map(|i| i * 2).collect();
        for key in 0..=n * 2 {
            check(&a, key);
        }
    }
}

#[test]
fn seeded_irregular_and_duplicate_runs() {
    let mut seed = 0x73af_1589_d382_bc91u64;
    for case in 0..256usize {
        let n = case * 17;
        let mut a = Vec::with_capacity(n);
        for _ in 0..n {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            a.push(seed % 1024);
        }
        a.sort_unstable();
        for key in [0, 1, 511, 512, 1023, 1024, u64::MAX] {
            check(&a, key);
        }
    }
}
