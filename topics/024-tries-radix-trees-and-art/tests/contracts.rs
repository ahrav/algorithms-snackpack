//! Regressions for zero-valued terminals and all-0xff prefix boundaries.
use topic024_tries_radix_trees_and_art::Index;
#[test]
fn zero_is_a_value_and_ff_prefix_needs_no_successor_key() {
    let rows = vec![
        (vec![], 0),
        (vec![255], 0),
        (vec![255, 0], 1),
        (vec![255, 255], 2),
    ];
    for name in ["flat", "tree", "trie", "radix"] {
        let x = Index::new(name, &rows);
        assert_eq!(x.get(&[]), Some(0));
        assert_eq!(x.get(&[255]), Some(0));
        assert_eq!(x.get(&[255, 1]), None);
        assert_eq!(x.prefix(&[255]), vec![0, 1, 2]);
    }
}
