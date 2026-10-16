//! The before/update/query example.
use segment_trees::{Kind, RangeSum, Tag};
fn main() {
    let mut tree = RangeSum::new(&[2, 1, 3, 4, 0, 5, 2, 1], Kind::Lazy);
    assert_eq!(tree.query(0, 8), 18);
    tree.update(2, 6, Tag::adding(3));
    assert_eq!(tree.query(0, 8), 30);
    assert_eq!(tree.query(3, 5), 10);
    println!("before=18 after=30 query[3,5)=10");
}
