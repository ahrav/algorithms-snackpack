//! Print byte shares and a fixed-arrival congestion witness.

use deficit_round_robin_and_fair_queueing::{
    Arrival, Flow, Outcome, PersistentMark, Policy, dispatch, replay_fifo, validate,
};

fn main() {
    let lengths_a = [400; 32];
    let lengths_b = [1500; 32];
    let flows = [
        Flow {
            lengths: &lengths_a,
            weight: 1,
        },
        Flow {
            lengths: &lengths_b,
            weight: 1,
        },
    ];
    validate(&flows);
    let cutoff = 5_800_u32;
    println!("Fluid service at tick {cutoff}: [2900, 2900] bytes");
    for (name, policy) in [
        ("rr", Policy::PacketRoundRobin),
        ("drr", Policy::DeficitRoundRobin { quantum: 1500 }),
        ("wfq", Policy::WeightedFairQueueing),
    ] {
        let mut elapsed = 0_u32;
        let mut served = [0_u32; 2];
        let mut first = Vec::new();
        dispatch(&flows, policy, |flow, packet| {
            let length = flows[flow].lengths[packet];
            served[flow] += length.min(cutoff.saturating_sub(elapsed));
            elapsed += length;
            if first.len() < 8 {
                first.push(flow);
            }
        });
        println!("{name}: service={served:?}; first_flows={first:?}");
    }
    let arrivals = [Arrival {
        tick: 0,
        bytes: 400,
    }; 4];
    let cap = 1_600;
    let plain = replay_fifo(&arrivals, cap, None);
    let marks = replay_fifo(
        &arrivals,
        cap,
        Some(PersistentMark {
            threshold: 600,
            interval: 400,
        }),
    );
    let starts = marks
        .iter()
        .map(|outcome| match outcome {
            Outcome::Sent { start, .. } => *start,
            Outcome::TailDrop => panic!("fixture has sufficient capacity"),
        })
        .collect::<Vec<_>>();
    assert_eq!(starts, [0, 400, 800, 1200]);
    assert_eq!(
        marks[3],
        Outcome::Sent {
            start: 1200,
            marked: true
        }
    );
    println!("tail cap: {plain:?}");
    println!("with marks: {marks:?}");
}
