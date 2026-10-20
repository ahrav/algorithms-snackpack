#![allow(missing_docs)]
use std::{hint::black_box, time::Instant};
use topic024_tries_radix_trees_and_art::{Entry, Index, oracle, workload};

fn fold(mut sum: u64, values: &[u64]) -> u64 {
    for &v in values {
        sum = sum.wrapping_mul(31).wrapping_add(v.wrapping_add(1));
    }
    sum
}
fn queries(case: &str, rows: &[Entry]) -> Vec<Vec<u8>> {
    (0..64)
        .map(|i| {
            let mut k = rows[(i * 127) % rows.len()].0.clone();
            match case {
                "miss" => k[0] = 255,
                "prefix256" => {
                    k.truncate(k.len() - 1);
                }
                "prefix_all" => k.truncate(7),
                _ => {
                    if i % 2 == 1 {
                        k.push(255);
                    }
                }
            }
            k
        })
        .collect()
}
fn operate(index: &Index, q: &[Vec<u8>], prefix: bool) -> u64 {
    let mut sum = 0;
    for k in q {
        if prefix {
            let out = index.prefix(black_box(k));
            sum = fold(sum, black_box(&out));
        } else {
            sum = fold(sum, &[index.get(black_box(k)).unwrap_or(u64::MAX)]);
        }
    }
    black_box(sum)
}
fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        args.extend(["flat".to_owned(), "tiny".to_owned()]);
    }
    if args.len() != 3 {
        eprintln!("usage: dictionary CANDIDATE CASE");
        std::process::exit(2);
    }
    let name = &args[1];
    let case = &args[2];
    let shape = if case == "wide" || case == "build_wide" {
        "wide"
    } else {
        "shared"
    };
    let n = if case == "tiny" { 32 } else { 8192 };
    let rows = workload(shape, n);
    let q = queries(case, &rows);
    let prefix = case.starts_with("prefix");
    let build = case.starts_with("build");
    let index = Index::new(name, &rows);
    for k in &q {
        if prefix {
            assert_eq!(
                index.prefix(k),
                oracle(&rows, k).iter().map(|e| e.1).collect::<Vec<_>>()
            );
        } else {
            assert_eq!(
                index.get(k),
                rows.iter().rev().find(|e| e.0 == *k).map(|e| e.1)
            );
        }
    }
    let layout = index.layout();
    let expected = operate(&index, &q, prefix);
    let loops = if build || case == "prefix_all" {
        1
    } else if case == "tiny" {
        400
    } else {
        40
    };
    // Release the correctness index before timing a complete build/query/drop operation.
    let index = if build {
        drop(index);
        None
    } else {
        Some(index)
    };
    let run = || {
        let mut checksum = 0_u64;
        for _ in 0..loops {
            let value = if build {
                let x = Index::new(name, black_box(&rows));
                operate(&x, &q, prefix)
            } else {
                operate(index.as_ref().unwrap(), &q, prefix)
            };
            assert_eq!(value, expected);
            checksum = checksum.wrapping_add(value);
        }
        black_box(checksum)
    };
    run(); // one complete operation batch warms the selected path
    let mut ns = Vec::new();
    for _ in 0..3 {
        let t = Instant::now();
        run();
        ns.push(t.elapsed().as_nanos());
    }
    ns.sort_unstable();
    println!(
        "{{\"candidate\":\"{name}\",\"case\":\"{case}\",\"ns\":{},\"subsamples_ns\":{:?},\"loops\":{loops},\"queries\":64,\"n\":{n},\"nodes\":{},\"capacity_bytes\":{},\"families\":{:?},\"checksum\":{expected},\"oracle\":true}}",
        ns[1], ns, layout.0, layout.1, layout.2
    );
}
