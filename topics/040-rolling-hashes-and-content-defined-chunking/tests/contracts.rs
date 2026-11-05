//! Boundary and exact-match contract checks.
use topic_040_chunking::{
    Config, Policy, boundaries, exact_matches, fingerprint, fixture, gear_table,
};
fn gear_reference(
    bytes: &[u8],
    min: usize,
    target: usize,
    max: usize,
    normalized: bool,
) -> Vec<usize> {
    let table = gear_table();
    let mut ends = Vec::new();
    let mut start = 0;
    for end in 1..=bytes.len() {
        let len = end - start;
        let cut = if len >= max {
            true
        } else if len < min {
            false
        } else {
            let h = bytes[start.max(end.saturating_sub(64))..end]
                .iter()
                .rev()
                .enumerate()
                .fold(0_u64, |acc, (j, &b)| {
                    acc.wrapping_add(table[usize::from(b)] << j)
                });
            let mask = if normalized && len < target {
                2 * target - 1
            } else if normalized {
                target / 2 - 1
            } else {
                target - 1
            };
            h & u64::try_from(mask).unwrap() == 0
        };
        if cut {
            ends.push(end);
            start = end;
        }
    }
    if start < bytes.len() {
        ends.push(bytes.len());
    }
    ends
}
#[test]
fn rolling_agrees_with_direct_and_bounds_hold() {
    for window in [1, 8, 32, 64] {
        let c = Config::new(64, 256, 1024, window).unwrap();
        for n in [0, 1, 31, 63, 64, 65, 255, 256, 1023, 1024, 1025, 4096] {
            for shape in ["random", "zero", "periodic"] {
                let x = fixture(n, shape);
                let d = boundaries(&x, c, Policy::Direct);
                assert_eq!(d, boundaries(&x, c, Policy::Rolling));
                for p in [Policy::Rolling, Policy::Gear, Policy::Normalized] {
                    let ends = boundaries(&x, c, p);
                    assert_eq!(ends.last().copied(), (n > 0).then_some(n));
                    let mut start = 0;
                    for &end in &ends {
                        assert!(end > start && end - start <= 1024);
                        assert!(end == n || end - start >= 64);
                        start = end;
                    }
                }
            }
        }
    }
    assert!(Config::new(0, 0, 1, 0).is_none());
    assert!(Config::new(1, 1, 4, 1).is_none());
    assert!(Config::new(8, 255, 1024, 8).is_none());
    assert!(Config::new(8, 256, 128, 8).is_none());
    assert!(Config::new(8, 256, 1024, 9).is_none());
    // Each remaining guard rejects alone: min > target, window == 0, target > usize::MAX / 2.
    assert!(Config::new(512, 256, 1024, 8).is_none());
    assert!(Config::new(0, 256, 1024, 0).is_none());
    let half = 1_usize << (usize::BITS - 1);
    assert!(Config::new(half, half, usize::MAX, 1).is_none());
    assert!(Config::new(half / 2, half / 2, usize::MAX, 1).is_some());
}
#[test]
fn fixed_cuts_at_every_target_multiple_and_emits_the_tail() {
    let c = Config::new(64, 256, 1024, 32).unwrap();
    for n in [0, 1, 255, 256, 257, 1024, 1025, 4096] {
        let x = fixture(n, "random");
        let mut expected: Vec<usize> = (256..=n).step_by(256).collect();
        if n % 256 != 0 {
            expected.push(n);
        }
        assert_eq!(boundaries(&x, c, Policy::Fixed), expected, "n={n}");
    }
}
#[test]
fn gear_policies_agree_with_closed_form_reference_and_reset_at_cuts() {
    // A mask cut leaves at least six zero low bits behind, so leaked state after
    // such a cut sits above every mask bit. A forced cut at max = 64 leaves arbitrary
    // state; a 2-byte minimum then exposes bits 2.. of that state to the 6-bit mask,
    // so the reference comparison detects cross-chunk state leakage.
    let (min, target) = (2, 64);
    for max in [64, 256] {
        let c = Config::new(min, target, max, 2).unwrap();
        for n in [0, 1, 2, 3, 63, 64, 65, 255, 256, 257, 4096] {
            for shape in ["random", "zero", "periodic"] {
                let x = fixture(n, shape);
                assert_eq!(
                    boundaries(&x, c, Policy::Gear),
                    gear_reference(&x, min, target, max, false),
                    "gear n={n} max={max} {shape}"
                );
                assert_eq!(
                    boundaries(&x, c, Policy::Normalized),
                    gear_reference(&x, min, target, max, true),
                    "normalized n={n} max={max} {shape}"
                );
            }
        }
    }
}
#[test]
#[should_panic(expected = "unknown shape")]
fn fixture_rejects_unknown_shape() {
    let _bytes = fixture(1, "period7");
}
#[test]
fn exact_search_agrees_with_byte_oracle_including_dense_hits() {
    for n in 0..80 {
        for m in 0..20 {
            let t = fixture(n, "periodic");
            let p = fixture(m, "periodic");
            let expected: Vec<_> = (0..=n)
                .filter(|&i| t.get(i..i + m) == Some(p.as_slice()))
                .collect();
            assert_eq!(exact_matches(&t, &p), expected);
        }
    }
    assert_eq!(exact_matches(b"aaaaa", b"aaa"), vec![0, 1, 2]);
}
#[test]
fn fingerprint_collision_requires_byte_verification() {
    let a = [0_u8; 65];
    let mut b = a;
    // (257-1)^8 == 2^64, with binomial coefficients fitting signed bytes.
    let plus = [1, 0, 28, 0, 70, 0, 28, 0, 1];
    let minus = [0, 8, 0, 56, 0, 56, 0, 8, 0];
    b[..9].copy_from_slice(&plus);
    let mut c = a;
    c[..9].copy_from_slice(&minus);
    assert_ne!(b, c);
    assert_eq!(fingerprint(&b), fingerprint(&c));
    assert!(exact_matches(&b, &c).is_empty());
    assert_eq!(fingerprint(&plus), fingerprint(&minus));
    // Rolling over the colliding window at offset 3 rejects it by bytes and
    // continues to the true match at offset 14.
    let mut text = vec![0_u8; 3];
    text.extend_from_slice(&plus);
    text.extend_from_slice(&[0, 0]);
    text.extend_from_slice(&minus);
    assert_eq!(exact_matches(&text, &minus), vec![14]);
    assert_eq!(exact_matches(&text, &plus), vec![3]);
}
