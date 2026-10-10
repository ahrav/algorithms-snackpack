//! Transient row loss between operations is invisible to an end-of-run snapshot;
//! this schedule is the library seeded test with a full-row check after each step.
use open_addressed_hash_tables::{HashMode, LinearMap, Policy};
use std::collections::BTreeMap;

#[test]
fn seeded_rows_match_oracle_after_every_transition() {
    for policy in [Policy::Lazy, Policy::Rebuild, Policy::Shift] {
        let mut map = LinearMap::new(8, policy, HashMode::Mixed);
        let mut oracle = BTreeMap::new();
        let mut state = 17_u64;
        for step in 0..20_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let k = (state >> 32) % 257;
            let (actual, expected) = match step % 3 {
                0 => (map.insert(k, state), oracle.insert(k, state)),
                1 => (map.remove(k), oracle.remove(&k)),
                _ => (map.get(k), oracle.get(&k).copied()),
            };
            assert_eq!(actual, expected, "{policy:?} step {step} key {k}");
            assert_eq!(map.len(), oracle.len(), "{policy:?} step {step} key {k}");
            let rows: BTreeMap<_, _> = map.iter().map(|(k, v)| (*k, *v)).collect();
            assert_eq!(rows, oracle, "{policy:?} step {step} key {k}");
        }
        assert_eq!(map.capacity(), 256, "{policy:?} final capacity");
    }
}
