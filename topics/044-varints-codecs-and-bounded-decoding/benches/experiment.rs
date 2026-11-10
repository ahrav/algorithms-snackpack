//! Cargo uses this target as a benchmark smoke check.
//!
//! Cargo passes `--bench` to `harness = false` targets, so this entry point
//! ignores command-line arguments.
use std::hint::black_box;
use topic044::{fixed, scalar, unrolled};

type Decoder = fn(&[u8], usize, &mut Vec<u32>) -> Result<(), topic044::Error>;

fn main() {
    let want = [1, 127, 128, 300, u32::MAX];
    let varint = [1, 127, 128, 1, 172, 2, 255, 255, 255, 255, 15];
    let fixed32: Vec<u8> = want.iter().flat_map(|x| x.to_le_bytes()).collect();
    let runs: [(Decoder, &[u8]); 3] = [(scalar, &varint), (unrolled, &varint), (fixed, &fixed32)];
    for (decode, input) in runs {
        let mut out = Vec::new();
        decode(black_box(input), want.len(), &mut out).unwrap();
        assert_eq!(out, want);
        black_box(out);
    }
}
