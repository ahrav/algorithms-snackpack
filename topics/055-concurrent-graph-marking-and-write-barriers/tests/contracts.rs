//! Expected outcomes for the marker model.

use concurrent_graph_marking::{Barrier, Heap, Termination, explore, moved_pointer_is_marked};

#[test]
fn moving_a_pointer_requires_a_barrier() {
    assert!(!moved_pointer_is_marked(Barrier::None));
    assert!(moved_pointer_is_marked(Barrier::Insertion));
    assert!(moved_pointer_is_marked(Barrier::Deletion));
}

#[test]
fn each_barrier_satisfies_its_own_reachability_contract() {
    for heap in [Heap::Fork, Heap::Cycle, Heap::Disconnected] {
        let paused = explore(heap, Barrier::None, Termination::Accounted, 0);
        assert!(paused.termination_points > 0);
        assert_eq!(paused.live_misses, 0);
        assert_eq!(paused.snapshot_misses, 0);
        assert_eq!(paused.extra_retention, 0);

        let insertion = explore(heap, Barrier::Insertion, Termination::Accounted, 2);
        assert!(insertion.termination_points > 0);
        assert_eq!(insertion.live_misses, 0);

        let snapshot = explore(heap, Barrier::Deletion, Termination::Accounted, 2);
        assert!(snapshot.termination_points > 0);
        assert_eq!(snapshot.snapshot_misses, 0);
        assert_eq!(snapshot.live_misses, 0);
        assert!(snapshot.extra_retention > 0);
    }
}

#[test]
fn shared_emptiness_accepts_premature_termination() {
    let bad = explore(Heap::Fork, Barrier::Insertion, Termination::SharedOnly, 2);
    let safe = explore(Heap::Fork, Barrier::Insertion, Termination::Accounted, 2);
    assert_eq!(bad.states, safe.states);
    assert!(bad.live_misses > 0);
    assert_eq!(safe.live_misses, 0);
    assert!(explore(Heap::Fork, Barrier::None, Termination::Accounted, 2).live_misses > 0);
}
