#![allow(clippy::pedantic, missing_docs)] // Standalone measurement driver; parsed fields declare units.
use membership_filters::{Bloom, Cuckoo, QuotientRuns, mix};
use std::{collections::BTreeSet, hint::black_box, time::Instant};
enum Candidate {
    Exact,
    Full(Bloom),
    Block(Bloom),
    Cuckoo(Cuckoo),
    Quotient(QuotientRuns),
}
impl Candidate {
    fn build(name: &str, keys: &[u64]) -> Self {
        match name {
            "exact" => Self::Exact,
            "bloom" | "blocked" => {
                let mut b = Bloom::new((keys.len() * 10).max(64), 7, name == "blocked");
                for &k in keys {
                    b.insert(k);
                }
                if name == "blocked" {
                    Self::Block(b)
                } else {
                    Self::Full(b)
                }
            }
            "cuckoo" => {
                let mut c = Cuckoo::new(keys.len().div_ceil(2).max(1).next_power_of_two());
                for &k in keys {
                    assert!(c.insert(k, 128), "complete admission failed");
                }
                Self::Cuckoo(c)
            }
            "quotient" => Self::Quotient(QuotientRuns::build(
                keys,
                keys.len().max(1).next_power_of_two(),
            )),
            _ => panic!("candidate"),
        }
    }
    #[inline(never)]
    fn maybe(&self, k: u64) -> bool {
        match self {
            Self::Exact => true,
            Self::Full(b) | Self::Block(b) => b.contains(k),
            Self::Cuckoo(c) => c.contains(k),
            Self::Quotient(q) => q.contains(k),
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Exact => 0,
            Self::Full(b) | Self::Block(b) => b.bytes(),
            Self::Cuckoo(c) => c.bytes(),
            Self::Quotient(q) => q.bytes(),
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        return;
    }
    let name = &args[1];
    let case = &args[2];
    let (n, hit, hot, build) = match case.as_str() {
        "tiny_miss" => (32, 0, false, false),
        "mid_miss" => (4096, 0, false, false),
        "mid_hit90" => (4096, 90, false, false),
        "large_miss" => (131072, 0, false, false),
        "large_hit90" => (131072, 90, false, false),
        "hot_miss" => (4096, 0, true, false),
        "build_mid" => (4096, 10, false, true),
        "first_batch" => (4096, 10, false, false),
        _ => panic!("case"),
    };
    let mut keys: Vec<_> = (0..n).map(|x| mix(x as u64 + 123)).collect();
    keys.sort_unstable();
    let oracle: BTreeSet<_> = keys.iter().copied().collect();
    let queries: Vec<_> = (0..8192)
        .map(|i| {
            if i % 100 < hit {
                keys[(mix(i as u64) as usize) % n]
            } else {
                mix(if hot { 900000 } else { 900000 + i as u64 })
            }
        })
        .collect();
    let expected: usize = queries.iter().filter(|k| oracle.contains(k)).count();
    let c = Candidate::build(name, &keys);
    assert!(keys.iter().all(|&k| c.maybe(k)));
    let mut fp = 0;
    let mut negatives = 0;
    let mut checks = 0;
    for &k in &queries {
        let t = oracle.contains(&k);
        let m = c.maybe(k);
        assert_eq!(m && keys.binary_search(&k).is_ok(), t);
        if !t {
            negatives += 1;
            if m {
                fp += 1;
            }
        }
        if m {
            checks += 1;
        }
    }
    let batch = || {
        queries
            .iter()
            .filter(|&&k| c.maybe(black_box(k)) && keys.binary_search(&black_box(k)).is_ok())
            .count()
    };
    if case != "first_batch" {
        for _ in 0..2 {
            assert_eq!(black_box(batch()), expected);
        }
    }
    let repetitions = if build {
        4
    } else if case == "first_batch" {
        1
    } else {
        12
    };
    let start = Instant::now();
    if build {
        for _ in 0..repetitions {
            black_box(keys.to_vec());
            black_box(Candidate::build(name, black_box(&keys)));
        }
    } else {
        for _ in 0..repetitions {
            assert_eq!(black_box(batch()), expected);
        }
    }
    let ns = start.elapsed().as_nanos();
    let operations = if build {
        repetitions * n
    } else {
        repetitions * queries.len()
    };
    println!(
        "OK ns={ns} operations={operations} bytes={} fp={fp} negatives={negatives} checks={checks} n={n} hits={expected}",
        c.bytes()
    );
}
