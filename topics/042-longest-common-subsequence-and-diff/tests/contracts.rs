//! Exhaustive oracle agreement and semantic negative controls.
use topic_042_lcs_diff::{Candidate, Edit, diff, oracle, valid, workload};
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
    assert!(!valid(b"A", b"B", &[Edit::Keep(b'A')], 0));
    assert!(!valid(b"A", b"B", &[Edit::Keep(b'B')], 0));
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
#[test]
fn counter_contracts_and_workloads() {
    for shape in ["identical", "sparse", "unrelated", "repeat", "unequal"] {
        let (a, b) = workload(shape, 64);
        let (m, n) = (a.len(), b.len());
        let cost = oracle(&a, &b);
        for candidate in ALL {
            let (script, stats) = diff(&a, &b, candidate);
            assert!(valid(&a, &b, &script, cost), "{candidate:?} {shape}");
            match candidate {
                Candidate::Matrix => {
                    assert_eq!(stats.score_slots, (m + 1) * (n + 1));
                    assert_eq!(stats.score_allocations, 1);
                    assert!(
                        stats.comparisons > m * n && stats.comparisons <= m * n + m + n,
                        "{shape} {}",
                        stats.comparisons
                    );
                }
                Candidate::Hirschberg => assert!(stats.score_slots <= 2 * (m.min(n) + 1)),
                Candidate::Myers => {
                    assert_eq!(stats.score_allocations, cost + 1);
                    assert_eq!(stats.score_slots, (cost + 1) * (cost + 2) / 2);
                }
            }
        }
    }
}
#[test]
#[should_panic(expected = "unknown shape")]
fn workload_rejects_unknown_shape() {
    let _ = workload("sprase", 8);
}
