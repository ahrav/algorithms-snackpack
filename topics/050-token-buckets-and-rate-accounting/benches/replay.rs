//! Standalone replay. Compile using rustc or run as a Cargo benchmark.
#[path = "../src/lib.rs"]
mod rate;
use rate::{Bounded, Meter, Virtual, Wide};
use std::{hint::black_box, time::Instant};
type Event = (usize, u64, u64);
fn workload(n: usize, flows: usize, pattern: &str) -> Vec<Event> {
    (0..n)
        .map(|i| {
            let round = u64::try_from(i / flows).unwrap();
            let time = match pattern {
                "burst" => round / 1024,
                "idle" => round * 100,
                "weighted" => round / 2,
                _ => round,
            };
            let cost = if pattern == "weighted" {
                [0, 1, 6, 11, 4, 10, 3][i % 7]
            } else {
                6
            };
            (i % flows, time, cost)
        })
        .collect()
}
fn replay<M: Meter>(events: &[Event], flows: usize, details: bool) -> (u64, Vec<(bool, u64)>) {
    let mut meters: Vec<M> = (0..flows).map(|_| M::new(10, 3)).collect();
    let mut digest = 0_u64;
    let mut output = Vec::new();
    for &(flow, time, cost) in events {
        let (ok, credit) = meters[flow]
            .admit(black_box(time), black_box(cost))
            .unwrap();
        digest = digest
            .wrapping_mul(1_000_003)
            .wrapping_add(credit * 2 + u64::from(ok));
        if details {
            output.push((ok, credit));
        }
    }
    black_box(&meters);
    (digest, output)
}
fn run<M: Meter>(events: &[Event], flows: usize, reps: usize, details: bool) {
    let start = Instant::now();
    let mut digest = 0;
    for _ in 0..reps {
        digest = black_box(replay::<M>(black_box(events), flows, false).0);
    }
    let ns = start.elapsed().as_nanos();
    let (_, output) = if details {
        replay::<M>(events, flows, true)
    } else {
        (0, Vec::new())
    };
    let output = output
        .iter()
        .map(|(ok, credit)| format!("[{ok},{credit}]"))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "{{\"ns\":{ns},\"reps\":{reps},\"digest\":{digest},\"state_bytes\":{},\"output\":[{output}]}}",
        std::mem::size_of::<M>() * flows
    );
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 6 {
        println!("usage: replay MODE N FLOWS PATTERN REPS [details]");
        return;
    }
    let n = a[2].parse().unwrap();
    let flows = a[3].parse().unwrap();
    let events = workload(n, flows, &a[4]);
    let reps = a[5].parse().unwrap();
    match a[1].as_str() {
        "wide" => run::<Wide>(&events, flows, reps, a.len() > 6),
        "bounded" => run::<Bounded>(&events, flows, reps, a.len() > 6),
        "virtual" => run::<Virtual>(&events, flows, reps, a.len() > 6),
        _ => panic!("unknown candidate"),
    }
}
