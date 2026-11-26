//! Snapshot contracts and independent numerical checks.

use associative_reductions_and_deferred_merges::{Affine, History, Policy, reference};

const POLICIES: [Policy; 3] = [Policy::Raw, Policy::Chunked, Policy::Compact];

#[test]
fn retained_boundaries_survive_reduction() {
    for policy in POLICIES {
        let history = History::build(&[10, 10, 10, 30, 80], &[2, 4, 5], policy).unwrap();
        assert_eq!(history.read(0), Some((0, 0)));
        assert_eq!(history.read(1), Some((20, 2)));
        assert_eq!(history.read(2), Some((60, 4)));
        assert_eq!(history.read(3), Some((140, 5)));
        assert_eq!(history.read(4), None);
    }
}

#[test]
fn rejecting_bad_boundaries_and_accepting_empty() {
    for policy in POLICIES {
        for invalid in [&[][..], &[0, 2], &[2, 2], &[3, 2], &[3], &[1]] {
            assert!(History::build(&[7, 9], invalid, policy).is_none());
        }
        assert!(History::build(&[], &[1], policy).is_none());
        let empty = History::build(&[], &[], policy).unwrap();
        assert_eq!(empty.read(0), Some((0, 0)));
        assert_eq!(empty.read(1), None);
        assert_eq!(empty.reserved(), (0, 0));
    }
}

#[test]
fn generated_prefixes_agree_with_wide_raw_oracle() {
    let samples = [i32::MIN, i32::MAX, 0, -90, 841, 20, -113];
    for seed in 0..128 {
        let len = 65 + seed;
        let values: Vec<_> = (0..len)
            .map(|i| samples[(i * i + seed * i + 3 * seed) % samples.len()])
            .collect();
        let step = [1, 2, 31, 32, 33, 64, 65, 127][seed % 8];
        let ends: Vec<_> = (step..len).step_by(step).chain([len]).collect();
        for policy in POLICIES {
            let history = History::build(&values, &ends, policy).unwrap();
            for (i, &end) in ends.iter().enumerate() {
                assert_eq!(history.read(i + 1), reference(&values, end), "seed {seed}");
            }
        }
    }
}

#[test]
fn affine_grouping_preserves_replay_order_and_wrap() {
    let a = Affine {
        multiplier: 2,
        increment: 0,
    };
    let b = Affine {
        multiplier: 1,
        increment: 3,
    };
    let c = Affine {
        multiplier: u64::MAX,
        increment: u64::MAX - 1,
    };
    assert_eq!(a.then(b).apply(10), 23);
    assert_eq!(b.then(a).apply(10), 26);
    for x in [0, 1, 10, u64::MAX / 2, u64::MAX] {
        let expected = c.apply(b.apply(a.apply(x)));
        assert_eq!(a.then(b).then(c).apply(x), expected);
        assert_eq!(a.then(b.then(c)).apply(x), expected);
    }
}
