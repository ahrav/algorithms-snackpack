//! Native fixed-workload reachability benchmark.
use std::{hint::black_box, time::Instant};
use topic035::{Closure, Dag, Intervals, Scratch, search};

#[allow(clippy::too_many_lines)]
fn main() {
    // Cargo appends `--bench`; a bare invocation runs one smoke cell.
    let mut args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|a| a != "--bench")
        .collect();
    if args.is_empty() {
        args = vec!["dfs".into(), "sparse".into()];
    }
    assert_eq!(
        args.len(),
        2,
        "usage: reachability <dfs|filter|closure|dfs-control> <tiny|sparse|dense|chain|negative|few|large-sparse>"
    );
    let candidate = &args[0];
    let workload = &args[1];
    assert!(
        ["dfs", "filter", "closure", "dfs-control"].contains(&candidate.as_str()),
        "unknown candidate {candidate}"
    );
    let n = match workload.as_str() {
        "tiny" => 32,
        "sparse" | "dense" => 512,
        "chain" | "negative" | "few" | "large-sparse" => 4096,
        _ => panic!("unknown workload {workload}"),
    };
    let q = if workload == "few" { 8 } else { 1024 };
    let mut edges = vec![vec![]; n];
    let mut seed = 35_u64;
    for (u, row) in edges.iter_mut().enumerate() {
        if ["chain", "few", "negative"].contains(&workload.as_str()) {
            if u + 1 < n && (workload != "negative" || u != n / 2 - 1) {
                row.push(u + 1);
            }
        } else {
            for v in u + 1..n {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                if (seed >> 32).is_multiple_of(if workload == "dense" {
                    4
                } else if n > 512 {
                    1024
                } else {
                    64
                }) {
                    row.push(v);
                }
            }
        }
    }
    let m = edges.iter().map(Vec::len).sum::<usize>();
    let g = Dag::new(edges).unwrap();
    let queries: Vec<_> = (0..q)
        .map(|i| {
            if workload == "negative" {
                (i % (n / 2), n / 2 + (i * 17) % (n / 2))
            } else {
                ((i * 37) % n, (i * 71 + 3) % n)
            }
        })
        .collect();
    // Independent oracle is outside all timing boundaries.
    let expected: Vec<bool> = queries.iter().map(|&(u, v)| g.oracle(u)[v]).collect();
    let positive = expected.iter().map(|&e| u64::from(e)).sum::<u64>();
    let start = Instant::now();
    let mut scratch = if candidate == "closure" {
        None
    } else {
        Some(Scratch::new(n))
    };
    let closure = if candidate == "closure" {
        Some(Closure::new(&g, 64 * 1024 * 1024).unwrap())
    } else {
        None
    };
    let filter = if candidate == "filter" {
        Some(Intervals::new(&g))
    } else {
        None
    };
    let build = start.elapsed().as_nanos();
    let bytes =
        closure.as_ref().map_or(0, Closure::bytes) + filter.as_ref().map_or(0, Intervals::bytes);
    let mut answer = |u, v| {
        if let Some(c) = &closure {
            c.reaches(black_box(u), black_box(v))
        } else {
            search(
                &g,
                scratch.as_mut().unwrap(),
                black_box(u),
                black_box(v),
                filter.as_ref(),
            )
        }
    };
    // Per-query validation catches false positives and false negatives that cancel in the batch sum.
    for (&(u, v), &e) in queries.iter().zip(&expected) {
        assert_eq!(
            answer(u, v),
            e,
            "{candidate} disagrees with the oracle on {u}->{v}"
        );
    }
    let mut batch = || {
        let mut sum = 0_u64;
        for &(u, v) in black_box(&queries) {
            sum += u64::from(answer(u, v));
        }
        black_box(sum)
    };
    let repeats = 8_u128;
    let start = Instant::now();
    for _ in 0..repeats {
        assert_eq!(batch(), positive);
    }
    let query_ns = start.elapsed().as_nanos() / repeats;
    let drop_start = Instant::now();

    drop(scratch);
    drop(closure);
    drop(filter);
    let teardown_ns = drop_start.elapsed().as_nanos();
    println!(
        "{{\"candidate\":\"{candidate}\",\"workload\":\"{workload}\",\"n\":{n},\"m\":{m},\"q\":{q},\"build_ns\":{build},\"query_ns\":{query_ns},\"teardown_ns\":{teardown_ns},\"index_bytes\":{bytes},\"positive\":{positive}}}"
    );
}
