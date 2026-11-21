//! CLI and timing driver for the isotonic-regression lab.
#![forbid(unsafe_code)]

use isotonic_regression::{Block, MAX_LEN, hull, shifting, stack};
use std::hint::black_box;
use std::io::{self, BufRead};
use std::time::Instant;

type Fit = fn(&[i32]) -> Vec<Block>;

fn algorithm(name: &str) -> Fit {
    match name {
        "shift" => shifting,
        "stack" => stack,
        "hull" => hull,
        _ => panic!("unknown algorithm {name}"),
    }
}

fn workload(size: usize, shape: &str) -> Vec<i32> {
    assert!(size <= MAX_LEN);
    let mut seed = 0xd510_6d47_u32;
    (0..size)
        .map(|i| {
            let i = i32::try_from(i).unwrap();
            match shape {
                "up" => i - 19,
                "fall" => 19 - i,
                "plateau" => i / 32 - 19,
                "noise" => {
                    seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    i / 16 + i32::try_from((seed >> 16) % 201).unwrap() - 100
                }
                _ => panic!("unknown shape {shape}"),
            }
        })
        .collect()
}

fn encode(blocks: &[Block]) -> String {
    blocks
        .iter()
        .map(|b| format!("{}:{}", b.sum, b.len))
        .collect::<Vec<_>>()
        .join(" ")
}

fn digest(blocks: &[Block]) -> u64 {
    blocks.iter().fold(0xcbf2_9ce4_8422_2325, |hash, block| {
        let sum_bits = u64::from_ne_bytes(block.sum.to_ne_bytes());
        (hash ^ sum_bits).wrapping_mul(0x100_0000_01b3) ^ u64::try_from(block.len).unwrap()
    })
}

fn benchmark(fit: Fit, input: &[i32], repetitions: u32) {
    assert!(repetitions > 0);
    for _ in 0..32 {
        black_box(fit(black_box(input)));
    }
    let start = Instant::now();
    for _ in 0..repetitions {
        black_box(fit(black_box(input)));
    }
    let nanos = start.elapsed().as_nanos();
    println!("{} {}", nanos, digest(&fit(input)));
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args[1..]
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["example"] => {
            let input = [6, 2, 3, 8];
            for name in ["shift", "stack", "hull"] {
                println!("{name} {}", encode(&algorithm(name)(&input)));
            }
        }
        ["fit", name] => {
            let fit = algorithm(name);
            for line in io::stdin().lock().lines() {
                let line = line.unwrap();
                let input: Vec<i32> = line
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                println!("{}", encode(&fit(&input)));
            }
        }
        ["input", size, shape] => {
            let input = workload(size.parse().unwrap(), shape);
            println!(
                "{}",
                input
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        ["bench", name, size, shape, repetitions] => {
            let input = workload(size.parse().unwrap(), shape);
            benchmark(algorithm(name), &input, repetitions.parse().unwrap());
        }
        _ => panic!(
            "usage: example | fit ALGORITHM | input SIZE SHAPE | bench ALGORITHM SIZE SHAPE REPS"
        ),
    }
}
