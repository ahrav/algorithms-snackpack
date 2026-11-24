//! Time one fixed batch several times in a fresh process.

use std::{env, hint::black_box, time::Instant};

use deficit_round_robin_and_fair_queueing::{Flow, Policy, dispatch, validate};

fn main() {
    let args: Vec<_> = env::args().collect();
    let (count, depth, reps) = match args[1].as_str() {
        "tiny" => (2, 4, 200_000),
        "equal" | "skew" => (16, 512, 256),
        "weighted-wide" => (256, 64, 128),
        _ => panic!("unknown workload"),
    };
    let mut state = 631_u32;
    let queues: Vec<Vec<_>> = (0..count)
        .map(|flow| {
            (0..depth)
                .map(|_| match args[1].as_str() {
                    "tiny" | "skew" => {
                        if flow % 2 == 0 {
                            400
                        } else {
                            1500
                        }
                    }
                    "equal" => 512,
                    "weighted-wide" => {
                        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        64 + (state >> 16) % 1437
                    }
                    _ => unreachable!(),
                })
                .collect()
        })
        .collect();
    let flows: Vec<_> = queues
        .iter()
        .enumerate()
        .map(|(i, lengths)| Flow {
            lengths,
            weight: if count == 256 { [1, 2, 3, 4][i % 4] } else { 1 },
        })
        .collect();
    let policy = match args[2].as_str() {
        "rr" => Policy::PacketRoundRobin,
        "drr" => Policy::DeficitRoundRobin { quantum: 1500 },
        "wfq" => Policy::WeightedFairQueueing,
        _ => panic!("unknown policy"),
    };
    validate(&flows);
    let start = Instant::now();
    let mut digest = 0_u64;
    let mut work = 0;
    for _ in 0..reps {
        work += dispatch(black_box(&flows), black_box(policy), |flow, packet| {
            digest = digest.rotate_left(9) ^ flow as u64 ^ (packet as u64).wrapping_mul(17);
        });
        digest = black_box(digest).wrapping_add(1);
    }
    let ns = start.elapsed().as_nanos();
    println!("{ns} {} {work} {digest}", count * depth * reps);
}
