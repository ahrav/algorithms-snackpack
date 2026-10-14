//! Compare the bounded quadratic reference with complete sorting on tiny inputs.
use hash_joins::{merge, nested};
use std::{hint::black_box, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Run measurements/tiny_reference.py for the paired experiment.");
        return;
    }
    let a: Vec<u64> = (0_u64..8).map(|i| i * 7919 % 17).collect();
    let b: Vec<u64> = (0_u64..12).map(|i| i * 3571 % 17).collect();
    let mut frequencies = std::collections::BTreeMap::<u64, u128>::new();
    for key in &a {
        *frequencies.entry(*key).or_default() += 1;
    }
    let expected: u128 = b
        .iter()
        .map(|key| *frequencies.get(key).unwrap_or(&0))
        .sum();
    assert_eq!(expected, 5);
    let run = || match args[1].as_str() {
        "nested" => nested(black_box(&a), black_box(&b)),
        "merge" => merge(black_box(&a), black_box(&b)),
        _ => panic!("unknown candidate"),
    };
    for _ in 0..2 {
        assert_eq!(black_box(run()), expected);
    }
    let start = Instant::now();
    for _ in 0..100_000 {
        assert_eq!(black_box(run()), expected);
    }
    println!(
        "OK ns={} operations=100000 pairs=5",
        start.elapsed().as_nanos()
    );
}
