//! Public composition and empty-domain contracts.
use segment_trees::{Kind, OrderedTree, RangeSum, Tag};
#[test]
fn empty_and_temporal_contracts() {
    for k in [Kind::Scan, Kind::Prefix, Kind::Eager, Kind::Lazy] {
        let mut empty = RangeSum::new(&[], k);
        empty.update(0, 0, Tag::assigning(8));
        assert_eq!(empty.query(0, 0), 0);
        let mut t = RangeSum::new(&[1; 9], k);
        t.update(0, 9, Tag::assigning(4));
        t.update(0, 9, Tag::adding(3));
        assert_eq!(t.query(2, 8), 42);
        t.update(3, 7, Tag::assigning(0));
        assert_eq!(t.query(0, 9), 35);
    }
    let empty = OrderedTree::<u64>::new(&[], 0, |a, b| a.wrapping_add(*b));
    assert_eq!(empty.query(0, 0), 0);
}
