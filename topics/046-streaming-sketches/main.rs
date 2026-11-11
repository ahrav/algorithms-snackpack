//! Standalone batch CLI for streaming-sketch experiments.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "bounded experiment dimensions"
)]
#[path = "src/lib.rs"]
mod sketches;
use std::{env, hint::black_box, time::Instant};
fn main() {
    let args: Vec<String> = env::args().collect();
    let candidate = args.get(1).map_or("count-min", String::as_str);
    let n: usize = args.get(2).map_or(Ok(128), |x| x.parse()).unwrap();
    let shape = args.get(3).map_or("uniform", String::as_str);
    let seed: u64 = args.get(4).map_or(Ok(7), |x| x.parse()).unwrap();
    let repeats: usize = args.get(5).map_or(Ok(1), |x| x.parse()).unwrap();
    let data: Vec<u64> = (0..n as u64)
        .map(|i| match shape {
            "uniform" => sketches::mix(i, 123) % 4096,
            "skew" => {
                if i % 10 < 9 {
                    0
                } else {
                    sketches::mix(i, 123) % 4096
                }
            }
            "sorted" => i,
            "reverse" => n as u64 - 1 - i,
            _ => panic!("unknown shape"),
        })
        .collect();
    let queries: Vec<u64> = (0..4096).collect();
    let mut elapsed = 0_u128;
    let mut result = None;
    for _ in 0..repeats {
        let start = Instant::now();
        let report =
            sketches::evaluate(black_box(&data), black_box(&queries), candidate, seed).unwrap();
        black_box(&report);
        elapsed += start.elapsed().as_nanos();
        result = Some(report); // previous drop excluded; summary destruction is inside evaluate
    }
    let r = result.unwrap();
    println!(
        "{{\"candidate\":{candidate:?},\"n\":{n},\"shape\":{shape:?},\"seed\":{seed},\"repeats\":{repeats},\"ns\":{},\"bytes\":{},\"estimate\":{},\"values\":{:?},\"state\":{:?}}}",
        elapsed as f64 / repeats as f64,
        r.retained_bytes,
        r.estimate,
        r.values,
        r.state
    );
}
