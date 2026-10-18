//! The running example: member lookup and order statistics agree.
use topic022_skip_lists_and_hybrid_indexes::{OrderedIndex, Skip};
fn main() {
    let mut scores = Skip::default();
    for (id, score) in [(1, 10), (2, 20), (3, 20), (4, 40)] {
        scores.put(id, score);
    }
    assert_eq!(scores.rank(3), Some(2));
    scores.put(5, 30);
    scores.validate();
    assert_eq!(scores.select(3), Some((30, 5)));
    assert_eq!(scores.get(5), Some(30));
    scores.put(2, 50);
    scores.remove(3);
    scores.validate();
    println!("{:?}", scores.snapshot());
    assert_eq!(scores.snapshot(), vec![(10, 1), (30, 5), (40, 4), (50, 2)]);
}
