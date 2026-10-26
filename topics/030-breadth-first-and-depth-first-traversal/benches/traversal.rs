//! Standalone controlled traversal benchmark.
#[path = "../src/lib.rs"]
#[allow(dead_code)]
mod traversal;
use std::hint::black_box;
use std::time::Instant;
use traversal::{
    Csr, Graph, UNREACHABLE, bfs_csr, bfs_deque, bfs_layers, bfs_vec, dfs_events, dfs_frames,
    distance_oracle,
};

fn workload(name: &str) -> (Graph, usize) {
    let (n, reps) = match name {
        "tiny" => (6, 50_000),
        "path_small" | "star_small" => (256, 2000),
        "path" | "star" => (32_768, 24),
        "grid" | "permuted_grid" => (16_384, 32),
        "sparse" => (32_768, 12),
        "dense" => (1_024, 24),
        "disconnected" => (32_768, 300),
        _ => panic!("unknown workload"),
    };
    let mut adj = vec![vec![]; n];
    let mut seed = 123_u64;
    match name {
        "tiny" => adj = vec![vec![1, 2], vec![3], vec![3], vec![1, 4], vec![], vec![]],
        "path" | "path_small" => {
            for (u, row) in adj.iter_mut().enumerate().take(n - 1) {
                row.push(u + 1);
            }
        }
        "star" | "star_small" => adj[0].extend(1..n),
        "grid" | "permuted_grid" => {
            for u in 0..n {
                let p = |v| {
                    if name == "permuted_grid" {
                        (v * 8191) % n
                    } else {
                        v
                    }
                };
                if u % 128 < 127 {
                    adj[p(u)].push(p(u + 1));
                }
                if u / 128 < 127 {
                    adj[p(u)].push(p(u + 128));
                }
            }
        }
        "sparse" | "dense" => {
            let degree = if name == "dense" { 64 } else { 4 };
            for (u, row) in adj.iter_mut().enumerate() {
                row.push((u + 1) % n);
                for _ in 1..degree {
                    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    row.push(usize::try_from(seed >> 32).unwrap() % n);
                }
            }
        }
        "disconnected" => {
            for (u, row) in adj.iter_mut().enumerate().take(255) {
                row.push(u + 1);
            }
        }
        _ => unreachable!(),
    }
    (Graph::new(adj).unwrap(), reps)
}
fn main() {
    let mut args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    if args.len() == 1 {
        args.extend(["tiny".to_owned(), "deque".to_owned()]);
    }
    let case = &args[1];
    let candidate = &args[2];
    let setup = Instant::now();
    let (g, reps) = workload(case);
    let build_ns = setup.elapsed().as_nanos();
    let setup = Instant::now();
    let csr = Csr::new(&g);
    let csr_ns = setup.elapsed().as_nanos();
    let signature = (0..g.len()).fold(0_u64, |h, u| {
        g.neighbors(u).iter().fold(h, |a, &v| {
            a.wrapping_mul(1_099_511_628_211)
                .wrapping_add(u64::try_from(u).unwrap())
                .wrapping_add(u64::try_from(v).unwrap())
        })
    });
    let oracle = distance_oracle(&g, &[0]);
    let bfs = || match candidate.as_str() {
        "deque" | "deque_a" | "deque_b" => bfs_deque(black_box(&g), black_box(&[0])),
        "vector" => bfs_vec(black_box(&g), black_box(&[0])),
        "csr" => bfs_csr(black_box(&csr), black_box(&[0])),
        "layers" => bfs_layers(black_box(&csr), black_box(&[0])),
        _ => panic!("candidate"),
    };
    let dfs = || match candidate.as_str() {
        "frames" => dfs_frames(black_box(&g), black_box(&[0])),
        "events" => dfs_events(black_box(&g), black_box(&[0])),
        _ => panic!("candidate"),
    };
    let is_dfs = matches!(candidate.as_str(), "frames" | "events");
    if is_dfs {
        let got = dfs();
        assert_eq!(got, dfs_frames(black_box(&g), black_box(&[0])));
        assert_eq!(got, dfs_events(black_box(&g), black_box(&[0])));
        let mut reached = vec![false; g.len()];
        for &v in &got.preorder {
            assert!(!reached[v]);
            reached[v] = true;
        }
        for (v, &d) in oracle.iter().enumerate() {
            assert_eq!(reached[v], d != UNREACHABLE);
        }
    } else {
        assert_eq!(bfs(), oracle);
    }
    let run = || {
        if is_dfs {
            drop(black_box(dfs()));
        } else {
            drop(black_box(bfs()));
        }
    };
    let first = Instant::now();
    run();
    let first_ns = first.elapsed().as_nanos();
    let start = Instant::now();
    for _ in 0..reps {
        run();
    }
    let total_ns = start.elapsed().as_nanos();
    println!(
        "{{\"case\":\"{case}\",\"candidate\":\"{candidate}\",\"n\":{},\"edges\":{},\"reached\":{},\"signature\":{signature},\"reps\":{reps},\"total_ns\":{total_ns},\"first_ns\":{first_ns},\"build_ns\":{build_ns},\"csr_ns\":{csr_ns}}}",
        g.len(),
        g.edges(),
        oracle.iter().filter(|&&d| d != UNREACHABLE).count()
    );
}
