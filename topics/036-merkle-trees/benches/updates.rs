//! Fresh-process benchmark entry point. See BENCHMARK.md for timing boundaries.
use std::{hint::black_box, time::Instant};
use topic036_merkle_trees::{Method, Tree, sha256, verify};
fn main() {
    // Cargo appends `--bench`; the remaining positional arguments keep their indices.
    let args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    if args.get(1).is_some_and(|s| s == "hash") {
        let n: usize = args[2].parse().unwrap();
        let input: Vec<u8> = (0..n).map(|i| u8::try_from(i % 251).unwrap()).collect();
        for b in sha256(&input) {
            print!("{b:02x}");
        }
        println!();
        return;
    }
    let n = args.get(1).map_or(4, |s| s.parse().unwrap());
    let k = args.get(2).map_or(1, |s| s.parse().unwrap());
    let bytes = args.get(3).map_or(1, |s| s.parse().unwrap());
    let pattern = args.get(4).map_or("cluster", String::as_str);
    let reps: usize = args.get(5).map_or(2, |s| s.parse().unwrap());
    let mode = args.get(6).map_or("update", String::as_str);
    let method = match args.get(7).map_or("batch", String::as_str) {
        "full" => Method::Full,
        "eager" => Method::Eager,
        "batch" => Method::Batch,
        _ => panic!("method"),
    };
    let initial: Vec<Vec<u8>> = (0..n)
        .map(|i| {
            (0..bytes)
                .map(|j| u8::try_from((i * 17 + j * 13) % 251).unwrap())
                .collect()
        })
        .collect();
    let batches: Vec<Vec<(usize, Vec<u8>)>> = (0..reps)
        .map(|r| {
            (0..k)
                .map(|q| {
                    let i = match pattern {
                        "scatter" => (q * 4051) % n,
                        "duplicate" => n / 2,
                        _ => q % n,
                    };
                    let mut v = initial[i].clone();
                    if pattern != "noop" {
                        // A delta in 1..=250 changes a byte held modulo 251.
                        v[0] = u8::try_from((usize::from(v[0]) + (r + q) % 250 + 1) % 251).unwrap();
                    }
                    (i, v)
                })
                .collect()
        })
        .collect();
    let start = Instant::now();
    let tree = Tree::new(initial.clone()).unwrap();
    let build_ns = start.elapsed().as_nanos();
    let mut elapsed = 0u128;
    let mut calls = 0usize;
    let mut final_root = tree.root();
    // Lifecycle mode drops the initial tree before the loop so each timed
    // batch owns the only tree in memory.
    let mut tree = (mode != "lifecycle").then_some(tree);
    for batch in &batches {
        if let Some(tree) = tree.as_mut() {
            let start = Instant::now();
            calls += tree.update(black_box(batch), method).unwrap();
            final_root = black_box(tree.root());
            elapsed += start.elapsed().as_nanos();
        } else {
            let start = Instant::now();
            let mut fresh = Tree::new(initial.clone()).unwrap();
            calls += fresh.update(black_box(batch), method).unwrap();
            final_root = black_box(fresh.root());
            drop(fresh);
            elapsed += start.elapsed().as_nanos();
        }
    }
    let mut expected = initial;
    for batch in if mode == "lifecycle" {
        &batches[reps - 1..]
    } else {
        &batches[..]
    } {
        for (i, v) in batch {
            expected[*i].clone_from(v);
        }
    }
    let check = Tree::new(expected.clone()).unwrap();
    assert_eq!(final_root, check.root());
    assert!(verify(
        &expected[n / 2],
        n / 2,
        n,
        &check.proof(n / 2).unwrap(),
        final_root
    ));
    black_box(tree);
    print!("{elapsed},{reps},{calls},{build_ns},");
    for b in final_root {
        print!("{b:02x}");
    }
    println!();
}
