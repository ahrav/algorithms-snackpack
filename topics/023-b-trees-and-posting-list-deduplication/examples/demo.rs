//! Running example preserving every row identity.
use topic023_b_trees_and_posting_list_deduplication::{Index, Kind};
fn main() {
    let input = [(7, 10), (7, 12), (7, 15), (9, 20), (11, 21), (13, 22)];
    let index = Index::build(Kind::Packed8, &input);
    let result = index.range(7, 12);
    assert_eq!(result, [(7, 10), (7, 12), (7, 15), (9, 20), (11, 21)]);
    println!("{result:?}");
}
