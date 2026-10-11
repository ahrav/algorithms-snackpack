//! Running insertion and deletion example.
use robin_hood_hashing::{HashMode, Policy, Table};

fn main() {
    for policy in [Policy::Linear, Policy::Robin] {
        let mut t = Table::new(8, policy, HashMode::Identity);
        for k in [0, 1, 8] {
            t.insert(k, k).unwrap();
        }
        println!(
            "{policy:?}: {:?}; miss 16: {:?}",
            t.layout(),
            t.get_counted(16)
        );
        assert_eq!(t.remove(0), Some(0));
        assert_eq!(t.get(8), Some(8));
        assert!(t.valid());
    }
}
