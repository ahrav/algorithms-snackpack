//! Public API rejection and enumeration regression boundaries.
use algorithms_fst::{BuildError, Index, Kind, oracle};
#[test]
fn every_incoming_prefix_emits_its_own_values() {
    let rows = [("bat", 10), ("bats", 12), ("cat", 20), ("cats", 22)]
        .map(|(k, v)| (k.as_bytes().to_vec(), v));
    for kind in [Kind::Flat, Kind::Trie, Kind::Final, Kind::Pushed] {
        let index = Index::build(&rows, kind).unwrap();
        for prefix in [b"".as_slice(), b"b", b"c", b"cat", b"cats", b"catss"] {
            assert_eq!(index.prefix(prefix), oracle(&rows, prefix));
        }
        assert_eq!(index.get(b"bat"), Some(10));
        assert_eq!(index.get(b"cat"), Some(20));
        assert!(matches!(
            Index::build(&[(vec![0; 65], 0)], kind),
            Err(BuildError::KeyTooLong)
        ));
    }
}
