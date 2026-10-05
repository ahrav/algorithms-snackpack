//! Frozen whole-call sampler timing harness.
#![allow(clippy::cast_precision_loss)]
use sampling_lab::{Uniform, uniform, weighted_heap, weighted_sort};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 1 {
        assert_eq!(
            uniform(8, 3, 42, Uniform::PrioritySort),
            uniform(8, 3, 42, Uniform::PriorityHeap)
        );
        println!("benchmark smoke passed");
        return;
    }
    if a.len() != 7 {
        eprintln!("sampling METHOD N K SHAPE REPS SEED");
        std::process::exit(2);
    }
    let method = &a[1];
    let n: usize = a[2].parse().unwrap();
    let k: usize = a[3].parse().unwrap();
    let reps: usize = a[5].parse().unwrap();
    let seed: u64 = a[6].parse().unwrap();
    let weights: Vec<f64> = (0..n)
        .map(|i| match a[4].as_str() {
            "skew" => {
                if i % 64 == 0 {
                    1e6
                } else {
                    1.0
                }
            }
            "ascending" => (i + 1) as f64,
            _ => 1.0,
        })
        .collect();
    let run = |s| match method.as_str() {
        "r" => uniform(n, k, s, Uniform::R),
        "x" => uniform(n, k, s, Uniform::X),
        "sort" => uniform(n, k, s, Uniform::PrioritySort),
        "heap" => uniform(n, k, s, Uniform::PriorityHeap),
        "wsort" => weighted_sort(&weights, k, s)
            .unwrap()
            .iter()
            .map(|x| x.id)
            .collect(),
        "wheap" => weighted_heap(&weights, k, s)
            .unwrap()
            .iter()
            .map(|x| x.id)
            .collect(),
        _ => panic!("method"),
    };
    if a[4] != "cold" {
        for i in 0..3 {
            black_box(run(seed + i));
        }
    }
    let start = Instant::now();
    let mut check = 0_usize;
    for i in 0..reps {
        let out = black_box(run(seed + i as u64));
        check = check.wrapping_add(out.iter().sum::<usize>());
    }
    println!("{} {}", start.elapsed().as_nanos(), black_box(check));
}
