//! Public API checks exercise the same contract for every candidate.
use topic021_heaps_and_priority_queues::{Error, Kind, Queue};
#[test]
fn public_fifo_and_reinsert_contract() {
    for kind in Kind::ALL {
        let mut q = Queue::from_priorities(kind, &[3, 3, 3]);
        q.set(0, 1).unwrap();
        q.set(0, 3).unwrap();
        q.cancel(1).unwrap();
        q.set(1, 3).unwrap();
        let got: Vec<_> = std::iter::from_fn(|| q.pop()).map(|e| e.id).collect();
        assert_eq!(got, [0, 2, 1]);
        assert!(q.is_empty());
    }
}
#[test]
fn empty_universe_rejects_without_side_effects() {
    for kind in Kind::ALL {
        let mut q = Queue::from_priorities(kind, &[]);
        assert_eq!(q.set(0, 1), Err(Error::InvalidId));
        assert_eq!(q.cancel(0), Err(Error::InvalidId));
        assert_eq!(q.pop(), None);
        assert!(q.valid());
    }
}
