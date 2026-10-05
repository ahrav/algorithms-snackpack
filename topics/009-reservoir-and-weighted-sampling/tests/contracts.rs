//! Contract, independent-oracle and bounded distribution checks.
#![allow(clippy::cast_precision_loss)]
use sampling_lab::{
    Key, Rng, Uniform, combine, open_word, oracle, uniform, weighted_heap, weighted_keys,
    weighted_sort,
};
#[test]
fn extremes_empty_invalid_and_endpoint() {
    assert!(open_word(0) > 0.0);
    assert!(open_word(u64::MAX) < 1.0);
    for w in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(weighted_sort(&[w], 0, 0).is_none());
        assert!(weighted_heap(&[w], 0, 0).is_none());
    }
    let w = [f64::from_bits(1), f64::MIN_POSITIVE, f64::MAX, 1.0];
    let keys = weighted_keys(&w, 0).unwrap();
    assert!(keys.iter().all(|x| x.score.is_finite()));
    for k in 0..=6 {
        assert_eq!(weighted_sort(&w, k, 0), weighted_heap(&w, k, 0));
    }
    assert_eq!(uniform(0, 5, 0, Uniform::R), Vec::<usize>::new());
}
#[test]
fn independent_priority_oracles() {
    for n in [0, 1, 2, 8, 31, 128] {
        for seed in 0..20 {
            for k in [0, 1, 3, n / 2, n, n + 2] {
                let w: Vec<_> = (0..n)
                    .map(|i| if i % 4 == 0 { 1000.0 } else { 1.0 })
                    .collect();
                let expected = oracle(&weighted_keys(&w, seed).unwrap(), k);
                assert_eq!(weighted_sort(&w, k, seed).unwrap(), expected);
                assert_eq!(weighted_heap(&w, k, seed).unwrap(), expected);
                let mut rng = Rng::new(seed);
                let mut pool: Vec<_> = (0..n).map(|i| (rng.word(), i)).collect();
                let mut expected = Vec::new();
                for _ in 0..k.min(n) {
                    let best = (0..pool.len()).min_by_key(|&i| pool[i]).unwrap();
                    expected.push(pool.swap_remove(best).1);
                }
                expected.sort_unstable();
                for m in [Uniform::PrioritySort, Uniform::PriorityHeap] {
                    assert_eq!(uniform(n, k, seed, m), expected);
                }
            }
        }
    }
}
#[test]
fn subset_and_seed_contracts() {
    for m in [
        Uniform::R,
        Uniform::X,
        Uniform::PrioritySort,
        Uniform::PriorityHeap,
    ] {
        for n in [0, 1, 2, 32, 4096] {
            for k in [0, 1, 3, n / 2, n, n + 1] {
                for seed in 0..10 {
                    let s = uniform(n, k, seed, m);
                    assert_eq!(s.len(), n.min(k));
                    assert!(s.windows(2).all(|x| x[0] < x[1]));
                    assert!(s.iter().all(|&x| x < n));
                    assert_eq!(s, uniform(n, k, seed, m));
                }
            }
        }
    }
    let mut r = Rng::new(0);
    assert_eq!(r.word(), 0xe220_a839_7b1d_cdaf);
}
#[test]
fn exact_r_all_subset_probabilities() {
    // Independent enumeration: bounds 3,4,5 give 60 equal paths, 10 subsets.
    let mut counts = std::collections::BTreeMap::new();
    for a in 0..3 {
        for b in 0..4 {
            for c in 0..5 {
                let mut sample = vec![0, 1];
                for (id, draw) in [(2, a), (3, b), (4, c)] {
                    if draw < 2 {
                        sample[draw] = id;
                    }
                }
                sample.sort_unstable();
                *counts.entry(sample).or_insert(0) += 1;
            }
        }
    }
    assert_eq!(counts.len(), 10);
    assert!(counts.values().all(|&x| x == 6));
}
#[test]
fn x_survival_matches_independent_exact_product() {
    // For t=4,k=2, P(skip at least s)=12/((4+s)*(3+s)).
    let mut survival = 1.0;
    for s in 1..100 {
        survival *= f64::from(s + 2) / f64::from(s + 4);
        let exact = 12.0 / f64::from((s + 4) * (s + 3));
        assert!((survival - exact).abs() < 1e-14);
    }
}
#[test]
fn bounded_distribution_diagnostic() {
    // Diagnostic, not a proof of independence or finite arithmetic exactness.
    for m in [
        Uniform::R,
        Uniform::X,
        Uniform::PrioritySort,
        Uniform::PriorityHeap,
    ] {
        let mut counts = [0; 10];
        let mut inclusion = [0; 5];
        for seed in 0..20000 {
            let s = uniform(5, 2, seed, m);
            inclusion[s[0]] += 1;
            inclusion[s[1]] += 1;
            let index = (0..s[0]).map(|i| 4 - i).sum::<usize>() + s[1] - s[0] - 1;
            counts[index] += 1;
        }
        assert!(counts.iter().all(|&x| (1700..2300).contains(&x)));
        assert!(inclusion.iter().all(|&x| (7600..8400).contains(&x)));
    }
}
#[test]
fn unequal_shards_and_ties_preserve_retained_keys() {
    let weights: Vec<_> = (0..101).map(|i| f64::from(i % 7 + 1)).collect();
    let keys = weighted_keys(&weights, 42).unwrap();
    for k in [0, 1, 3, 17, 101, 105] {
        let shards: Vec<_> = [&keys[..1], &keys[1..9], &keys[9..]]
            .iter()
            .map(|part| oracle(part, k))
            .collect();
        assert_eq!(combine(&shards, k), oracle(&keys, k));
    }
    let ties: Vec<_> = (0..9).rev().map(|id| Key { id, score: 0.0 }).collect();
    assert_eq!(combine(std::slice::from_ref(&ties), 3), oracle(&ties, 3));
    assert_eq!(
        combine(&[oracle(&keys[..50], 3), oracle(&keys[50..], 3)], 3),
        oracle(&keys, 3)
    );
}

#[test]
fn direct_clock_oracle_and_weighted_frequency() {
    let weights = [1.0, 1.0, 1.0, 2.0];
    let mut heavy = 0;
    for seed in 0..20000 {
        let mut rng = Rng::new(seed);
        let mut clocks: Vec<_> = weights
            .iter()
            .enumerate()
            .map(|(id, w)| (-rng.open().ln() / w, id))
            .collect();
        clocks.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut expected: Vec<_> = clocks.iter().take(2).map(|x| x.1).collect();
        expected.sort_unstable();
        let actual = weighted_heap(&weights, 2, seed).unwrap();
        assert_eq!(actual.iter().map(|x| x.id).collect::<Vec<_>>(), expected);
        heavy += usize::from(actual.iter().any(|x| x.id == 3));
    }
    // Sequential-weight heavy inclusion is 0.7, distinct from proportional 0.8.
    assert!((13600..14400).contains(&heavy));
}
