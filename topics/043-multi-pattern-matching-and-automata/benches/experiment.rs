//! Default Cargo benchmark smoke; the fixed process schedule is in measurements/run.py.
//!
//! Cargo passes `--bench` to `harness = false` targets, so this entry point
//! ignores command-line arguments.
use std::hint::black_box;
use topic043::{Dense, Hit, Sparse, direct};

fn main() {
    let p: Vec<Vec<u8>> = ["he", "she", "hers", "his", "he"]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
    let want = vec![
        Hit {
            id: 0,
            start: 2,
            end: 4,
        },
        Hit {
            id: 1,
            start: 1,
            end: 4,
        },
        Hit {
            id: 2,
            start: 2,
            end: 6,
        },
        Hit {
            id: 4,
            start: 2,
            end: 4,
        },
    ];
    let text = black_box(b"ushers");
    let sparse = Sparse::new(black_box(&p)).unwrap();
    let dense = Dense::new(black_box(&p)).unwrap();
    for mode in 0..3 {
        let mut got = Vec::new();
        match mode {
            0 => direct(&p, text, |h| got.push(h)).unwrap(),
            1 => sparse.scan(text, |h| got.push(h)),
            _ => dense.scan(text, |h| got.push(h)),
        }
        got.sort_unstable();
        assert_eq!(got, want);
        black_box(got);
    }
}
