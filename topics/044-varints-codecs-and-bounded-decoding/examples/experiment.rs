//! Frozen decode-only experiment. Allocation, encoding, and validation precede timing.
#![allow(
    clippy::cast_precision_loss,
    reason = "elapsed nanoseconds are reported as floating point"
)]
use std::{hint::black_box, time::Instant};
use topic044::{encode, fixed, reference, scalar, unrolled};

type Decoder = fn(&[u8], usize, &mut Vec<u32>) -> Result<(), topic044::Error>;

fn data(case: usize) -> Vec<u32> {
    let n = if case < 4 { 32 } else { 65_536 };
    let mut state = 42u32;
    (0..n)
        .map(|i| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            match case % 4 {
                0 => state & 127,
                1 => [
                    127,
                    128,
                    16383,
                    16384,
                    (1 << 21) - 1,
                    1 << 21,
                    (1 << 28) - 1,
                    1 << 28,
                ][i % 8],
                2 => state,
                _ => {
                    if i % 2 == 0 {
                        1
                    } else {
                        u32::MAX
                    }
                }
            }
        })
        .collect()
}

pub(crate) fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_none_or(|x| x == "example") {
        let v = vec![1, 127, 128, 300, u32::MAX];
        let bytes = encode(&v);
        let mut out = vec![];
        scalar(&bytes, 5, &mut out).unwrap();
        assert_eq!(out, v);
        println!("{bytes:02x?} -> {out:?}; bytes={} fixed=20", bytes.len());
        return;
    }
    let case: usize = args[2].parse().unwrap();
    assert!(case < 8);
    let values = data(case);
    let encoded = encode(&values);
    let fixed_bytes: Vec<_> = values.iter().flat_map(|x| x.to_le_bytes()).collect();
    let mut out = Vec::with_capacity(values.len());
    reference(&encoded, values.len(), &mut out).unwrap();
    assert_eq!(out, values);
    for candidate in [scalar, unrolled] {
        candidate(&encoded, values.len(), &mut out).unwrap();
        assert_eq!(out, values);
    }
    fixed(&fixed_bytes, values.len(), &mut out).unwrap();
    assert_eq!(out, values);
    if args[1] == "dump" {
        println!(
            "{{\"values\":{values:?},\"encoded\":{encoded:?},\"fixed\":{fixed_bytes:?},\"decoded\":{out:?}}}"
        );
        return;
    }
    let (candidate, input): (Decoder, &[u8]) = match args[1].as_str() {
        "scalar" => (scalar, &encoded),
        "unrolled" => (unrolled, &encoded),
        "fixed" => (fixed, &fixed_bytes),
        _ => panic!("candidate"),
    };
    let reps = if values.len() == 32 { 100_000 } else { 50 };
    let start = Instant::now();
    for _ in 0..reps {
        candidate(black_box(input), values.len(), black_box(&mut out)).unwrap();
        black_box(&out);
    }
    let ns = start.elapsed().as_nanos() as f64 / f64::from(reps);
    println!(
        "{{\"candidate\":\"{}\",\"case\":{case},\"n\":{},\"bytes\":{},\"reps\":{reps},\"ns\":{ns}}}",
        args[1],
        values.len(),
        input.len()
    );
}
