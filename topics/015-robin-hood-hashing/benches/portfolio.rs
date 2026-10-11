//! Controlled candidate-per-process throughput workload.
use robin_hood_hashing::{HashMode, Policy, Table};
use std::{collections::BTreeMap, hint::black_box, time::Instant};

fn step(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn setup(policy: Policy, cap: usize, n: usize, mode: HashMode) -> Table {
    let mut t = Table::new(cap, policy, mode);
    for k in 0..n as u64 {
        t.insert(k, k.wrapping_mul(3)).unwrap();
    }
    t
}

#[allow(clippy::too_many_lines)] // Keep the frozen timed boundary visible in one function.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: portfolio linear|robin|scan WORKLOAD SEED");
        return;
    }
    let policy = match args[1].as_str() {
        "linear" => Policy::Linear,
        "robin" => Policy::Robin,
        "scan" => Policy::RobinScan,
        _ => panic!("candidate"),
    };
    let name = args[2].as_str();
    let (cap, load, hit, mode, reps) = match name {
        "tiny" => (32, 50, 100, HashMode::Mixed, 16),
        "small_hit" => (2048, 50, 100, HashMode::Mixed, 16),
        "high_hit" | "skew" => (32_768, 87, 100, HashMode::Mixed, 8),
        "high_miss" => (32_768, 87, 0, HashMode::Mixed, 8),
        "high_mix" | "build" => (32_768, 87, 50, HashMode::Mixed, 8),
        "large_mix" => (262_144, 87, 50, HashMode::Mixed, 8),
        "same_home" => (256, 87, 50, HashMode::Constant, 2),
        "churn" => (4096, 87, 50, HashMode::Mixed, 8),
        "first_batch" => (32_768, 87, 50, HashMode::Mixed, 1),
        _ => panic!("workload"),
    };
    let n = cap * load / 100;
    let mut state = args[3].parse::<u64>().unwrap();
    let queries: Vec<u64> = (0..16_384)
        .map(|_| {
            let x = step(&mut state);
            let mut k = (x >> 16) % n as u64;
            if name == "skew" && !x.is_multiple_of(10) {
                k %= 8;
            }
            if x % 100 >= hit {
                k += n as u64;
            }
            k
        })
        .collect();
    let oracle: BTreeMap<u64, u64> = (0..n as u64).map(|k| (k, k * 3)).collect();
    let check = setup(policy, cap, n, mode);
    for &k in &queries {
        assert_eq!(check.get(k), oracle.get(&k).copied());
    }
    assert!(check.valid());
    // Validate the exact mutation contract independently before accepting a timing.
    if name == "churn" {
        let mut t = check.clone();
        let mut expected = oracle.clone();
        for &q in &queries {
            let k = q % n as u64;
            assert_eq!(t.remove(k), expected.remove(&k));
            assert_eq!(t.insert(k, q), Ok(expected.insert(k, q)));
        }
        assert!(t.valid());
        for (&k, &v) in &expected {
            assert_eq!(t.get(k), Some(v));
        }
    }
    let mut counts: Vec<usize> = queries.iter().map(|&k| check.get_counted(k).1).collect();
    counts.sort_unstable();
    let probes = counts.iter().sum::<usize>();
    let p95 = counts[counts.len() * 95 / 100];
    let max = *counts.last().unwrap();
    let mut t = setup(policy, cap, n, mode);
    if name != "first_batch" {
        for &k in &queries {
            black_box(t.get(k));
        }
    }
    let start = Instant::now();
    let mut sum = 0_u64;
    for round in 0..reps {
        if name == "build" {
            let built = setup(policy, cap, n, mode);
            sum = sum.wrapping_add(black_box(built.get(17)).unwrap_or(0));
            black_box(built); // Allocation, insertion and teardown inside boundary.
        } else if name == "churn" {
            for &q in &queries {
                let k = q % n as u64;
                sum = sum.wrapping_add(black_box(t.remove(k)).unwrap());
                black_box(t.insert(k, q + round)).unwrap();
                sum = sum.wrapping_add(black_box(t.get(q)).unwrap_or(0));
            }
        } else {
            for &q in &queries {
                sum = sum.wrapping_add(black_box(t.get(black_box(q))).unwrap_or(0));
            }
        }
    }
    let ns = start.elapsed().as_nanos();
    black_box(sum);
    let units = if name == "build" {
        n * usize::try_from(reps).unwrap()
    } else {
        queries.len() * usize::try_from(reps).unwrap()
    };
    println!(
        "{{\"candidate\":\"{}\",\"workload\":\"{}\",\"ns\":{},\"units\":{},\"checksum\":{},\"query_probes\":{},\"probe_p95\":{},\"probe_max\":{},\"capacity\":{},\"live\":{}}}",
        args[1], name, ns, units, sum, probes, p95, max, cap, n
    );
}
