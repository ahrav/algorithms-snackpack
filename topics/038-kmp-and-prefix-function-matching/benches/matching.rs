//! Frozen workload runner. One process emits one candidate's aggregate timing.
use std::{hint::black_box, time::Instant};
use topic038_kmp_and_prefix_function_matching::{Dfa, Kmp, direct, oracle};

fn random(n: usize, alphabet: u8, seed: u64) -> Vec<u8> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            b'a' + u8::try_from(x % u64::from(alphabet)).unwrap()
        })
        .collect()
}

fn workload(name: &str) -> (Vec<u8>, Vec<u8>, usize) {
    match name {
        "tiny" => (
            b"abababacababaca--ababaca--ababaca!".to_vec(),
            b"aba".to_vec(),
            2000,
        ),
        "random" => (random(8192, 26, 17), random(16, 26, 91), 20),
        "large" => (random(131_072, 26, 19), random(64, 26, 93), 2),
        "near" => {
            let mut p = vec![b'a'; 64];
            p[63] = b'b';
            (vec![b'a'; 8192], p, 20)
        }
        "dense" => (vec![b'a'; 8192], vec![b'a'; 64], 20),
        "binary" => (random(8192, 2, 23), random(16, 2, 97), 20),
        _ => panic!("unknown workload"),
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: matching CASE direct|kmp|dfa|kmp-aa total|prepared");
        std::process::exit(2);
    }
    let (text, pattern, iters) = workload(&args[1]);
    let expected = oracle(&text, &pattern).unwrap();
    let k = Kmp::new(&pattern).unwrap();
    let d = Dfa::new(&pattern).unwrap();
    assert_eq!(direct(&text, &pattern).unwrap(), expected);
    assert_eq!(k.find_all(&text), expected);
    assert_eq!(d.find_all(&text), expected);
    let run = || match (args[2].as_str(), args[3].as_str()) {
        ("direct", _) => direct(black_box(&text), black_box(&pattern)).unwrap(),
        ("kmp" | "kmp-aa", "prepared") => k.find_all(black_box(&text)),
        ("dfa", "prepared") => d.find_all(black_box(&text)),
        ("kmp" | "kmp-aa", "total") => Kmp::new(black_box(&pattern))
            .unwrap()
            .find_all(black_box(&text)),
        ("dfa", "total") => Dfa::new(black_box(&pattern))
            .unwrap()
            .find_all(black_box(&text)),
        _ => panic!("unknown candidate or boundary"),
    };
    for _ in 0..3 {
        black_box(run());
    }
    let start = Instant::now();
    for _ in 0..iters {
        black_box(run());
    }
    let ns = start.elapsed().as_nanos();
    println!(
        "{{\"case\":\"{}\",\"candidate\":\"{}\",\"boundary\":\"{}\",\"n\":{},\"m\":{},\"z\":{},\"iters\":{},\"elapsed_ns\":{},\"direct_comparisons\":{},\"kmp_comparisons\":{}}}",
        args[1],
        args[2],
        args[3],
        text.len(),
        pattern.len(),
        expected.len(),
        iters,
        ns,
        topic038_kmp_and_prefix_function_matching::direct_comparisons(&text, &pattern).unwrap(),
        k.comparisons(&text)
    );
}
