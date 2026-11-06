//! Independent edit-graph and representation-boundary contracts.
use topic_041_edit::{Myers, Trie, bounded, distance, scan};

// Shortest paths in the edit graph form an independent oracle. Each edge
// consumes query bytes, term bytes, or both; edge costs encode the contract.
fn oracle(a: &[u8], b: &[u8]) -> usize {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let mut heap = BinaryHeap::from([Reverse((0, 0, 0))]);
    let mut seen = vec![vec![false; b.len() + 1]; a.len() + 1];
    while let Some(Reverse((cost, i, j))) = heap.pop() {
        if seen[i][j] {
            continue;
        }
        seen[i][j] = true;
        if i == a.len() && j == b.len() {
            return cost;
        }
        if i < a.len() {
            heap.push(Reverse((cost + 1, i + 1, j)));
        }
        if j < b.len() {
            heap.push(Reverse((cost + 1, i, j + 1)));
        }
        if i < a.len() && j < b.len() {
            heap.push(Reverse((cost + usize::from(a[i] != b[j]), i + 1, j + 1)));
        }
    }
    unreachable!()
}

#[test]
fn exhaustive_edit_graph_and_dictionary_agreement() {
    let mut words = vec![vec![]];
    for n in 1..=5 {
        for bits in 0..(1 << n) {
            words.push(
                (0..n)
                    .map(|j| if bits & (1 << j) == 0 { b'a' } else { b'b' })
                    .collect(),
            );
        }
    }
    let trie = Trie::new(&words);
    for a in &words {
        let myers = Myers::new(a);
        for b in &words {
            let want = oracle(a, b);
            assert_eq!(distance(a, b), want);
            assert_eq!(myers.distance(b), want);
            for k in 0..=6 {
                assert_eq!(bounded(a, b, k), (want <= k).then_some(want));
            }
        }
        for k in 0..=3 {
            let want: Vec<_> = words
                .iter()
                .enumerate()
                .filter_map(|(i, b)| (oracle(a, b) <= k).then_some(i))
                .collect();
            assert_eq!(trie.search(a, k).0, want);
            for method in ["row", "band", "myers"] {
                assert_eq!(scan(&words, a, k, method), want);
            }
        }
    }
}

#[test]
fn width_global_boundary_and_byte_contract() {
    for n in [0, 1, 63, 64, 65, 128] {
        let a = vec![255; n];
        for b in [
            vec![],
            a.clone(),
            [vec![0], a.clone()].concat(),
            vec![254; n],
        ] {
            assert_eq!(Myers::new(&a).distance(&b), distance(&a, &b));
            for k in [0, 1, 2, usize::MAX] {
                let d = distance(&a, &b);
                assert_eq!(bounded(&a, &b, k), (d <= k).then_some(d));
            }
        }
    }
    assert_eq!(distance(b"cat", b"xxcat"), 2);
    assert_eq!(Myers::new(b"cat").distance(b"xxcat"), 2);
    assert_eq!(distance(b"ab", b"ba"), 2);
    assert_eq!(distance("é".as_bytes(), b"e"), 2);
    let terms = vec![
        b"cat".to_vec(),
        b"cat".to_vec(),
        vec![],
        b"cart".to_vec(),
        b"dog".to_vec(),
    ];
    assert_eq!(Trie::new(&terms).search(b"cat", 1).0, vec![0, 1, 3]);
    assert_eq!(Trie::default().search(b"", 0).0, Vec::<usize>::new());
}
