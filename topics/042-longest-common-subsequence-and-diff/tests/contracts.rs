//! Exhaustive oracle agreement and semantic negative controls.
use topic_042_lcs_diff::{Candidate, Edit, diff, oracle, valid};
const ALL: [Candidate; 3] = [Candidate::Matrix, Candidate::Hirschberg, Candidate::Myers];
#[test]
fn exhaustive_binary_pairs() {
    let mut words = Vec::new();
    for n in 0..=5 {
        for bits in 0..(1 << n) {
            words.push(
                (0..n)
                    .map(|i| u8::from(bits & (1 << i) != 0))
                    .collect::<Vec<_>>(),
            );
        }
    }
    assert_eq!(words.len(), 63);
    for a in &words {
        for b in &words {
            let cost = oracle(a, b);
            for candidate in ALL {
                let (script, _) = diff(a, b, candidate);
                assert!(
                    valid(a, b, &script, cost),
                    "{candidate:?} {a:?} {b:?} {script:?}"
                );
            }
        }
    }
}
#[test]
fn semantic_controls_and_boundaries() {
    assert_eq!(oracle(b"ABCABBA", b"CBABAC"), 5);
    assert_eq!(oracle(b"A", b"B"), 2);
    for (a, b) in [
        (b"".as_slice(), b"abc".as_slice()),
        (b"abc", b""),
        (b"aaaa", b"aaaa"),
        (b"\0\xff", b"\xff\0"),
        (b"ABCABBA", b"CBABAC"),
    ] {
        for c in ALL {
            let (s, _) = diff(a, b, c);
            assert!(valid(a, b, &s, oracle(a, b)));
        }
    }
    assert!(!valid(b"A", b"B", &[Edit::Keep(b'A')], 2));
    assert!(!valid(
        b"A",
        b"B",
        &[Edit::Delete(b'A'), Edit::Insert(b'B')],
        1
    ));
    assert!(!valid(
        b"A",
        b"A",
        &[Edit::Delete(b'A'), Edit::Insert(b'A')],
        0
    ));
}
