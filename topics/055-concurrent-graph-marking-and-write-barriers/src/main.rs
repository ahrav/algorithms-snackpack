//! Print a witness and the bounded exploration report.
#![forbid(unsafe_code)]

use concurrent_graph_marking::{Barrier, Heap, Termination, explore, moved_pointer_is_marked};

fn main() {
    for barrier in [Barrier::None, Barrier::Insertion, Barrier::Deletion] {
        println!(
            "moved pointer {barrier:?}: {}",
            moved_pointer_is_marked(barrier)
        );
    }
    for heap in [Heap::Fork, Heap::Cycle, Heap::Disconnected] {
        for (name, barrier, termination, writes) in [
            ("paused", Barrier::None, Termination::Accounted, 0),
            ("unguarded", Barrier::None, Termination::Accounted, 2),
            ("insertion", Barrier::Insertion, Termination::Accounted, 2),
            ("snapshot", Barrier::Deletion, Termination::Accounted, 2),
            (
                "shared-only",
                Barrier::Insertion,
                Termination::SharedOnly,
                2,
            ),
        ] {
            let row = explore(heap, barrier, termination, writes);
            println!(
                "{heap:?},{name},{},{},{},{},{}",
                row.states,
                row.termination_points,
                row.live_misses,
                row.snapshot_misses,
                row.extra_retention
            );
        }
    }
}
