use std::{hint::black_box, time::Instant};
use topic033::{Graph, certify, gabow, kosaraju, kosaraju_prepared, tarjan, transpose};

fn workload(name: &str) -> (Graph, usize, bool) {
    if name == "tiny-first" {
        return (topic033::example(), 1, false);
    }
    if name == "tiny" {
        return (topic033::example(), 10_000, true);
    }
    let n = if name == "dense" { 256 } else { 20_000 };
    let mut g = vec![vec![]; n];
    let mut seed = 20261029_u64;
    for (v, edges) in g.iter_mut().enumerate() {
        match name {
            "isolated" => (),
            "chain" => {
                if v + 1 < n {
                    edges.push(v + 1);
                }
            }
            "cycle" => edges.push((v + 1) % n),
            "groups" | "permuted" | "duplicates" => {
                edges.push(v / 8 * 8 + (v + 1) % 8);
                if v % 8 == 7 && v + 1 < n {
                    edges.push(v + 1);
                }
                if name == "duplicates" {
                    edges.extend(vec![v / 8 * 8 + (v + 1) % 8; 7]);
                }
            }
            "dense" => edges.extend((0..n).filter(|&w| w != v)),
            "random" => {
                for _ in 0..4 {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    edges.push(usize::try_from(seed % u64::try_from(n).unwrap()).unwrap());
                }
            }
            _ => panic!("unknown workload"),
        }
    }
    if name == "permuted" {
        let mut p = vec![vec![]; n];
        // 7919 and 20000 are coprime, giving a fixed vertex permutation.
        for v in 0..n {
            p[v * 7919 % n] = g[v].iter().map(|&w| w * 7919 % n).collect();
        }
        g = p;
    }
    (g, if name == "dense" { 8 } else { 16 }, true)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let name = args
        .get(1)
        .filter(|s| !s.starts_with('-'))
        .map_or("tiny", String::as_str);
    let candidate = args.get(2).map_or("tarjan", String::as_str);
    let (g, reps, warm) = workload(name);
    let reverse = if candidate == "prepared" {
        transpose(&g)
    } else {
        vec![]
    };
    let compute = || match candidate {
        "tarjan" | "tarjan-aa" => tarjan(black_box(&g)),
        "gabow" => gabow(black_box(&g)),
        "kosaraju" => kosaraju(black_box(&g)),
        "prepared" => kosaraju_prepared(black_box(&g), black_box(&reverse)),
        _ => panic!("unknown candidate"),
    };
    if warm {
        black_box(compute());
    }
    let start = Instant::now();
    for _ in 0..reps {
        black_box(compute());
    }
    let ns = start.elapsed().as_nanos();
    let id = compute();
    assert!(certify(&g, &id));
    let edges: usize = g.iter().map(Vec::len).sum();
    let components = id.iter().max().map_or(0, |&v| v + 1);
    println!(
        "{{\"workload\":\"{name}\",\"candidate\":\"{candidate}\",\"vertices\":{},\"edges\":{edges},\"components\":{components},\"reps\":{reps},\"ns_total\":{ns},\"ns_per_op\":{},\"certified\":true}}",
        g.len(),
        ns as f64 / reps as f64
    );
}
