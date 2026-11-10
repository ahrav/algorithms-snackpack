//! Cargo uses this target as a benchmark smoke check; measurements/run.py runs the timed schedule.
//!
//! Cargo passes `--bench` to `harness = false` targets, so this entry point
//! ignores command-line arguments.
use std::hint::black_box;
use topic045::{MODES, decode, encode};

fn main() {
    let want = [1000, 1001, 1002, 1003];
    // Complete frame lengths of the README running example, in mode order.
    let lengths = [21, 22, 21, 16, 21, 14];
    for (mode, length) in MODES.into_iter().zip(lengths) {
        let frame = encode(black_box(mode), black_box(&want));
        assert_eq!(frame.len(), length, "{mode:?}");
        assert_eq!(decode(black_box(&frame), want.len()).unwrap(), want);
        black_box(frame);
    }
}
