//! Allocating end-to-end block encode or decode experiment.
#![allow(
    clippy::cast_precision_loss,
    reason = "timings use fractional nanoseconds"
)]
use std::{hint::black_box, time::Instant};
use topic045::{MODES, decode, encode};

fn data(case: usize) -> Vec<u32> {
    let n = [16, 1024, 65536][case / 4];
    let mut state = 42u32;
    (0..n)
        .map(|i| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            match case % 4 {
                0 => 1000 + u32::try_from(i).unwrap(),
                1 => 1000,
                2 => state,
                _ => {
                    if i % 2 == 0 {
                        0x1234_0000
                    } else {
                        0x1234_0001
                    }
                }
            }
        })
        .collect()
}

pub(crate) fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 || args[1] == "example" {
        for mode in MODES {
            let frame = encode(mode, &[1000, 1001, 1002, 1003]);
            assert_eq!(decode(&frame, 4).unwrap(), [1000, 1001, 1002, 1003]);
            println!("{mode:?}: {} bytes {frame:02x?}", frame.len());
        }
        return;
    }
    let case: usize = args[2].parse().unwrap();
    assert!(case < 12);
    let values = data(case);
    let mode: usize = args[3].parse().unwrap();
    let frame = encode(MODES[mode], &values);
    assert_eq!(decode(&frame, values.len()).unwrap(), values);
    if args[1] == "dump" {
        println!("{{\"values\":{values:?},\"frame\":{frame:?}}}");
        return;
    }
    let reps = (262_144 / values.len()).clamp(4, 4096);
    let start = Instant::now();
    for _ in 0..reps {
        if args[1] == "encode" {
            black_box(encode(black_box(MODES[mode]), black_box(&values)));
        } else {
            black_box(decode(black_box(&frame), values.len()).unwrap());
        }
    }
    println!(
        "{{\"case\":{case},\"mode\":{mode},\"operation\":\"{}\",\"n\":{},\"bytes\":{},\"reps\":{reps},\"ns\":{}}}",
        args[1],
        values.len(),
        frame.len(),
        start.elapsed().as_nanos() as f64 / reps as f64
    );
}
