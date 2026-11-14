//! Standalone process boundary: generate outside timing, replay inside timing.
use std::{hint::black_box, time::Instant};
use topic_049_timers::{Candidate, replay, workload};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: replay scan|heap|tree|wheel IDS near|far|burst 0|90".into());
    }
    let candidate = match args[1].as_str() {
        "scan" => Candidate::Scan,
        "heap" => Candidate::Heap,
        "tree" => Candidate::Tree,
        "wheel" => Candidate::Wheel,
        _ => return Err("unknown candidate".into()),
    };
    let ids = args[2].parse()?;
    let events = workload(ids, &args[3], args[4].parse()?)?;
    // The untimed first replay warms code and yields the externally checked result.
    let first = replay(candidate, ids, &events)?;
    let repetitions = 16;
    let start = Instant::now();
    for _ in 0..repetitions {
        let value = replay(black_box(candidate), black_box(ids), black_box(&events))?;
        black_box(&value);
        // Teardown belongs to the timing boundary.
        drop(value);
    }
    let ns = start.elapsed().as_nanos() / repetitions;
    println!(
        "{{\"ns\":{ns},\"fired\":{:?},\"live\":{:?},\"inspections\":{},\"ticks\":{},\"peak_records\":{}}}",
        first
            .fired
            .iter()
            .map(|&(p, d, id)| [u64::from(p), u64::from(d), id as u64])
            .collect::<Vec<_>>(),
        first
            .live
            .iter()
            .map(|&(id, d)| [id as u64, u64::from(d)])
            .collect::<Vec<_>>(),
        first.work.inspections,
        first.work.ticks,
        first.work.peak_records
    );
    Ok(())
}
