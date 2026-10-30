//! Whole-operation measurement with an independent preflight oracle.
use std::{hint::black_box, time::Instant};
use topic034::{Graph, Sets, Work, bitsets, chk, expand, insert, oracle, parents, workload};
type Answer = (Vec<Option<usize>>, Work);
type Candidate = fn(&Graph, &Sets, &[(usize, usize)]) -> Answer;
fn sets(g: &Graph, _: &Sets, _: &[(usize, usize)]) -> Answer {
    let (d, w) = bitsets(g, 0);
    (parents(&d, 0), w)
}
fn tree(g: &Graph, _: &Sets, _: &[(usize, usize)]) -> Answer {
    chk(g, 0)
}
fn updates(g: &Graph, d: &Sets, edits: &[(usize, usize)], mode: usize) -> Answer {
    let mut g = g.clone();
    let mut d = d.clone();
    let mut total = Work::default();
    for &(u, v) in edits {
        let w = if mode == 0 {
            insert(&mut g, 0, &mut d, u, v)
        } else {
            g[u].push(v);
            if mode == 1 {
                let (next, w) = bitsets(&g, 0);
                d = next;
                w
            } else {
                let (p, w) = chk(&g, 0);
                d = expand(&p);
                w
            }
        };
        total.visits += w.visits;
        total.edges += w.edges;
        total.steps += w.steps;
    }
    (parents(&d, 0), total)
}
fn incremental(g: &Graph, d: &Sets, e: &[(usize, usize)]) -> Answer {
    updates(g, d, e, 0)
}
fn rebuild(g: &Graph, d: &Sets, e: &[(usize, usize)]) -> Answer {
    updates(g, d, e, 1)
}
fn rebuild_tree(g: &Graph, d: &Sets, e: &[(usize, usize)]) -> Answer {
    updates(g, d, e, 2)
}
fn main() {
    let mut args: Vec<_> = std::env::args()
        .skip(1)
        .filter(|a| a != "--bench")
        .collect();
    if args.is_empty() {
        args = vec!["chk".into(), "tiny".into(), "7".into(), "1000".into()];
    }
    assert_eq!(args.len(), 4, "candidate shape n repetitions");
    let n: usize = args[2].parse().unwrap();
    let reps: usize = args[3].parse().unwrap();
    assert!(n >= 2 && reps > 0);
    let shape = &args[1];
    let g = workload(
        if shape == "newreach" {
            "unreachable"
        } else {
            shape
        },
        n,
    );
    let d = bitsets(&g, 0).0;
    let edits: Vec<_> = (0..8)
        .map(|i| match shape.as_str() {
            "noop" => (n - 1, n - 1),
            "bypass" => (0, n / 2 + i.min(n / 2 - 1)),
            "newreach" => (0, n / 2),
            _ => (0, 0),
        })
        .collect();
    let f: Candidate = match args[0].as_str() {
        "sets" => sets,
        "chk" | "control" => tree,
        "incremental" => incremental,
        "rebuild" => rebuild,
        "rebuild-tree" => rebuild_tree,
        _ => panic!("candidate"),
    };
    let dynamic = matches!(args[0].as_str(), "incremental" | "rebuild" | "rebuild-tree");
    let mut expected_graph = g.clone();
    if dynamic {
        for &(u, v) in &edits {
            expected_graph[u].push(v);
        }
    }
    let expected = oracle(&expected_graph, 0);
    let result = f(&g, &d, &edits);
    assert_eq!(expand(&result.0), expected);
    let start = Instant::now();
    for _ in 0..reps {
        black_box(f(black_box(&g), black_box(&d), black_box(&edits)));
    }
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(expand(&f(&g, &d, &edits).0), expected);
    println!(
        "{{\"candidate\":\"{}\",\"shape\":\"{}\",\"n\":{},\"m\":{},\"reps\":{},\"ns\":{},\"visits\":{},\"edges\":{},\"steps\":{},\"oracle\":true}}",
        args[0],
        shape,
        n,
        g.iter().map(Vec::len).sum::<usize>(),
        reps,
        elapsed,
        result.1.visits,
        result.1.edges,
        result.1.steps
    );
}
