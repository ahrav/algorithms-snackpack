//! Five-vertex distance and stale-entry example.
use shortest_path_queues::{Graph, INF, Queue, shortest_paths};
fn main() {
    let g = Graph::new(
        5,
        &[(0, 1, 10), (0, 2, 1), (2, 1, 1), (1, 3, 1), (2, 3, 20)],
    )
    .unwrap();
    let run = shortest_paths(&g, 0, Queue::Lazy).unwrap();
    assert_eq!(run.distances, vec![0, 2, 1, 3, INF]);
    assert_eq!(run.stats.stale, 2);
    println!(
        "distances: [0, 2, 1, 3, unreachable]; stale pops: {}",
        run.stats.stale
    );
}
