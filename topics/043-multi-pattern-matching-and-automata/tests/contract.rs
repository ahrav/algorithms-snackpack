//! Public API regression for complete match records.
use topic043::{Hit, Sparse};
#[test]
fn exact_suffix_and_duplicate_records() {
    let p: Vec<Vec<u8>> = ["he", "she", "hers", "his", "he"]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
    let ac = Sparse::new(&p).unwrap();
    let mut c = ac.cursor();
    let mut got = Vec::new();
    c.feed(b"ush", |h| got.push(h));
    c.feed(b"ers", |h| got.push(h));
    got.sort_unstable();
    assert_eq!(
        got,
        vec![
            Hit {
                id: 0,
                start: 2,
                end: 4
            },
            Hit {
                id: 1,
                start: 1,
                end: 4
            },
            Hit {
                id: 2,
                start: 2,
                end: 6
            },
            Hit {
                id: 4,
                start: 2,
                end: 4
            }
        ]
    );
}
