//! One fresh process per candidate, workload, and independent run.
#![allow(clippy::cast_precision_loss)]
use lower_bound_portfolio::{CANDIDATES, oracle};
use std::{hint::black_box, time::Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        println!("usage: search CANDIDATE N SHAPE warm|first REPEATS");
        return;
    }
    let n: usize = args[2].parse().expect("n");
    let repeats: usize = args[5].parse().expect("repeats");
    let f = CANDIDATES
        .iter()
        .find(|(name, _)| *name == args[1])
        .expect("candidate")
        .1;
    let setup_start = Instant::now();
    let a: Vec<u64> = (0..n)
        .map(|i| {
            let i = u64::try_from(i).expect("bounded workload");
            if args[3] == "duplicates" {
                i / 32
            } else {
                i * 2
            }
        })
        .collect();
    let mut seed = 0xb817_934a_135d_ac71u64;
    let max = a.last().copied().unwrap_or(0);
    let queries: Vec<u64> = (0..128u64)
        .map(|i| {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            match args[3].as_str() {
                "low" => seed % 16,
                "repeat" => max / 2,
                "monotone" => i * max / 127,
                "endpoints" => {
                    if i % 2 == 0 {
                        0
                    } else {
                        u64::MAX
                    }
                }
                "uniform" | "duplicates" => seed % (max + 2),
                _ => panic!("shape"),
            }
        })
        .collect();
    let setup_ns = setup_start.elapsed().as_nanos();
    // No input reads through an oracle before a first-pass timing.
    if args[4] == "warm" {
        for &key in &queries {
            assert_eq!(f(&a, key), oracle(&a, key));
        }
        for _ in 0..4 {
            for &key in &queries {
                black_box(f(black_box(&a), black_box(key)));
            }
        }
    } else {
        assert_eq!(args[4], "first");
        assert_eq!(repeats, 1);
    }
    let start = Instant::now();
    let mut sum = 0usize;
    for _ in 0..repeats {
        for &key in &queries {
            sum = sum.wrapping_add(black_box(f(black_box(&a), black_box(key))));
        }
    }
    let elapsed = start.elapsed().as_nanos();
    for &key in &queries {
        assert_eq!(f(&a, key), oracle(&a, key));
    }
    let teardown_start = Instant::now();
    drop(a);
    drop(queries);
    let teardown_ns = teardown_start.elapsed().as_nanos();
    let calls = repeats * 128;
    println!(
        "{{\"candidate\":\"{}\",\"n\":{},\"shape\":\"{}\",\"mode\":\"{}\",\"calls\":{},\"elapsed_ns\":{},\"ns_per_query\":{},\"setup_ns\":{},\"teardown_ns\":{},\"checksum\":{}}}",
        args[1],
        n,
        args[3],
        args[4],
        calls,
        elapsed,
        elapsed as f64 / calls as f64,
        setup_ns,
        teardown_ns,
        sum
    );
}
