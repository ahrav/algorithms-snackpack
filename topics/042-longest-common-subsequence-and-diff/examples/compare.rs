//! One process sample; setup and oracle are outside the timed region.
use std::{hint::black_box, time::Instant};
use topic_042_lcs_diff::{Candidate, Edit, diff, oracle, valid, workload};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(1).map_or("matrix", String::as_str);
    let shape = args.get(2).map_or("sparse", String::as_str);
    let n = args.get(3).map_or(64, |x| x.parse().unwrap());
    let reps: usize = args.get(4).map_or(1, |x| x.parse().unwrap());
    assert!(reps > 0);
    let candidate = match name {
        "matrix" => Candidate::Matrix,
        "hirschberg" => Candidate::Hirschberg,
        "myers" => Candidate::Myers,
        _ => panic!("unknown candidate"),
    };
    let (a, b) = workload(shape, n);
    let cost = oracle(&a, &b);
    let (script, stats) = diff(&a, &b, candidate);
    assert!(valid(&a, &b, &script, cost));
    let start = Instant::now();
    for _ in 0..reps {
        let (result, counters) = black_box(diff(black_box(&a), black_box(&b), candidate));
        black_box((&result, &counters));
        drop(result);
    }
    let elapsed = start.elapsed().as_nanos();
    let kept = script.iter().filter(|x| matches!(x, Edit::Keep(_))).count();
    println!(
        "{{\"candidate\":\"{name}\",\"shape\":\"{shape}\",\"m\":{},\"n\":{},\"cost\":{cost},\"kept\":{kept},\"reps\":{reps},\"ns_total\":{elapsed},\"states\":{},\"comparisons\":{},\"score_slots\":{},\"score_allocations\":{}}}",
        a.len(),
        b.len(),
        stats.states,
        stats.comparisons,
        stats.score_slots,
        stats.score_allocations
    );
}
