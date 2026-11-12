//! Standalone batch benchmark entry.
use std::hint::black_box;
use topic_046_streaming_sketches as sketches;
fn main() {
    let data: Vec<u64> = (0..16384).map(|i| sketches::mix(i, 123)).collect();
    let queries = [data[0], data[16383]];
    let low = *data.iter().min().unwrap();
    let high = *data.iter().max().unwrap();
    for candidate in [
        "exact-frequency",
        "count-min",
        "exact-distinct",
        "hll",
        "exact-quantile",
        "rank-summary",
    ] {
        let r = sketches::evaluate(black_box(&data), black_box(&queries), candidate, 7).unwrap();
        match candidate {
            "exact-frequency" => assert_eq!(r.values, [1, 1]),
            "count-min" => assert!(r.values.len() == 2 && r.values.iter().all(|&count| count >= 1)),
            "exact-distinct" => assert_eq!(r.estimate.to_bits(), 16384.0_f64.to_bits()),
            "hll" => assert!((r.estimate - 16384.0).abs() <= 1638.4, "{}", r.estimate),
            _ => assert_eq!(
                (r.values.len(), r.values[0], r.values[100]),
                (101, low, high),
                "{candidate}"
            ),
        }
        black_box(r);
    }
}
