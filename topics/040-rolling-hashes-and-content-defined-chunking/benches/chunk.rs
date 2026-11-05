//! Measure complete chunking calls over generated input.
use std::{hint::black_box, time::Instant};
use topic_040_chunking::{Config, Policy, boundaries, fixture};
fn main() {
    // Cargo appends `--bench`; the positional arguments keep their indices.
    let args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    let name = args.get(1).map_or("rolling", String::as_str);
    let n = args.get(2).map_or(Ok(256), |s| s.parse()).expect("length");
    let target = args.get(3).map_or(Ok(256), |s| s.parse()).expect("target");
    let shape = args.get(4).map_or("random", String::as_str);
    let policy = match name {
        "fixed" => Policy::Fixed,
        "direct" => Policy::Direct,
        "rolling" => Policy::Rolling,
        "gear" => Policy::Gear,
        "normalized" => Policy::Normalized,
        _ => panic!("unknown policy"),
    };
    let config = Config::new(target / 4, target, target * 4, 32)
        .expect("config: target is a power of two >= 128 so that window 32 <= min = target / 4");
    let input = fixture(n, shape);
    let ends = boundaries(&input, config, policy);
    let mut edited = input.clone();
    edited.splice(n / 3..n / 3, 0_u8..17);
    let edited_ends = boundaries(&edited, config, policy);
    let iterations = if n < 1024 {
        512
    } else if policy == Policy::Direct {
        2
    } else {
        16
    };
    for _ in 0..2 {
        black_box(boundaries(black_box(&input), config, policy));
    }
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(boundaries(black_box(&input), config, policy));
    }
    let ns = start.elapsed().as_nanos();
    println!(
        "{{\"ns_total\":{ns},\"iterations\":{iterations},\"ends\":{ends:?},\"edited_ends\":{edited_ends:?}}}"
    );
}
