//! Frozen whole-operation snapshot and lifecycle benchmark.
#![allow(dead_code)]
#[path = "../src/lib.rs"]
mod lesson;
use lesson::{Index, Kind, Pair, oracle};
use std::{hint::black_box, time::Instant};
fn mix(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}
fn fold(pairs: &[Pair]) -> u64 {
    pairs
        .iter()
        .fold(0_u64, |acc, &(k, r)| acc.wrapping_add(mix(k) ^ r))
}
fn main() {
    // `cargo bench` passes `--bench` to this executable.
    let args: Vec<_> = std::env::args().filter(|a| a != "--bench").collect();
    if args.len() != 3 {
        return;
    }
    let kind = match args[1].as_str() {
        "flat" => Kind::Flat,
        "tree" => Kind::Tree,
        "posting" => Kind::Posting,
        "packed8" => Kind::Packed8,
        "packed32" => Kind::Packed32,
        _ => panic!("kind"),
    };
    let cell = args[2].as_str();
    let (count, distinct, width, repeats, build) = match cell {
        "tiny" => (32, 8, 1, 128, false),
        "distinct" => (16384, 16384, 1, 8, false),
        "duplicates" => (32768, 128, 1, 8, false),
        "hot" => (32768, 1, 1, 1, false),
        "range" => (32768, 128, 8, 2, false),
        "broad" => (16384, 16384, 4096, 1, false),
        "build_distinct" => (16384, 16384, 1, 1, true),
        "build_duplicates" => (32768, 128, 1, 1, true),
        "small_lifecycle" => (32, 8, 1, 32, true),
        _ => panic!("cell"),
    };
    let mut input: Vec<Pair> = (0..count).map(|n| ((n % distinct) * 2, n)).collect();
    // Frozen Fisher-Yates permutation. Same workload on every host/candidate.
    for i in (1..input.len()).rev() {
        let j = usize::try_from(mix(u64::try_from(i).unwrap()) % u64::try_from(i + 1).unwrap())
            .unwrap();
        input.swap(i, j);
    }
    let queries: Vec<_> = (0..64_u64)
        .map(|q| {
            let low = if cell == "hot" {
                0
            } else {
                mix(q + 23019) % (distinct * 2 + 2)
            };
            (low, low + width)
        })
        .collect();
    let index = Index::build(kind, &input);
    assert!(index.valid());
    let mut expected = 0_u64;
    for &(low, high) in &queries {
        let exact = oracle(&input, low, high);
        let actual = index.range(low, high);
        assert_eq!(actual, exact);
        expected = expected.wrapping_add(fold(&exact));
    }
    let bytes = index.logical_bytes();
    let index = if build { None } else { Some(index) };
    let execute = || {
        let mut checksum = 0_u64;
        for _ in 0..repeats {
            if build {
                let temp = Index::build(kind, black_box(&input));
                for &(low, high) in &queries {
                    checksum = checksum.wrapping_add(fold(&black_box(
                        temp.range(black_box(low), black_box(high)),
                    )));
                }
                drop(black_box(temp));
            } else {
                for &(low, high) in &queries {
                    checksum = checksum.wrapping_add(fold(&black_box(
                        index
                            .as_ref()
                            .expect("steady index")
                            .range(black_box(low), black_box(high)),
                    )));
                }
            }
        }
        black_box(checksum)
    };
    // One whole-operation warmup, then three subsamples in the same process.
    let warm = execute();
    assert_eq!(warm, expected.wrapping_mul(repeats));
    let mut samples = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let checksum = execute();
        let ns = start.elapsed().as_nanos();
        assert_eq!(checksum, warm);
        samples.push(ns);
    }
    samples.sort_unstable();
    println!(
        "{{\"ns\":{},\"checksum\":{},\"logical_bytes\":{},\"pairs\":{},\"distinct\":{},\"queries\":64,\"repeats\":{},\"build\":{}}}",
        samples[1], warm, bytes, count, distinct, repeats, build
    );
}
