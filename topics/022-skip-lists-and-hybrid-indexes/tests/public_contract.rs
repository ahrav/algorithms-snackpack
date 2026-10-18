//! Public API boundary and arena-reuse checks.
use topic022_skip_lists_and_hybrid_indexes::{Flat, OrderedIndex, Skip, Tree};
#[test]
fn running_example_and_boundary_contracts() {
    for mut c in [
        Box::<Flat>::default() as Box<dyn OrderedIndex>,
        Box::<Tree>::default(),
        Box::<Skip>::default(),
    ] {
        assert!(!c.remove(7));
        assert_eq!(c.rank(7), None);
        assert_eq!(c.select(0), None);
        for (id, s) in [(1, 10), (2, 20), (3, 20), (4, 40)] {
            c.put(id, s);
        }
        assert_eq!(c.rank(3), Some(2));
        c.put(5, 30);
        assert_eq!(c.select(3), Some((30, 5)));
        c.put(2, 50);
        assert!(c.remove(3));
        assert_eq!(c.snapshot(), vec![(10, 1), (30, 5), (40, 4), (50, 2)]);
        c.put(u64::MAX, i64::MAX);
        c.put(0, i64::MIN);
        assert_eq!(c.range(i64::MIN, 1), vec![(i64::MIN, 0)]);
        assert_eq!(c.select(usize::MAX), None);
        assert!(c.range(0, 0).is_empty());
    }
}
#[test]
#[should_panic(expected = "assertion `left != right` failed")]
fn zero_seed_is_rejected() {
    let _ = Skip::with_seed(0);
}
#[test]
fn retired_slots_are_reused_without_visible_handles() {
    let mut c = Skip::default();
    for i in 0..10000 {
        c.put(i, 0);
        assert!(c.remove(i));
        c.validate();
    }
    assert_eq!(c.retained_slots(), 1);
}
