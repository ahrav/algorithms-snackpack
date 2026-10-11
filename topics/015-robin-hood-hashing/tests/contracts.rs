//! External value, absence and reachability contracts.
use robin_hood_hashing::{HashMode, Policy, Table};

#[test]
fn empty_absent_and_reinsert() {
    for p in [Policy::Linear, Policy::Robin, Policy::RobinScan] {
        let mut t = Table::new(8, p, HashMode::Constant);
        assert!(t.is_empty());
        assert_eq!(t.get(0), None);
        assert_eq!(t.remove(0), None);
        for k in [u64::MAX, 0, 8, 16] {
            t.insert(k, k).unwrap();
        }
        assert_eq!(t.remove(u64::MAX), Some(u64::MAX));
        assert_eq!(t.insert(u64::MAX, 42), Ok(None));
        assert_eq!(t.get(u64::MAX), Some(42));
        assert!(t.valid());
    }
}

#[test]
fn robin_early_exit_does_not_hide_present_key() {
    let mut t = Table::new(8, Policy::Robin, HashMode::Identity);
    for k in [0, 1, 8, 16, 3] {
        t.insert(k, k).unwrap();
    }
    for k in [0, 1, 8, 16, 3] {
        assert_eq!(t.get(k), Some(k));
    }
    assert_eq!(t.get(24), None);
    assert!(t.valid());
}
