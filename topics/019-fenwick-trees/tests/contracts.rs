//! Public API witnesses use independent logical arrays.
use fenwick_trees::{Counts, Fenwick, RangeAdd};
#[test]
fn running_example() {
    let mut tree = Fenwick::new(&[3, 1, 4, 0, 2, 5, 0, 1]);
    tree.add(3, 6);
    assert_eq!(tree.prefix(7), 21);
    assert_eq!(tree.range(2, 7), 17);
    let counts = Counts::new(&[3, 1, 4, 6, 2, 5, 0, 1]).unwrap();
    assert_eq!(counts.select(8), Some(3));
    let mut ranges = RangeAdd::new(8);
    ranges.add(3, 4, 6);
    assert_eq!(ranges.prefix(7), 6);
}
