//! Running example: deletion preserves lookup; directory splits cost pointers too.
use open_addressed_hash_tables::{ExtendibleMap, HashMode, LinearMap, Policy};
fn main() {
    for policy in [Policy::Lazy, Policy::Rebuild, Policy::Shift] {
        let mut map = LinearMap::new(8, policy, HashMode::Identity);
        for key in [1, 9, 17] {
            map.insert(key, key * 10);
        }
        map.remove(9);
        assert_eq!(map.get(17), Some(170));
        assert_eq!(map.insert(17, 171), Some(170));
        assert_eq!(map.len(), 2);
        println!(
            "{policy:?}: key17={:?}, probes={}, deleted={}",
            map.get(17),
            map.lookup_probes(17),
            map.deleted()
        );
    }
    let mut map = ExtendibleMap::new(2);
    for key in [1, 9, 17] {
        map.insert(key, key * 10);
    }
    println!(
        "extendible (directory,buckets,pointer visits,rows moved)={:?}",
        map.stats()
    );
    assert_eq!(map.stats(), (16, 5, 45, 8));
}
