//! Public contract regression checks.
use open_addressed_hash_tables::{HashMode, LinearMap, Policy};
#[test]
fn existing_key_after_tombstone_is_replaced_once() {
    let mut map = LinearMap::new(8, Policy::Lazy, HashMode::Identity);
    for k in [1, 9, 17] {
        map.insert(k, k);
    }
    map.remove(9);
    assert_eq!(map.insert(17, 3), Some(17));
    assert_eq!(map.len(), 2);
    assert_eq!(map.iter().filter(|(k, _)| **k == 17).count(), 1);
}
#[test]
fn rebuild_cleans_without_changing_live_map() {
    let mut map = LinearMap::new(32, Policy::Rebuild, HashMode::Identity);
    for k in 0..20 {
        map.insert(k, k);
    }
    for k in 0..8 {
        map.remove(k);
    }
    map.insert(31, 31);
    assert_eq!(map.deleted(), 0);
    assert_eq!(map.work().rebuilds, 1);
    for k in 8..20 {
        assert_eq!(map.get(k), Some(k));
    }
}
