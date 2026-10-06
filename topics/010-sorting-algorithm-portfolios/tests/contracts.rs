//! Independent contract and invariant checks.
use sorting_lab::{METHODS, Record, agrees, is_stable, sort, workload};
fn check(a: &[Record]) {
    for method in METHODS {
        let mut b = a.to_vec();
        match sort(&mut b, method) {
            Ok(()) => assert!(agrees(a, &b, is_stable(method)), "{method} {a:?}"),
            Err(i) => {
                assert_eq!(method, "groups");
                assert!(a[i - 1].key >> 32 > a[i].key >> 32);
                assert_eq!(a, b);
            }
        }
    }
}
#[test]
fn exhaustive() {
    let mut total = 0;
    for n in 0..=8 {
        for code in 0..3_usize.pow(n) {
            let mut c = code;
            let a: Vec<_> = (0..n)
                .map(|i| {
                    let key = (c % 3) as u64;
                    c /= 3;
                    Record { key, id: i }
                })
                .collect();
            check(&a);
            total += 1;
        }
    }
    assert_eq!(total, 9841);
}
#[test]
fn shapes_and_cutoffs() {
    for n in [0, 1, 2, 15, 16, 17, 31, 32, 33, 255, 256, 257, 4096] {
        for shape in [
            "random",
            "equal",
            "duplicates",
            "sorted",
            "reverse",
            "organ",
            "runs",
            "nearly",
            "groups",
            "skew",
        ] {
            for seed in [1, 17, 4096] {
                check(&workload(n, shape, seed));
            }
        }
    }
}
#[test]
fn strict_reverse_stability() {
    check(&[
        Record { key: 3, id: 0 },
        Record { key: 2, id: 1 },
        Record { key: 2, id: 2 },
        Record { key: 1, id: 3 },
    ]);
}
#[test]
fn extrema_and_duplicate_payload() {
    check(&[
        Record {
            key: u64::MAX,
            id: 7,
        },
        Record { key: 0, id: 7 },
        Record {
            key: 1 << 32,
            id: 0,
        },
        Record { key: 0, id: 7 },
        Record {
            key: u64::MAX,
            id: 2,
        },
    ]);
}
#[test]
fn prefix_violation_atomic() {
    let a = [
        Record {
            key: 2 << 32,
            id: 0,
        },
        Record {
            key: 1 << 32,
            id: 1,
        },
    ];
    let mut b = a;
    assert_eq!(sort(&mut b, "groups"), Err(1));
    assert_eq!(a, b);
}
#[test]
fn shuffled_ids_use_arrival_order() {
    check(&[
        Record { key: 2, id: 99 },
        Record { key: 2, id: 0 },
        Record { key: 1, id: 4 },
        Record { key: 2, id: 1 },
    ]);
}
#[test]
fn arbitrary_segments_cannot_concatenate() {
    let input = [
        Record { key: 9, id: 0 },
        Record { key: 1, id: 1 },
        Record { key: 8, id: 2 },
        Record { key: 2, id: 3 },
    ];
    let mut wrong = input;
    wrong[..2].sort_by_key(|r| r.key);
    wrong[2..].sort_by_key(|r| r.key);
    assert!(!agrees(&input, &wrong, true));
    check(&input);
}
