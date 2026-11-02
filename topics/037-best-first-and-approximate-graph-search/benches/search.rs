//! Standalone deterministic graph workload and query-only batch timer.
use std::hint::black_box;
use std::time::Instant;
use topic037_graph_search::Graph;

fn next(seed: &mut u64) -> i32 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    i32::try_from((*seed >> 32) % 2001).unwrap() - 1000
}
fn build(case: usize) -> (Graph, Vec<Vec<i32>>) {
    let (n, d, m, cluster, disconnect) = match case {
        0 => (32, 2, 4, false, false),
        1 => (64, 16, 16, false, false),
        2 => (1024, 2, 4, false, false),
        3 => (1024, 2, 16, false, false),
        4 => (1024, 16, 4, false, false),
        5 => (1024, 16, 16, false, false),
        6 => (1024, 16, 16, true, false),
        7 => (1024, 2, 16, true, true),
        _ => panic!("case must be 0..7"),
    };
    let mut seed = 37;
    let points: Vec<Vec<i32>> = (0..n)
        .map(|i| {
            (0..d)
                .map(|_| {
                    let x = next(&mut seed);
                    if cluster {
                        x / 10 + if i < n / 2 { -3000 } else { 3000 }
                    } else {
                        x
                    }
                })
                .collect()
        })
        .collect();
    let edges = (0..n)
        .map(|i| {
            let allowed = |j: usize| !disconnect || (j < n / 2) == (i < n / 2);
            let mut ranked: Vec<_> = (0..n)
                .filter(|&j| j != i && allowed(j))
                .map(|j| {
                    let dist: i64 = points[i]
                        .iter()
                        .zip(&points[j])
                        .map(|(&a, &b)| (i64::from(a) - i64::from(b)).pow(2))
                        .sum();
                    (dist, j)
                })
                .collect();
            ranked.sort_unstable();
            let mut row: Vec<_> = ranked.iter().take(m - 2).map(|x| x.1).collect();
            for j in [(i + 1) % n, (i + n - 1) % n] {
                if allowed(j) && !row.contains(&j) {
                    row.push(j);
                }
            }
            row
        })
        .collect();
    let queries = (0..32)
        .map(|_| {
            (0..d)
                .map(|_| {
                    let x = next(&mut seed);
                    if cluster { x / 10 + 3000 } else { x }
                })
                .collect()
        })
        .collect();
    (Graph::new(points, edges).unwrap(), queries)
}
fn execute(g: &Graph, q: &[i32], method: &str) -> (Vec<(i64, usize)>, topic037_graph_search::Work) {
    if method == "flat" {
        g.exact(q, 4).unwrap()
    } else {
        g.search(q, 4, method.parse().unwrap(), 0).unwrap()
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = args.get(1).map_or(0, |x| x.parse().unwrap());
    let method = args.get(2).map_or("flat", String::as_str);
    let start = Instant::now();
    let (g, queries) = build(case);
    let build_ns = start.elapsed().as_nanos();
    if method == "fixture" {
        println!(
            "{{\"points\":{:?},\"edges\":{:?},\"queries\":{:?}}}",
            g.points(),
            g.edges(),
            queries
        );
        return;
    }
    let mut answers = Vec::new();
    let mut distances = 0;
    let mut expanded = 0;
    let mut frontier = 0;
    for q in &queries {
        let (a, w) = execute(&g, q, method);
        answers.push(
            a.iter()
                .map(|&(x, id)| vec![x, i64::try_from(id).unwrap()])
                .collect::<Vec<_>>(),
        );
        distances += w.distances;
        expanded += w.expanded;
        frontier = frontier.max(w.frontier);
    }
    let reps = (4096 / g.points().len()).max(2);
    let start = Instant::now();
    let mut checksum = 0_i64;
    for _ in 0..reps {
        for q in &queries {
            let (a, w) = execute(black_box(&g), black_box(q), black_box(method));
            checksum = checksum.wrapping_add(
                a.iter()
                    .map(|x| x.0 + i64::try_from(x.1).unwrap())
                    .sum::<i64>(),
            );
            black_box(w);
            black_box(a);
        }
    }
    let ns = start.elapsed().as_nanos();
    println!(
        "{{\"ns\":{ns},\"count\":{},\"checksum\":{checksum},\"build_ns\":{build_ns},\"distances\":{distances},\"expanded\":{expanded},\"frontier\":{frontier},\"answers\":{answers:?}}}",
        reps * queries.len()
    );
}
