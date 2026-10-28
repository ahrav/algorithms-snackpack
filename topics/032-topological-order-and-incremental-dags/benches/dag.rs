//! Fresh-process ordering and cached-update workloads.
use std::hint::black_box;
use std::time::Instant;
use topic_032_topological_order_and_incremental_dags::{
    Evaluator, Graph, Ordering, Update, oracle,
};

fn workload(name: &str) -> (usize, Vec<(usize, usize)>, usize, bool, usize) {
    let (n, fan, chain, component, seed, noop, loops) = match name {
        "tiny" => (8, 2, false, 8, 7, false, 512),
        "chain_root" => (4096, 1, true, 4096, 0, false, 8),
        "chain_leaf" => (4096, 1, true, 4096, 4095, false, 64),
        "islands" => (16384, 1, true, 64, 16320, false, 16),
        "layered_sink" => (4096, 8, false, 4096, 4095, false, 16),
        "dense_root" => (1024, 32, false, 1024, 0, false, 8),
        "noop" => (4096, 8, false, 4096, 0, true, 128),
        _ => panic!("unknown workload"),
    };
    let mut edges = Vec::new();
    for v in 0..n {
        let start = v / component * component;
        if chain {
            if v > start {
                edges.push((n - v, n - 1 - v));
            }
        } else {
            for distance in 1..=fan {
                if v >= start + distance {
                    edges.push((n - 1 - (v - distance), n - 1 - v));
                }
            }
        }
    }
    (n, edges, n - 1 - seed, noop, loops)
}
fn check_order(n: usize, edges: &[(usize, usize)], order: &[usize]) {
    assert_eq!(order.len(), n);
    let mut ranks = vec![usize::MAX; n];
    for (i, &u) in order.iter().enumerate() {
        assert!(u < n);
        assert_eq!(ranks[u], usize::MAX);
        ranks[u] = i;
    }
    for &(u, v) in edges {
        assert!(ranks[u] < ranks[v]);
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        println!("usage: dag order|update fifo|dfs|min|full|dirty|heap|full_aa workload");
        return;
    }
    let (family, candidate, name) = (&args[1], &args[2], &args[3]);
    let (n, edges, seed, noop, loops) = workload(name);
    let construction = Instant::now();
    let g = Graph::new(n, &edges).unwrap();
    let graph_ns = construction.elapsed().as_nanos();
    if family == "order" {
        let mode = match candidate.as_str() {
            "fifo" | "fifo_aa" => Ordering::Fifo,
            "dfs" => Ordering::Dfs,
            "min" => Ordering::Minimum,
            _ => panic!("candidate"),
        };
        let checked = g.topological(mode).unwrap();
        check_order(n, &edges, &checked);
        let started = Instant::now();
        for _ in 0..loops {
            black_box(g.topological(black_box(mode)).unwrap());
        }
        let ns = started.elapsed().as_nanos();
        println!(
            "{{\"ns\":{ns},\"operations\":{loops},\"vertices\":{n},\"edges\":{},\"graph_ns\":{graph_ns},\"checksum\":{},\"oracle\":true}}",
            edges.len(),
            checked.iter().sum::<usize>()
        );
    } else {
        let mode = match candidate.as_str() {
            "full" | "full_aa" => Update::Full,
            "dirty" => Update::Dirty,
            "heap" => Update::Heap,
            _ => panic!("candidate"),
        };
        let initialization = Instant::now();
        let mut e = Evaluator::new(g.clone(), vec![1; n]).unwrap();
        let init_ns = initialization.elapsed().as_nanos();
        let changes: Vec<_> = (0..16)
            .map(|i| vec![(seed, if noop { 1 } else { i + 2 })])
            .collect();
        for edits in &changes {
            e.update(edits, mode).unwrap();
            assert_eq!(e.values(), oracle(n, &edges, e.bases()));
        }
        let expected = e.values().to_vec();
        let mut e = Evaluator::new(g, vec![1; n]).unwrap();
        let (mut evaluated, mut parents, mut scheduled, mut scanned) = (0, 0, 0, 0);
        let started = Instant::now();
        for _ in 0..loops {
            for edits in &changes {
                let work = black_box(e.update(black_box(edits), black_box(mode)).unwrap());
                evaluated += work.evaluated;
                parents += work.parents;
                scheduled += work.scheduled_edges;
                scanned += work.order_entries;
                black_box(e.values());
            }
        }
        let ns = started.elapsed().as_nanos();
        assert_eq!(e.values(), expected);
        let checksum = e.values().iter().fold(0_u64, |a, &b| a.wrapping_add(b));
        let operations = loops * 16;
        println!(
            "{{\"ns\":{ns},\"operations\":{operations},\"vertices\":{n},\"edges\":{},\"graph_ns\":{graph_ns},\"init_ns\":{init_ns},\"evaluated\":{evaluated},\"parents\":{parents},\"scheduled_edges\":{scheduled},\"order_entries\":{scanned},\"checksum\":{checksum},\"oracle\":true}}",
            edges.len()
        );
    }
}
