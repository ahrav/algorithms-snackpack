//! Contract tests compare independent structures and stable named IDs.
use topic047_placement::{Map, Policy, hash};
const POLICIES: [Policy; 5] = [
    Policy::Modulo,
    Policy::Ring1,
    Policy::Ring32,
    Policy::Rendezvous,
    Policy::Bounded,
];
// Linear minimum clockwise distance is independent of binary search.
fn reference(p: Policy, ids: &[u64], keys: &[u64]) -> Vec<u64> {
    let mut owners = ids.to_vec();
    owners.sort_unstable();
    let v = if p == Policy::Ring1 { 1 } else { 32 };
    let tokens: Vec<_> = owners
        .iter()
        .flat_map(|&id| (0..v).map(move |t| (hash(id ^ hash(t ^ 0xd1b5_4a32_d192_ed03)), id, t)))
        .collect();
    let mut counts = std::collections::BTreeMap::<u64, usize>::new();
    let cap = (keys.len() * 5).div_ceil(owners.len() * 4);
    keys.iter()
        .map(|&key| {
            let owner = match p {
                Policy::Modulo => owners[usize::try_from(hash(key) % owners.len() as u64).unwrap()],
                Policy::Rendezvous => *owners
                    .iter()
                    .min_by_key(|&&id| {
                        (
                            std::cmp::Reverse(hash(key ^ hash(id ^ 0xa076_1d64_78bd_642f))),
                            id,
                        )
                    })
                    .unwrap(),
                _ => {
                    tokens
                        .iter()
                        .filter(|t| p != Policy::Bounded || *counts.get(&t.1).unwrap_or(&0) < cap)
                        .min_by_key(|t| {
                            (
                                u128::from(t.0)
                                    .wrapping_add(1u128 << 64)
                                    .wrapping_sub(u128::from(hash(key)))
                                    % (1u128 << 64),
                                t.1,
                                t.2,
                            )
                        })
                        .unwrap()
                        .1
                }
            };
            *counts.entry(owner).or_default() += 1;
            owner
        })
        .collect()
}
#[test]
fn boundaries_and_oracle() {
    for p in POLICIES {
        assert_eq!(
            Map::new(p, &[]).unwrap_err().to_string(),
            "empty membership"
        );
        assert!(Map::new(p, &[7, 7]).is_err());
        assert_eq!(Map::new(p, &[7]).unwrap().assign(&[]), [] as [u64; 0]);
        assert_eq!(
            Map::new(p, &[7]).unwrap().assign(&[0, u64::MAX, 0]),
            [7, 7, 7]
        );
        for n in 1..=7 {
            let ids: Vec<_> = (0..n).map(|i| i * 19 + 3).collect();
            for len in [0, 1, 2, 7, 32, 257] {
                let keys: Vec<_> = (0..len).map(hash).collect();
                let got = Map::new(p, &ids).unwrap().assign(&keys);
                assert_eq!(got, reference(p, &ids, &keys));
                let mut reverse = ids.clone();
                reverse.reverse();
                assert_eq!(got, Map::new(p, &reverse).unwrap().assign(&keys));
                if p == Policy::Bounded {
                    let cap = (keys.len() * 5).div_ceil(ids.len() * 4);
                    assert!(
                        ids.iter()
                            .all(|id| got.iter().filter(|&x| x == id).count() <= cap)
                    );
                }
            }
        }
    }
}
#[test]
fn named_owner_monotonicity() {
    let keys: Vec<_> = (0..8192).map(hash).collect();
    for p in [Policy::Ring1, Policy::Ring32, Policy::Rendezvous] {
        let before = Map::new(p, &[7, 19, 100, 999]).unwrap().assign(&keys);
        let added = Map::new(p, &[7, 19, 100, 999, 1001]).unwrap().assign(&keys);
        let removed = Map::new(p, &[7, 100, 999]).unwrap().assign(&keys);
        for i in 0..keys.len() {
            assert!(before[i] == added[i] || added[i] == 1001);
            assert!(before[i] == removed[i] || before[i] == 19);
        }
    }
}
