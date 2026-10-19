//! Public contract regressions.
use topic023_b_trees_and_posting_list_deduplication::{Index, Kind, PostingMap, oracle};
#[test]
fn running_example_and_extremes() {
    let input = [
        (7, 10),
        (7, 12),
        (7, 15),
        (9, 20),
        (11, 21),
        (13, 22),
        (7, 12),
        (7, u64::MAX),
        (u64::MAX, 3),
    ];
    for kind in [
        Kind::Flat,
        Kind::Tree,
        Kind::Posting,
        Kind::Packed8,
        Kind::Packed32,
    ] {
        let index = Index::build(kind, &input);
        assert!(index.valid());
        assert_eq!(index.range(7, 12), oracle(&input, 7, 12));
        assert_eq!(index.range(13, u64::MAX), [(13, 22)]);
        assert!(index.range(7, 7).is_empty());
        assert!(index.range(12, 7).is_empty());
    }
}
#[test]
fn row_deletion_preserves_other_postings() {
    let mut map = PostingMap::default();
    for pair in [(7, 10), (7, 12), (7, 15), (9, 20)] {
        assert!(map.insert(pair));
    }
    assert!(map.remove((7, 12)));
    assert_eq!(map.range(0, 10), [(7, 10), (7, 15), (9, 20)]);
    assert!(map.remove((7, 10)));
    assert!(map.remove((7, 15)));
    assert_eq!(map.range(0, 10), [(9, 20)]);
}
