//! Replay one policy and report dispatch timings and complete decision vectors.

use std::{env, fs, hint::black_box, time::Instant};
use topic_053_stable_assignment::{Dispatch, Fleet, Policy, Request};

/// Run the replay and print results.
pub fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 6, "compare POLICY BACKENDS PENALTY REPS TRACE");
    let policy = match args[1].as_str() {
        "rr" => Policy::RoundRobin,
        "one" => Policy::One,
        "two" => Policy::Two,
        "scan" => Policy::Scan,
        "hrw" => Policy::Rendezvous,
        "local" => Policy::Local,
        p => panic!("unknown policy: {p}"),
    };
    let backends = args[2].parse().unwrap();
    let penalty = args[3].parse().unwrap();
    let reps: u32 = args[4].parse().unwrap();
    assert!(reps > 0);
    let setup = Instant::now();
    let trace = fs::read_to_string(&args[5]).unwrap();
    let trace: Vec<_> = trace
        .lines()
        .map(|line| {
            let mut value = line.split_ascii_whitespace();
            let request = Request {
                at: value.next().unwrap().parse().unwrap(),
                service: value.next().unwrap().parse().unwrap(),
                key: value.next().unwrap().parse().unwrap(),
                zone: value.next().unwrap().parse().unwrap(),
                sample: [
                    value.next().unwrap().parse().unwrap(),
                    value.next().unwrap().parse().unwrap(),
                ],
            };
            assert!(value.next().is_none());
            request
        })
        .collect();
    let mut fleet = Fleet::new(backends, penalty);
    let mut output = vec![Dispatch::default(); trace.len()];
    let setup_ns = setup.elapsed().as_nanos();
    let mut dispatch_ns = 0;
    for _ in 0..reps {
        fleet.reset();
        let start = Instant::now();
        for (&request, slot) in trace.iter().zip(&mut output) {
            *slot = fleet.dispatch(policy, request);
        }
        black_box(&output);
        dispatch_ns += start.elapsed().as_nanos();
    }
    println!("{setup_ns} {dispatch_ns}");
    for Dispatch {
        backend,
        wait,
        response,
    } in output
    {
        println!("{backend} {wait} {response}");
    }
}
