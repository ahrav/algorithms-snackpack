//! Fixed workload candidate executable.
use merge_lab::{
    Row, balanced_cascade, gallop_merge, heap_cascade, left_cascade, linear_merge, parallel_merge,
    probe_join, replay_join, sort_cascade, sort_merge,
};
use std::hint::black_box;
use std::time::Instant;

fn rows(n: usize, kind: &str, side: u64) -> Vec<Row> {
    let mut seed = 0x1234_5678_9abc_def0_u64 + side;
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let key = match kind {
            "interleaved" => i as u64 * 2 + side,
            "disjoint" => i as u64 + side * (n as u64 + 1),
            "equal" => 5,
            "duplicates" => seed % 256,
            "sparse" => i as u64 * 4 + side,
            _ => seed % (n as u64 * 4 + 1),
        };
        v.push(Row {
            key,
            id: side * 10_000_000 + i as u64,
        });
    }
    v.sort_by_key(|r| r.key);
    v
}

#[allow(clippy::many_single_char_names, clippy::too_many_lines)]
pub(crate) fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        args.extend([
            "join_equal1024".to_owned(),
            "replay".to_owned(),
            "1".to_owned(),
        ]);
    }
    if args.len() != 4 {
        eprintln!("experiment WORKLOAD CANDIDATE ITERATIONS");
        std::process::exit(2);
    }
    let (work, candidate) = (args[1].as_str(), args[2].as_str());
    let iterations: usize = args[3].parse().unwrap();
    let (family, n, m, kind, k): (&str, usize, usize, &str, usize) = match work {
        "merge_cold32" | "merge_warm32" => ("merge", 16, 16, "random", 0),
        "merge_random4096" => ("merge", 2048, 2048, "random", 0),
        "merge_interleaved1m" => ("merge", 524_288, 524_288, "interleaved", 0),
        "merge_random1m" => ("merge", 524_288, 524_288, "random", 0),
        "merge_disjoint1m" => ("merge", 524_288, 524_288, "disjoint", 0),
        "merge_equal1m" => ("merge", 524_288, 524_288, "equal", 0),
        "merge_skew1m" => ("merge", 1024, 1_048_576, "random", 0),
        "join_cold64" => ("join", 32, 32, "random", 0),
        "join_sparse8192" => ("join", 4096, 4096, "sparse", 0),
        "join_duplicates8192" => ("join", 4096, 4096, "duplicates", 0),
        "join_equal1024" => ("join", 512, 512, "equal", 0),
        "cascade_tiny" => ("cascade", 16, 0, "random", 16),
        "cascade_4" => ("cascade", 4096, 0, "random", 4),
        "cascade_32" => ("cascade", 4096, 0, "duplicates", 32),
        "cascade_skew" => ("cascade", 32768, 0, "random", 16),
        _ => panic!("unknown workload"),
    };
    let a = rows(n, kind, 0);
    let b = rows(m, kind, 1);
    let runs: Vec<_> = (0..k)
        .map(|r| {
            rows(
                if work == "cascade_skew" && r > 0 {
                    32
                } else {
                    n
                },
                kind,
                r as u64,
            )
        })
        .collect();
    let merge = || match candidate {
        "sort" => sort_merge(black_box(&a), black_box(&b)),
        "linear" => linear_merge(black_box(&a), black_box(&b)),
        "gallop" => gallop_merge(black_box(&a), black_box(&b)),
        "parallel4" => parallel_merge(black_box(&a), black_box(&b), 4),
        _ => panic!("candidate"),
    };
    let join = || match candidate {
        "probe" => probe_join(black_box(&a), black_box(&b)),
        "replay" => replay_join(black_box(&a), black_box(&b)),
        _ => panic!("candidate"),
    };
    let cascade = || match candidate {
        "sort" => sort_cascade(black_box(&runs)),
        "left" => left_cascade(black_box(&runs)),
        "balanced" => balanced_cascade(black_box(&runs)),
        "heap" => heap_cascade(black_box(&runs)),
        _ => panic!("candidate"),
    };
    // Independent oracle runs before timing; no counters attributed to kernels.
    let output_len = match family {
        "merge" => {
            let out = merge();
            assert_eq!(out, sort_merge(&a, &b));
            out.len()
        }
        "join" => {
            let mut want = Vec::new();
            let mut groups = std::collections::BTreeMap::<u64, Vec<u64>>::new();
            for x in &b {
                groups.entry(x.key).or_default().push(x.id);
            }
            for x in &a {
                if let Some(ys) = groups.get(&x.key) {
                    for &y in ys {
                        want.push((x.id, y));
                    }
                }
            }
            let out = join();
            assert_eq!(out, want);
            out.len()
        }
        _ => {
            let out = cascade();
            assert_eq!(out, sort_cascade(&runs));
            out.len()
        }
    };
    let warmups = if work.contains("cold") { 0 } else { 3 };
    for _ in 0..warmups {
        match family {
            "merge" => {
                black_box(merge());
            }
            "join" => {
                black_box(join());
            }
            _ => {
                black_box(cascade());
            }
        }
    }
    let start = Instant::now();
    for _ in 0..iterations {
        match family {
            "merge" => {
                black_box(merge());
            }
            "join" => {
                black_box(join());
            }
            _ => {
                black_box(cascade());
            }
        }
    }
    let ns = start.elapsed().as_nanos();
    println!(
        "{{\"workload\":\"{work}\",\"candidate\":\"{candidate}\",\"iterations\":{iterations},\"warmups\":{warmups},\"ns\":{ns},\"output_len\":{output_len}}}"
    );
}
