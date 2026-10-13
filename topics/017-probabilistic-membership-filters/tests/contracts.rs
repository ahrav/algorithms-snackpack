//! Public API checks independent of internal filter hashes.
use membership_filters::{Bloom, Cuckoo, QuotientRuns};
#[test]
fn duplicates_empty_and_exact_recheck() {
    let keys = [0, u64::MAX, 42, 42];
    let q = QuotientRuns::build(&keys, 1);
    let mut c = Cuckoo::new(2);
    let mut b = Bloom::new(1, 1, false);
    for k in keys {
        assert!(c.insert(k, 8));
        b.insert(k);
    }
    for k in keys.into_iter().chain(0..10000) {
        let truth = keys.contains(&k);
        for maybe in [q.contains(k), c.contains(k), b.contains(k)] {
            assert!(!truth || maybe);
            assert_eq!(maybe && keys.contains(&k), truth);
        }
    }
    assert!(!QuotientRuns::build(&[], 1).contains(42));
    assert!(!Cuckoo::new(1).contains(42));
    assert!(!Bloom::new(64, 7, true).contains(42));
}
