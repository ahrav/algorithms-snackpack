//! Independent-oracle and admission regression tests.
use shortest_path_queues::{Error, Graph, INF, Queue, oracle, shortest_paths};
const ALL: [Queue; 6] = [
    Queue::Scan,
    Queue::Lazy,
    Queue::Indexed,
    Queue::Radix,
    Queue::Dial,
    Queue::ZeroOne,
];
fn agree(g: &Graph, s: usize, domain: u32) {
    let want = oracle(g, s).unwrap();
    for q in ALL {
        if (q == Queue::Dial && domain > 64) || (q == Queue::ZeroOne && domain > 1) {
            continue;
        }
        let got = shortest_paths(g, s, q).unwrap();
        assert_eq!(got.distances, want, "{q:?}");
        if q == Queue::Indexed {
            assert!(got.stats.peak <= g.len());
            assert_eq!(got.stats.stale, 0);
        }
    }
}
#[test]
fn running_example_stale_and_disconnected() {
    let g = Graph::new(
        5,
        &[(0, 1, 10), (0, 2, 1), (2, 1, 1), (1, 3, 1), (2, 3, 20)],
    )
    .unwrap();
    assert_eq!(oracle(&g, 0).unwrap(), vec![0, 2, 1, 3, INF]);
    agree(&g, 0, 20);
    let r = shortest_paths(&g, 0, Queue::Lazy).unwrap();
    assert_eq!(r.stats.stale, 2);
}
#[test]
fn reject_before_search_and_integer_boundary() {
    assert!(matches!(
        Graph::new(1, &[(0, 1, 0)]),
        Err(Error::InvalidVertex)
    ));
    let empty = Graph::new(0, &[]).unwrap();
    assert!(empty.is_empty());
    for q in ALL {
        assert!(matches!(
            shortest_paths(&empty, 0, q),
            Err(Error::InvalidVertex)
        ));
    }
    let g = Graph::new(3, &[(0, 1, u32::MAX), (1, 2, u32::MAX)]).unwrap();
    assert_eq!(
        oracle(&g, 0).unwrap(),
        vec![0, u64::from(u32::MAX), 2 * u64::from(u32::MAX)]
    );
    agree(&g, 0, u32::MAX);
    assert!(matches!(
        shortest_paths(&g, 0, Queue::Dial),
        Err(Error::WeightDomain)
    ));
    assert!(matches!(
        shortest_paths(&g, 0, Queue::ZeroOne),
        Err(Error::WeightDomain)
    ));
}
#[test]
fn exhaustive_three_vertex_binary_graphs_all_roots() {
    for mask in 0..19683_u32 {
        let mut x = mask;
        let mut edges = Vec::new();
        for u in 0..3 {
            for v in 0..3 {
                let a = x % 3;
                x /= 3;
                if a > 0 {
                    edges.push((u, v, a - 1));
                }
            }
        }
        let g = Graph::new(3, &edges).unwrap();
        for s in 0..3 {
            agree(&g, s, 1);
        }
    }
}
#[test]
fn duplicate_decreases_zero_cycles_and_radix_boundaries() {
    for w in [
        0,
        1,
        2,
        3,
        7,
        8,
        63,
        64,
        65,
        255,
        256,
        65535,
        65536,
        u32::MAX,
    ] {
        let mut edges = vec![(0, 0, 0), (0, 1, w), (1, 2, 0), (2, 1, 0), (2, 3, w)];
        for k in (0..=64).rev() {
            edges.push((0, 4, k));
        }
        let g = Graph::new(6, &edges).unwrap();
        agree(&g, 0, w.max(64));
    }
}
#[test]
fn seeded_multigraphs_all_roots() {
    let mut state = 123_456_789_u64;
    for case in 0..160 {
        let n = 2 + case % 15;
        let mut edges = Vec::new();
        for _ in 0..(n * n) {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let u = usize::try_from(state % u64::try_from(n).unwrap()).unwrap();
            let v = usize::try_from((state >> 16) % u64::try_from(n).unwrap()).unwrap();
            let w = u32::try_from((state >> 32) % 65).unwrap();
            edges.push((u, v, w));
        }
        let g = Graph::new(n, &edges).unwrap();
        for s in 0..n {
            agree(&g, s, 64);
        }
    }
}
