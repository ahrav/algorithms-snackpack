//! Independent round, fluid-time, and tick-by-tick queue oracles.

use deficit_round_robin_and_fair_queueing::{
    Arrival, Flow, Outcome, PersistentMark, Policy, dispatch, replay_fifo, validate,
};

type Packet = (usize, usize);

fn order(flows: &[Flow<'_>], policy: Policy) -> (Vec<Packet>, usize) {
    let mut out = Vec::new();
    let visits = dispatch(flows, policy, |flow, packet| out.push((flow, packet)));
    (out, visits)
}

fn packet_rounds(flows: &[Flow<'_>]) -> Vec<Packet> {
    let depth = flows
        .iter()
        .map(|flow| flow.lengths.len())
        .max()
        .unwrap_or(0);
    (0..depth)
        .flat_map(|packet| {
            flows.iter().enumerate().filter_map(move |(flow, queue)| {
                (packet < queue.lengths.len()).then_some((flow, packet))
            })
        })
        .collect()
}

fn byte_rounds(flows: &[Flow<'_>], quantum: u32, carry: bool) -> (Vec<Packet>, usize) {
    let mut trace = Vec::new();
    let mut sent = vec![0; flows.len()];
    let mut next = vec![0; flows.len()];
    let mut visits = 0;
    let mut round = 0_u64;
    while next
        .iter()
        .zip(flows)
        .any(|(&index, flow)| index < flow.lengths.len())
    {
        round += 1;
        for (flow, queue) in flows.iter().enumerate() {
            if next[flow] == queue.lengths.len() {
                continue;
            }
            visits += 1;
            let budget = u64::from(quantum) * u64::from(queue.weight);
            let ceiling = if carry {
                round * budget
            } else {
                sent[flow] + budget
            };
            let mut cumulative = sent[flow];
            for (packet, &length) in queue.lengths.iter().enumerate().skip(next[flow]) {
                cumulative += u64::from(length);
                if cumulative > ceiling {
                    break;
                }
                trace.push((flow, packet));
                sent[flow] = cumulative;
                next[flow] += 1;
            }
        }
    }
    (trace, visits)
}

fn fluid_departures(flows: &[Flow<'_>]) -> Vec<(Packet, u64)> {
    // Work and wall time use twelfths. Every weight used in this oracle
    // divides twelve, so each packet boundary lands exactly on a step.
    let mut next = vec![0; flows.len()];
    let mut left: Vec<_> = flows
        .iter()
        .map(|f| 12 * f.lengths.first().copied().unwrap_or(0))
        .collect();
    let mut wall = 0;
    let mut completions = Vec::new();
    while left.iter().any(|&bytes| bytes > 0) {
        wall += flows
            .iter()
            .zip(&left)
            .filter(|(_, bytes)| **bytes > 0)
            .map(|(flow, _)| u64::from(flow.weight))
            .sum::<u64>();
        for (flow, queue) in flows.iter().enumerate() {
            if left[flow] == 0 {
                continue;
            }
            left[flow] -= queue.weight;
            if left[flow] == 0 {
                completions.push(((flow, next[flow]), wall));
                next[flow] += 1;
                left[flow] = 12 * queue.lengths.get(next[flow]).copied().unwrap_or(0);
            }
        }
    }
    completions
}

fn next_seed(seed: &mut u32, modulus: u32) -> u32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (*seed >> 12) % modulus
}

#[test]
fn generated_batches_match_independent_models() {
    let mut seed = 23;
    for _ in 0..384 {
        let count = next_seed(&mut seed, 6);
        let lengths: Vec<Vec<_>> = (0..count)
            .map(|_| {
                (0..next_seed(&mut seed, 8))
                    .map(|_| 1 + next_seed(&mut seed, 12))
                    .collect()
            })
            .collect();
        let flows: Vec<_> = lengths
            .iter()
            .map(|lengths| Flow {
                lengths,
                weight: 1 + next_seed(&mut seed, 4),
            })
            .collect();
        validate(&flows);
        let expected_rr = packet_rounds(&flows);
        let rr_count = expected_rr.len();
        assert_eq!(
            order(&flows, Policy::PacketRoundRobin),
            (expected_rr, rr_count)
        );
        for quantum in [1, 5, 12] {
            assert_eq!(
                order(&flows, Policy::DeficitRoundRobin { quantum }),
                byte_rounds(&flows, quantum, true)
            );
        }

        let expected = fluid_departures(&flows);
        let (wfq, selections) = order(&flows, Policy::WeightedFairQueueing);
        assert_eq!(selections, expected.len());
        assert_eq!(
            wfq,
            expected
                .iter()
                .map(|(packet, _)| *packet)
                .collect::<Vec<_>>()
        );
        let max = flows
            .iter()
            .flat_map(|flow| flow.lengths)
            .copied()
            .max()
            .unwrap_or(0);
        let mut wall = 0;
        for ((flow, index), fluid_finish) in expected {
            wall += 12 * u64::from(flows[flow].lengths[index]);
            assert!(wall <= fluid_finish + 12 * u64::from(max));
        }
    }
}

#[test]
fn carry_changes_round_two_and_tiny_quantum_costs_visits() {
    let flows = [
        Flow {
            lengths: &[400; 12],
            weight: 1,
        },
        Flow {
            lengths: &[1500; 6],
            weight: 1,
        },
    ];
    let correct = order(&flows, Policy::DeficitRoundRobin { quantum: 1500 }).0;
    assert_eq!(
        correct[..9],
        [
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (1, 1)
        ]
    );
    assert_ne!(correct, byte_rounds(&flows, 1500, false).0);
    let single = [Flow {
        lengths: &[1500],
        weight: 1,
    }];
    assert_eq!(
        order(&single, Policy::DeficitRoundRobin { quantum: 375 }),
        (vec![(0, 0)], 4)
    );
    assert_eq!(
        order(&single, Policy::DeficitRoundRobin { quantum: 1500 }),
        (vec![(0, 0)], 1)
    );
}

fn tick_fifo(arrivals: &[Arrival], capacity: u64, policy: Option<PersistentMark>) -> Vec<Outcome> {
    let mut result = vec![Outcome::TailDrop; arrivals.len()];
    let mut waiting = Vec::new();
    let mut starts = Vec::new();
    let mut link_free = 0;
    let end = arrivals.last().map_or(0, |arrival| arrival.tick)
        + arrivals
            .iter()
            .map(|arrival| u64::from(arrival.bytes))
            .sum::<u64>();
    for tick in 0..=end {
        for (index, arrival) in arrivals.iter().enumerate().filter(|(_, a)| a.tick == tick) {
            let used: u64 = waiting
                .iter()
                .map(|&i: &usize| u64::from(arrivals[i].bytes))
                .sum();
            if used + u64::from(arrival.bytes) <= capacity {
                waiting.push(index);
            }
        }
        if tick < link_free || waiting.is_empty() {
            continue;
        }
        let index = waiting.remove(0);
        starts.push((tick, tick - arrivals[index].tick));
        let marked = policy.is_some_and(|mark| {
            starts
                .iter()
                .rev()
                .take_while(|(_, sojourn)| *sojourn >= mark.threshold)
                .any(|(start, _)| tick - start >= mark.interval)
        });
        result[index] = Outcome::Sent {
            start: tick,
            marked,
        };
        link_free = tick + u64::from(arrivals[index].bytes);
    }
    result
}

#[test]
fn queue_marks_and_byte_limit_match_tick_oracle() {
    let mut seed = 17;
    for _ in 0..512 {
        let mut tick = 0;
        let arrivals: Vec<_> = (0..next_seed(&mut seed, 13))
            .map(|_| {
                tick += u64::from(next_seed(&mut seed, 4));
                Arrival {
                    tick,
                    bytes: 1 + next_seed(&mut seed, 6),
                }
            })
            .collect();
        let cap = u64::from(next_seed(&mut seed, 19));
        let policy = PersistentMark {
            threshold: 1 + u64::from(next_seed(&mut seed, 8)),
            interval: u64::from(next_seed(&mut seed, 7)),
        };
        let plain = replay_fifo(&arrivals, cap, None);
        let marked = replay_fifo(&arrivals, cap, Some(policy));
        assert_eq!(plain, tick_fifo(&arrivals, cap, None));
        assert_eq!(marked, tick_fifo(&arrivals, cap, Some(policy)));
        let cleared: Vec<_> = marked
            .into_iter()
            .map(|outcome| match outcome {
                Outcome::TailDrop => outcome,
                Outcome::Sent { start, .. } => Outcome::Sent {
                    start,
                    marked: false,
                },
            })
            .collect();
        assert_eq!(plain, cleared);
    }
}

#[test]
fn simultaneous_arrivals_and_persistence_boundary() {
    let arrivals = [Arrival {
        tick: 0,
        bytes: 400,
    }; 4];
    let mark = PersistentMark {
        threshold: 600,
        interval: 400,
    };
    let sent = |start, marked| Outcome::Sent { start, marked };
    assert_eq!(
        replay_fifo(&arrivals, 1600, Some(mark)),
        [
            sent(0, false),
            sent(400, false),
            sent(800, false),
            sent(1200, true)
        ]
    );
    assert_eq!(
        replay_fifo(&arrivals, 1599, Some(mark)),
        [
            sent(0, false),
            sent(400, false),
            sent(800, false),
            Outcome::TailDrop
        ]
    );
    assert_eq!(
        replay_fifo(
            &[
                Arrival { tick: 7, bytes: 5 },
                Arrival { tick: 12, bytes: 5 }
            ],
            5,
            Some(mark)
        ),
        [sent(7, false), sent(12, false)]
    );
}
