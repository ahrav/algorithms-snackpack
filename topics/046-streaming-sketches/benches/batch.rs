//! Standalone batch benchmark entry.
use topic_046_streaming_sketches as sketches;
fn main() {
    let data: Vec<u64> = (0..16384).map(|i| sketches::mix(i, 123)).collect();
    let r = sketches::evaluate(&data, &[4], "count-min", 7).unwrap();
    std::hint::black_box(r);
}
