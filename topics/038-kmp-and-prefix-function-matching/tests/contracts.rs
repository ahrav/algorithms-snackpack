//! Differential checks for complete offsets, borders, overlaps and streams.
use topic038_kmp_and_prefix_function_matching::{
    Dfa, Kmp, direct, direct_comparisons, oracle, prefix_function,
};

fn check(text: &[u8], pattern: &[u8]) {
    let want = oracle(text, pattern).unwrap();
    let k = Kmp::new(pattern).unwrap();
    assert_eq!(direct(text, pattern).unwrap(), want);
    assert_eq!(k.find_all(text), want);
    assert_eq!(Dfa::new(pattern).unwrap().find_all(text), want);
    for split in 0..=text.len() {
        let mut s = k.stream();
        let mut got = s.push(&text[..split]).unwrap();
        got.extend(s.push(b"").unwrap());
        got.extend(s.push(&text[split..]).unwrap());
        assert_eq!(got, want);
    }
    let mut s = k.stream();
    let got: Vec<_> = text.chunks(1).flat_map(|c| s.push(c).unwrap()).collect();
    assert_eq!(got, want);
}

#[test]
fn exhaustive_binary_and_borders() {
    for n in 0..=8 {
        for bits in 0..1usize << n {
            let t: Vec<u8> = (0..n).map(|i| u8::from(bits & (1 << i) != 0)).collect();
            for m in 1..=4 {
                for pb in 0..1usize << m {
                    let p: Vec<u8> = (0..m).map(|i| u8::from(pb & (1 << i) != 0)).collect();
                    check(&t, &p);
                    let brute: Vec<_> = (0..m)
                        .map(|i| (0..=i).rev().find(|&j| p[..j] == p[i + 1 - j..=i]).unwrap())
                        .collect();
                    assert_eq!(prefix_function(&p), brute);
                }
            }
        }
    }
}

#[test]
fn named_contracts() {
    assert!(oracle(b"", b"").is_none());
    assert!(direct(b"abc", b"").is_none());
    assert!(Kmp::new(b"").is_none());
    assert!(Dfa::new(b"").is_none());
    assert_eq!(prefix_function(b"ababaca"), [0, 0, 1, 2, 3, 0, 1]);
    // Index 5 falls back through borders 3 and 1 before matching at 0.
    assert_eq!(prefix_function(b"ababaa"), [0, 0, 1, 2, 3, 1]);
    assert_eq!(direct_comparisons(b"abababacababaca", b"ababaca"), Some(30));
    assert_eq!(
        Kmp::new(b"ababaca")
            .unwrap()
            .comparisons(b"abababacababaca"),
        30
    );
    for (t, p) in [
        (b"aaaaa".as_slice(), b"aaa".as_slice()),
        (b"abababacababaca", b"ababaca"),
        (b"abc", b"abc"),
        (b"", b"x"),
        (b"x", b"long"),
        (b"xabc", b"abc"),
    ] {
        check(t, p);
    }
    let all: Vec<u8> = (0..=255).collect();
    check(&all, &[254, 255]);
    check(&all, &[0]);
    let k = Kmp::new(b"aaa").unwrap();
    assert_eq!(k.find_all(b"aaaaa"), [0, 1, 2]);
}
