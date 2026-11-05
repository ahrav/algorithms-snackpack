//! Boundary and exact-match contract checks.
use topic_040_chunking::{Config, Policy, boundaries, exact_matches, fingerprint, fixture};
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
}
