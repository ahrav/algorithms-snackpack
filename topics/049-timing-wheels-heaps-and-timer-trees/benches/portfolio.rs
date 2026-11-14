//! Smoke benchmark. Use run.py for the declared fresh-process comparison.
use std::{hint::black_box, time::Instant};
use topic_049_timers::{Candidate, replay, workload};
fn main() {
    let events = workload(4096, "near", 90).expect("fixed workload");
    for c in [
        Candidate::Scan,
        Candidate::Heap,
        Candidate::Tree,
        Candidate::Wheel,
    ] {
        let t = Instant::now();
        black_box(replay(c, 4096, &events).expect("valid workload"));
        println!("{c:?}: {:?}", t.elapsed());
    }
}
