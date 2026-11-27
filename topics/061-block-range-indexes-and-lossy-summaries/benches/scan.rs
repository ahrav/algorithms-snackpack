//! Measure exact resident query batches and track build cost separately.

use block_range_indexes_and_lossy_summaries::{Index, Layout, full_scan};
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

const LARGE: usize = 1 << 18;

fn workload(name: &str) -> (Vec<i32>, usize, [(i32, i32); 8], usize) {
    let (size, width, repeats) = match name {
        "tiny" => (64, 16, 4_096),
        "ordered-wide-block" => (LARGE, 2_048, 16),
        _ => (LARGE, 256, 16),
    };
    let rows = (0..i32::try_from(size).unwrap())
        .map(|i| match name {
            "shuffled" => {
                i.wrapping_mul(65_537).wrapping_add(12_345) & (i32::try_from(size).unwrap() - 1)
            }
            "bimodal-gap" | "bimodal-hit" => (i & 63) + if i & 64 == 0 { 0 } else { 4096 },
            _ => i,
        })
        .collect();
    let queries = std::array::from_fn(|i| {
        let i = i32::try_from(i).unwrap();
        match name {
            "tiny" => (i * 5, i * 5 + 2),
            "bimodal-gap" => (1000 + i * 37, 1012 + i * 37),
            "bimodal-hit" => (i * 3, i * 3 + 1),
            "wide" => (i * 2048, 230_000 + i * 1024),
            _ => (10_123 + i * 25_037, 10_135 + i * 25_037),
        }
    });
    (rows, width, queries, repeats)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        return;
    }
    let (rows, width, queries, repeats) = workload(&args[2]);
    let width = NonZeroUsize::new(width).unwrap();
    let start = Instant::now();
    let index = match args[1].as_str() {
        "full" => None,
        "minmax" => Some(Index::build(&rows, width, Layout::MinMax)),
        "two" => Some(Index::build(&rows, width, Layout::TwoRanges)),
        _ => panic!("unknown candidate"),
    };
    let build_ns = start.elapsed().as_nanos();
    let scan = |low, high| match &index {
        None => full_scan(black_box(&rows), low, high),
        Some(index) => black_box(index).scan(low, high),
    };
    let exact = queries.map(|(low, high)| scan(low, high).matches);
    let start = Instant::now();
    let mut counts = [0; 4];
    for _ in 0..repeats {
        for &(low, high) in black_box(&queries) {
            let result = black_box(scan(low, high));
            counts[0] += result.matches;
            counts[1] += result.summaries;
            counts[2] += result.candidate_blocks;
            counts[3] += result.checked_rows;
        }
    }
    let query_ns = start.elapsed().as_nanos();
    println!("{}", exact.map(|n| n.to_string()).join(","));
    let bytes = index.as_ref().map_or(0, Index::summary_bytes);
    println!(
        "{build_ns},{query_ns},{repeats},{bytes},{},{},{},{}",
        counts[0], counts[1], counts[2], counts[3]
    );
}
