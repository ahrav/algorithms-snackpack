//! Independent first-match contracts, bounded exhaustive cases, and hostile witnesses.
use topic_039_adaptive_string_search::{Candidate, Finder, oracle};
const CANDIDATES: [Candidate; 5] = [
    Candidate::Direct,
    Candidate::Horspool,
    Candidate::BoyerMoore,
    Candidate::TwoWay,
    Candidate::Adaptive,
];
fn check(t: &[u8], p: &[u8]) {
    let expected = oracle(t, p);
    for c in CANDIDATES {
        let f = Finder::new(p, c);
        assert_eq!(f.find(t), expected, "{c:?} t={t:?} p={p:?}");
        assert_eq!(f.diagnose(t).0, expected);
    }
}
#[test]
fn boundaries_and_first_occurrence() {
    for (t, p, expected) in [
        (b"".as_slice(), b"".as_slice(), Some(0)),
        (b"", b"a", None),
        (b"aaa", b"aa", Some(0)),
        (b"ABAAABCD", b"ABCD", Some(4)),
        (b"\x00\xff\x00", b"\xff\x00", Some(1)),
    ] {
        assert_eq!(oracle(t, p), expected);
        check(t, p);
    }
    assert_ne!(Some(1), oracle(b"aaa", b"aa")); // Reject a last-match oracle mutation.
    assert_ne!(None, oracle(b"xabc", b"abc")); // Reject omission of the final window.
    let all: Vec<u8> = (0..=255).collect();
    for start in 0..256 {
        check(&all, &all[start..]);
    }
}
#[test]
fn exhaustive_binary_inputs() {
    for n in 0..=9 {
        for mask in 0..1_usize << n {
            let t: Vec<u8> = (0..n).map(|i| u8::from(mask & (1 << i) != 0)).collect();
            for m in 0..=6 {
                for bits in 0..1_usize << m {
                    let p: Vec<u8> = (0..m).map(|i| u8::from(bits & (1 << i) != 0)).collect();
                    check(&t, &p);
                }
            }
        }
    }
}
#[test]
fn seeded_larger_and_periodic_inputs() {
    let mut seed = 39_u64;
    for case in 0..2000 {
        let n = case % 513;
        let m = case % 67;
        let mut bytes = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed.to_le_bytes()[0]
        };
        let mut t: Vec<u8> = (0..n)
            .map(|_| bytes() % if case % 2 == 0 { 4 } else { 255 })
            .collect();
        let p: Vec<u8> = (0..m)
            .map(|_| bytes() % if case % 2 == 0 { 4 } else { 255 })
            .collect();
        if case % 3 == 0 && m <= n {
            t[n - m..].copy_from_slice(&p);
        }
        check(&t, &p);
    }
    for m in [4, 31, 32, 63, 64, 127] {
        let t = vec![b'a'; 4096];
        let mut p = vec![b'a'; m];
        p[0] = b'b';
        check(&t, &p);
        let h = Finder::new(&p, Candidate::Horspool).diagnose(&t).1;
        assert_eq!(h.comparisons, (t.len() - m + 1) * m);
        let a = Finder::new(&p, Candidate::Adaptive).diagnose(&t).1;
        assert_eq!(a.fallbacks, 1);
        assert!(a.comparisons <= 4 * t.len() + 2 * m);
        p[0] = b'a';
        p[m - 1] = b'b';
        check(&t, &p);
        let periodic: Vec<u8> = (0..4096)
            .map(|i| if i % 2 == 0 { b'a' } else { b'b' })
            .collect();
        for offset in 0..m {
            check(&periodic[offset..], &periodic[..m]);
        }
    }
}
