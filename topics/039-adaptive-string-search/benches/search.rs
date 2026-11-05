//! Fixed synthetic first-match benchmark. See BENCHMARK.md for timing boundaries.
use std::{hint::black_box, time::Instant};
use topic_039_adaptive_string_search::{Candidate, Finder, oracle};

fn main() {
    // Cargo appends `--bench`; the positional arguments keep their indices.
    let args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    if args.len() != 6 {
        eprintln!("usage: search direct|horspool|bm|tw|adaptive N M shape prepared|oneshot");
        // `cargo test --all-targets` runs this binary without arguments.
        if args.len() > 1 {
            std::process::exit(2);
        }
        return;
    }
    let candidate = match args[1].as_str() {
        "direct" => Candidate::Direct,
        "horspool" => Candidate::Horspool,
        "bm" => Candidate::BoyerMoore,
        "tw" => Candidate::TwoWay,
        "adaptive" => Candidate::Adaptive,
        _ => panic!("candidate"),
    };
    let text_len: usize = args[2].parse().expect("N");
    let pattern_len: usize = args[3].parse().expect("M");
    assert!(pattern_len > 0 && pattern_len <= text_len);
    let prepared = match args[5].as_str() {
        "prepared" => true,
        "oneshot" => false,
        _ => panic!("endpoint"),
    };
    let shape = &args[4];
    let mut seed = 39_u64;
    let sigma = if shape == "small" { 4 } else { 256 };
    let mut random = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        u8::try_from(seed % sigma).expect("byte")
    };
    let mut text: Vec<u8> = (0..text_len).map(|_| random()).collect();
    let mut pattern: Vec<u8> = (0..pattern_len).map(|_| random()).collect();
    match shape.as_str() {
        "wide" => text[text_len - pattern_len..].copy_from_slice(&pattern),
        "small" => {
            pattern[pattern_len - 1] = 255;
            text[text_len - pattern_len..].copy_from_slice(&pattern);
        }
        "prefix" => {
            text.fill(b'a');
            pattern.fill(b'a');
            pattern[pattern_len - 1] = b'b';
        }
        "suffix" => {
            text.fill(b'a');
            pattern.fill(b'a');
            pattern[0] = b'b';
        }
        "early" => text[..pattern_len].copy_from_slice(&pattern),
        _ => panic!("shape"),
    }
    let expected = oracle(&text, &pattern);
    if shape == "wide" || shape == "small" {
        assert_eq!(
            expected,
            Some(text_len - pattern_len),
            "the {shape} fixture matches before its planted suffix; choose another N or M"
        );
    }
    let finder = Finder::new(&pattern, candidate);
    assert_eq!(finder.find(&text), expected);
    let (answer, stats) = finder.diagnose(&text);
    assert_eq!(answer, expected);
    let iters = if shape == "early" {
        100_000
    } else {
        (500_000 / text_len).max(16)
    };
    let query = || {
        if prepared {
            finder.find(black_box(&text))
        } else {
            Finder::new(black_box(&pattern), candidate).find(black_box(&text))
        }
    };
    let mut checksum = 0_usize;
    for _ in 0..4 {
        black_box(query());
    }
    let before = Instant::now();
    for _ in 0..iters {
        checksum = checksum.wrapping_add(black_box(query()).unwrap_or(text_len));
    }
    let ns = before.elapsed().as_nanos();
    assert_eq!(checksum, expected.unwrap_or(text_len).wrapping_mul(iters));
    println!(
        "{{\"ns_total\":{},\"iterations\":{iters},\"comparisons\":{},\"windows\":{},\"fallbacks\":{},\"answer\":{:?},\"checksum\":{checksum}}}",
        ns,
        stats.comparisons,
        stats.windows,
        stats.fallbacks,
        expected.map_or(-1, |x| i64::try_from(x).expect("offset"))
    );
}
