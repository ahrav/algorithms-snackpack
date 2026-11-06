#![allow(missing_docs)]
use std::{hint::black_box, time::Instant};
use topic_041_edit::{Trie, scan};

fn matrix(a: &[u8], b: &[u8]) -> usize {
    let mut table = vec![vec![0; b.len() + 1]; a.len() + 1];
    for (i, row) in table.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in table[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            table[i][j] = (table[i - 1][j] + 1)
                .min(table[i][j - 1] + 1)
                .min(table[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]));
        }
    }
    table[a.len()][b.len()]
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let method = args.get(1).map_or("row", String::as_str);
    let n = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(32);
    let m = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(8);
    let shape = args.get(4).map_or("random", String::as_str);
    let k = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(1);
    let repeats = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut seed = 0x1234_5678_u64;
    let query: Vec<u8> = (0..m)
        .map(|j| b'a' + u8::try_from(j % 16).unwrap_or(0))
        .collect();
    let terms: Vec<Vec<u8>> = (0..n)
        .map(|i| {
            let mut word: Vec<_> = (0..m)
                .map(|j| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    if shape == "dense" || (shape == "prefix" && j < m * 3 / 4) {
                        query[j]
                    } else {
                        b'a' + u8::try_from(seed % 16).unwrap_or(0)
                    }
                })
                .collect();
            if shape == "dense" && m > 0 && i % 2 == 0 {
                word[i % m] = b'z';
            }
            if i == 0 {
                word.clone_from(&query);
            }
            word
        })
        .collect();
    let wanted: Vec<_> = terms
        .iter()
        .enumerate()
        .filter_map(|(i, t)| (matrix(&query, t) <= k).then_some(i))
        .collect();
    let start = Instant::now();
    let trie = if method == "trie" {
        Some(Trie::new(&terms))
    } else {
        None
    };
    let build_ns = start.elapsed().as_nanos();
    let search = || {
        if let Some(t) = &trie {
            t.search(black_box(&query), k).0
        } else {
            scan(black_box(&terms), black_box(&query), k, method)
        }
    };
    assert_eq!(search(), wanted);
    let start = Instant::now();
    let mut checksum = 0;
    for _ in 0..repeats {
        let ids = black_box(search());
        checksum += ids.len();
        black_box(&ids);
    }
    let elapsed_ns = start.elapsed().as_nanos();
    assert_eq!(checksum, wanted.len() * repeats);
    let (nodes, edges) = trie
        .as_ref()
        .map_or((0, 0), |t| (t.node_count(), t.search(&query, k).1));
    println!(
        "{{\"method\":\"{method}\",\"n\":{n},\"m\":{m},\"shape\":\"{shape}\",\"k\":{k},\"repeats\":{repeats},\"elapsed_ns\":{elapsed_ns},\"build_ns\":{build_ns},\"hits\":{},\"nodes\":{nodes},\"edges\":{edges},\"oracle\":true}}",
        wanted.len()
    );
}
