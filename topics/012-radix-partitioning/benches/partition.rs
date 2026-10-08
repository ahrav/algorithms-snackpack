//! One candidate per process. Oracle and setup are outside the timing boundary.
#![forbid(unsafe_code)]
use partition::{Digit, Row, check, run};
use radix_partitioning as partition;
use std::{hint::black_box, time::Instant};

fn experiment<const P: usize>(
    name: &str,
    n: usize,
    bits: u32,
    shape: &str,
    reps: usize,
    cold: bool,
) {
    let mut seed = 0x1234_5678_9abc_def0_u64;
    let mut input: Vec<_> = (0..n)
        .map(|i| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let key = match shape {
                "skew" if i % 10 != 0 => 5,
                "same" => 5,
                "zero" => seed << 12,
                _ => seed,
            };
            Row {
                key,
                id: i as u64,
                payload: [seed.wrapping_mul(17); P],
            }
        })
        .collect();
    let digit = Digit::new(0, bits);
    if shape == "grouped" {
        input.sort_by_key(|r| digit.bucket(r.key));
    }
    let setup = Instant::now();
    if !cold {
        for _ in 0..2 {
            black_box(run(name, &input, digit));
        }
    }
    let warmup_ns = setup.elapsed().as_nanos();
    let start = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..reps {
        let result = run(name, black_box(&input), digit);
        checksum ^= result.rows.first().map_or(0, |r| r.id)
            ^ result.bounds.last().copied().unwrap_or(0) as u64;
        black_box(&result);
        drop(result);
    }
    let elapsed = start.elapsed().as_nanos();
    check(&input, &run(name, &input, digit), digit, name != "cycles");
    println!(
        "{{\"candidate\":\"{name}\",\"n\":{n},\"bits\":{bits},\"shape\":\"{shape}\",\"width\":{},\"reps\":{reps},\"cold\":{cold},\"elapsed_ns\":{elapsed},\"warmup_ns\":{warmup_ns},\"checksum\":{checksum},\"oracle\":\"pass\"}}",
        std::mem::size_of::<Row<P>>()
    );
}

fn main() {
    let a: Vec<_> = std::env::args().collect();
    if a.len() == 1 || (a.len() == 2 && a[1] == "--test") {
        for name in ["buckets", "scatter", "two_pass", "cycles"] {
            experiment::<0>(name, 32, 2, "uniform", 1, false);
        }
        return;
    }
    assert_eq!(a.len(), 8, "candidate n bits shape width reps cold");
    let n = a[2].parse().unwrap();
    let bits = a[3].parse().unwrap();
    let reps = a[6].parse().unwrap();
    let cold = a[7] == "1";
    if a[5] == "64" {
        experiment::<6>(&a[1], n, bits, &a[4], reps, cold);
    } else {
        experiment::<0>(&a[1], n, bits, &a[4], reps, cold);
    }
}
