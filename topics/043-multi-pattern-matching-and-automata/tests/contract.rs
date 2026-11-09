//! Public API regression for complete match records.
use topic043::{Dense, EmptyPattern, Hit, Sparse, direct};

fn dictionary() -> Vec<Vec<u8>> {
    ["he", "she", "hers", "his", "he"]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect()
}

fn assert_increasing_end(hits: &[Hit]) {
    for pair in hits.windows(2) {
        assert!(pair[0].end <= pair[1].end, "emission order {hits:?}");
    }
}

#[test]
fn exact_suffix_and_duplicate_records() {
    let p = dictionary();
    let ac = Sparse::new(&p).unwrap();
    let mut c = ac.cursor();
    let mut got = Vec::new();
    c.feed(b"ush", |h| got.push(h));
    c.feed(b"ers", |h| got.push(h));
    assert_increasing_end(&got);
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

#[test]
fn every_candidate_emits_in_increasing_end_order() {
    let p: Vec<Vec<u8>> = [&b"a"[..], b"aa", b"aaa", b"aa"]
        .iter()
        .map(|s| s.to_vec())
        .collect();
    let text = b"aaaaa";
    let sparse = Sparse::new(&p).unwrap();
    let dense = Dense::new(&p).unwrap();
    let mut got = Vec::new();
    direct(&p, text, |h| got.push(h)).unwrap();
    assert_increasing_end(&got);
    assert_eq!(got.len(), 5 + 4 + 3 + 4);
    got.clear();
    sparse.scan(text, |h| got.push(h));
    assert_increasing_end(&got);
    assert_eq!(got.len(), 16);
    got.clear();
    dense.scan(text, |h| got.push(h));
    assert_increasing_end(&got);
    assert_eq!(got.len(), 16);
}

#[test]
fn rejects_the_first_of_several_empty_patterns() {
    let p = vec![b"a".to_vec(), Vec::new(), Vec::new()];
    assert_eq!(Sparse::new(&p).err(), Some(EmptyPattern(1)));
    assert_eq!(Dense::new(&p).err(), Some(EmptyPattern(1)));
    let mut count = 0;
    assert_eq!(direct(&p, b"abc", |_| count += 1), Err(EmptyPattern(1)));
    assert_eq!(count, 0);
}

#[test]
fn running_example_state_count_and_table_payload() {
    let p = dictionary();
    let sparse = Sparse::new(&p).unwrap();
    let dense = Dense::new(&p).unwrap();
    assert_eq!(sparse.states(), 10);
    assert_eq!(dense.table_bytes(), 10 * 256 * std::mem::size_of::<usize>());
}
