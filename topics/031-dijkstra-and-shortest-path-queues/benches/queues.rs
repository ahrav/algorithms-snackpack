//! Whole-query benchmark with graph and oracle phases recorded separately.
use shortest_path_queues::{Graph, Queue, oracle, shortest_paths};
use std::hint::black_box;
use std::time::Instant;
fn workload(name: &str) -> (Graph, usize) {
    let mut edges = Vec::new();
    let (n, degree, max, reps) = match name {
        "tiny" => (5, 0, 20, 20000),
        "sparse_small" => (128, 6, 1000, 120),
        "sparse_large" => (4096, 6, 1000, 8),
        "bounded" => (4096, 6, 7, 8),
        "zero_one" => (4096, 6, 1, 8),
        "dense" => (512, 511, 1000, 8),
        "decreases" => (256, 0, 1000, 8),
        "wide_chain" => (4096, 0, u32::MAX, 8),
        "disconnected" => (4096, 0, 1, 40),
        _ => panic!("unknown workload"),
    };
    if name == "tiny" {
        edges = vec![(0, 1, 10), (0, 2, 1), (2, 1, 1), (1, 3, 1), (2, 3, 20)];
    } else if name == "decreases" {
        for u in 0..n - 1 {
            edges.push((u, u + 1, 0));
            for v in u + 2..n {
                edges.push((u, v, u32::try_from(2 * n - u).unwrap()));
            }
        }
    } else if name == "wide_chain" {
        for u in 0..n - 1 {
            edges.push((u, u + 1, u32::MAX));
        }
    } else if name == "disconnected" {
        edges = vec![(0, 1, 1), (1, 2, 0), (2, 0, 1)];
    } else {
        let mut state = 0x0123_4567_89ab_cdef_u64;
        for u in 0..n {
            edges.push((u, (u + 1) % n, 1));
            for j in 0..degree {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let v = if name == "dense" {
                    (u + j + 1) % n
                } else {
                    usize::try_from(state % u64::try_from(n).unwrap()).unwrap()
                };
                edges.push((
                    u,
                    v,
                    u32::try_from((state >> 16) % (u64::from(max) + 1)).unwrap(),
                ));
            }
        }
    }
    (Graph::new(n, &edges).unwrap(), reps)
}
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let name = a.get(1).map_or("tiny", String::as_str);
    let candidate = a.get(2).map_or("lazy", String::as_str);
    let queue = match candidate {
        "scan" => Queue::Scan,
        "lazy" | "lazy_aa" => Queue::Lazy,
        "indexed" => Queue::Indexed,
        "radix" => Queue::Radix,
        "dial" => Queue::Dial,
        "zero_one" => Queue::ZeroOne,
        _ => panic!("candidate"),
    };
    let setup = Instant::now();
    let (g, reps) = workload(name);
    let setup_ns = setup.elapsed().as_nanos();
    let cold_start = Instant::now();
    let run = shortest_paths(&g, 0, queue).unwrap();
    let first_query_ns = cold_start.elapsed().as_nanos();
    let check = Instant::now();
    let expected = oracle(&g, 0).unwrap();
    assert_eq!(run.distances, expected);
    let oracle_ns = check.elapsed().as_nanos();
    let checksum = run.distances.iter().fold(0_u64, |x, y| x.wrapping_add(*y));
    let start = Instant::now();
    for _ in 0..reps {
        black_box(shortest_paths(black_box(&g), black_box(0), black_box(queue)).unwrap());
    }
    let elapsed = start.elapsed().as_nanos();
    println!(
        "{{\"workload\":\"{name}\",\"candidate\":\"{candidate}\",\"n\":{},\"edges\":{},\"reps\":{reps},\"elapsed_ns\":{elapsed},\"setup_ns\":{setup_ns},\"oracle_ns\":{oracle_ns},\"first_query_ns\":{first_query_ns},\"pops\":{},\"stale\":{},\"peak\":{},\"improved\":{},\"checksum\":{checksum}}}",
        g.len(),
        g.edge_count(),
        run.stats.pops,
        run.stats.stale,
        run.stats.peak,
        run.stats.improved
    );
}
