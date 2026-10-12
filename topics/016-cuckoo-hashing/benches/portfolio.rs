//! Hash-inclusive warmed queries, complete builds and delete/reinsert pairs.
#![allow(clippy::cast_precision_loss)]
use cuckoo_hashing::{Cuckoo, Layout, Linear, Table};
use std::{hint::black_box, time::Instant};
fn config(name: &str) -> (usize, usize, usize, bool) {
    match name {
        "tiny_hit" => (32, 10, 4096, false),
        "small_hit" | "small_miss" | "hot" => (4096, 1400, 32768, false),
        "large_hit" | "large_miss" => (1_048_576, 360_000, 65536, false),
        "high_hit" | "high_miss" => (131_072, 104_857, 65536, false),
        "build35" => (65536, 22000, 1, true),
        "build80" => (65536, 52000, 1, true),
        "churn" => (8192, 2800, 8192, false),
        _ => panic!("unknown workload"),
    }
}
fn populate<T: Table>(table: &mut T, n: usize) -> usize {
    let mut rejected = 0;
    for k in 0..u64::try_from(n).unwrap() {
        if table.try_insert(k, k.wrapping_mul(3)).is_err() {
            rejected += 1;
        }
    }
    rejected
}
fn query(index: usize, n: usize, name: &str) -> u64 {
    if name == "hot" {
        return u64::try_from(index % 16).unwrap();
    }
    let perm = index.wrapping_mul(15_485_863) % n;
    u64::try_from(perm + if name.ends_with("miss") { n } else { 0 }).unwrap()
}
fn cycle<T: Table>(table: &mut T, name: &str, n: usize, q: usize, build: bool) -> u64 {
    let mut checksum = 0u64;
    if build {
        for k in 0..u64::try_from(n).unwrap() {
            checksum = checksum.wrapping_add(u64::from(
                black_box(table.try_insert(black_box(k), k.wrapping_mul(3))).is_ok(),
            ));
        }
    } else if name == "churn" {
        for i in 0..q {
            let key = u64::try_from(i % n).unwrap();
            checksum = checksum.wrapping_add(black_box(table.remove(black_box(key))).unwrap_or(0));
            checksum = checksum.wrapping_add(u64::from(
                black_box(table.try_insert(key, key.wrapping_mul(3))).is_ok(),
            ));
        }
    } else {
        for i in 0..q {
            checksum = checksum
                .wrapping_add(black_box(table.get(black_box(query(i, n, name)))).unwrap_or(0));
        }
    }
    black_box(checksum)
}
fn run<T: Table>(name: &str, make: impl Fn() -> T) {
    let (_, n, q, build) = config(name);
    let mut table = make();
    cuckoo_hashing::check(make(), 32);
    let rejected = if build {
        let mut check = make();
        populate(&mut check, n)
    } else {
        populate(&mut table, n)
    };
    if rejected > 0 {
        println!("REJECT {name} failures={rejected} requested={n}");
        return;
    }
    // Independently verify values and expected query checksum outside timing.
    let oracle: std::collections::BTreeMap<_, _> =
        (0..u64::try_from(n).unwrap()).map(|k| (k, k * 3)).collect();
    if !build {
        assert_eq!(
            table.entries(),
            oracle.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>()
        );
    }
    let expected = if build {
        u64::try_from(n).unwrap()
    } else if name == "churn" {
        (0..q).map(|i| u64::try_from(i % n).unwrap() * 3 + 1).sum()
    } else {
        (0..q)
            .map(|i| oracle.get(&query(i, n, name)).copied().unwrap_or(0))
            .sum()
    };
    drop(oracle);
    let rounds = if build {
        12
    } else if name.starts_with("large") || name.starts_with("high") {
        32
    } else {
        128
    };
    let mut total = 0u64;
    if build {
        // Boundary includes allocation, initialization, insertion and destruction.
        for _ in 0..2 {
            let mut t = make();
            assert_eq!(cycle(&mut t, name, n, q, true), expected);
        }
    } else {
        for _ in 0..2 {
            assert_eq!(cycle(&mut table, name, n, q, false), expected);
        }
    }
    let start = Instant::now();
    for _ in 0..rounds {
        let got = if build {
            let mut t = make();
            cycle(&mut t, name, n, q, true)
        } else {
            cycle(&mut table, name, n, q, false)
        };
        assert_eq!(got, expected);
        total = total.wrapping_add(got);
    }
    let ns = start.elapsed().as_nanos();
    let operations = rounds * if build { n } else { q };
    let counts: Vec<_> = if build {
        Vec::new()
    } else {
        (0..q).map(|i| table.examined(query(i, n, name))).collect()
    };
    let sum: usize = counts.iter().sum();
    let max = counts.iter().max().copied().unwrap_or(0);
    println!(
        "OK ns={ns} operations={operations} checksum={total} mean_slots={} max_slots={max} bytes_slots={}",
        if counts.is_empty() {
            0.0
        } else {
            sum as f64 / counts.len() as f64
        },
        table.capacity() * std::mem::size_of::<Option<cuckoo_hashing::Entry>>()
    );
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let candidate = args.get(1).map_or("linear", String::as_str);
    let workload = args.get(2).map_or("small_hit", String::as_str);
    let cap = config(workload).0;
    match candidate {
        "linear" => run(workload, || Linear::new(cap, Layout::Mixed)),
        "cuckoo1" => run(workload, || Cuckoo::<1>::new(cap, Layout::Mixed, 64)),
        "cuckoo4" => run(workload, || Cuckoo::<4>::new(cap, Layout::Mixed, 64)),
        _ => panic!("unknown candidate"),
    }
}
