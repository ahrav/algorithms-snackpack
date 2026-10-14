//! Controlled complete resident count-join timing.
use hash_joins::{chunk, hash, merge, partition};
use std::{hint::black_box, time::Instant};
fn data(case: &str) -> (Vec<u64>, Vec<u64>, usize, usize) {
    let (n, m, keys, budget, repetitions): (usize, usize, usize, usize, usize) = match case {
        "tiny" => (8, 12, 17, 1024, 4000),
        "uniform" | "disjoint" | "sorted" => (4096, 8192, 16384, 1024, 20),
        "duplicates" => (4096, 8192, 16, 1024, 20),
        "heavy" => (4096, 8192, 1, 1024, 20),
        "large" => (65536, 131_072, 262_144, 1024, 2),
        "small_build" => (32, 131_072, 262_144, 1024, 4),
        _ => panic!("unknown case"),
    };
    let mut a: Vec<_> = (0..n).map(|i| ((i * 7919) % keys) as u64).collect();
    let mut b: Vec<_> = (0..m).map(|i| ((i * 3571) % keys) as u64).collect();
    if case == "disjoint" {
        for k in &mut b {
            *k += keys as u64;
        }
    }
    if case == "sorted" {
        a.sort_unstable();
        b.sort_unstable();
    }
    (a, b, budget, repetitions)
}
// Independent frequency oracle using an ordered map, outside the timer.
fn oracle(a: &[u64], b: &[u64]) -> u128 {
    let mut frequencies = std::collections::BTreeMap::<u64, u128>::new();
    for key in a {
        *frequencies.entry(*key).or_default() += 1;
    }
    b.iter()
        .map(|key| *frequencies.get(key).unwrap_or(&0))
        .sum()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        println!("Run measurements/runner.py for the declared campaign.");
        return;
    }
    let (a, b, budget, repetitions) = data(&args[2]);
    let run = || match args[1].as_str() {
        "hash" => hash(black_box(&a), black_box(&b)),
        "merge" => merge(black_box(&a), black_box(&b)),
        "partition" => partition(black_box(&a), black_box(&b)),
        "chunk" => chunk(black_box(&a), black_box(&b), budget),
        _ => panic!("unknown candidate"),
    };
    let expected = oracle(&a, &b);
    for _ in 0..2 {
        assert_eq!(black_box(run()), expected);
    }
    let start = Instant::now();
    for _ in 0..repetitions {
        assert_eq!(black_box(run()), expected);
    }
    let elapsed = start.elapsed().as_nanos();
    println!(
        "OK ns={elapsed} operations={repetitions} pairs={expected} build={} probe={} budget={budget}",
        a.len(),
        b.len()
    );
}
