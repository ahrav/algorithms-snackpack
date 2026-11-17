//! Model agreement and representation invariants on bounded adversarial sequences.
use topic_052_red_black::Index;
#[test]
fn prefixes_and_identity() {
    for input in [
        vec![],
        vec![10, 20, 30, 20],
        vec![u64::MAX, 0, u64::MAX, 1],
        (0..257).collect(),
        (0..257).rev().collect(),
        (0..1024).map(|i| i * 37 % 257).collect(),
    ] {
        for kind in ["flat", "boxed", "arena", "btree"] {
            let mut t = Index::new(kind, 0).unwrap();
            let mut oracle = Vec::new();
            let mut births = Vec::new();
            for &k in &input {
                let yes = !oracle.contains(&k);
                assert_eq!(t.insert(k), yes);
                if yes {
                    oracle.push(k);
                    births.push(k);
                    oracle.sort_unstable();
                }
                assert!(t.valid());
                assert_eq!(t.keys(), oracle);
                if kind == "arena" {
                    assert_eq!(t.slot_keys(), births);
                }
                for &q in &[0, 1, 20, 256, 258, u64::MAX] {
                    assert_eq!(t.contains(q), oracle.contains(&q));
                }
            }
            assert!(t.valid());
            assert_eq!(t.keys(), oracle);
        }
    }
}
#[test]
fn all_small_permutations() {
    fn perm(v: &mut [u64], i: usize) {
        if i == v.len() {
            for kind in ["boxed", "arena"] {
                let mut t = Index::new(kind, 0).unwrap();
                for &k in v.iter() {
                    assert!(t.insert(k));
                    assert!(t.valid());
                }
                assert_eq!(t.keys(), vec![0, 1, 2, 3, 4, 5]);
            }
            return;
        }
        for j in i..v.len() {
            v.swap(i, j);
            perm(v, i + 1);
            v.swap(i, j);
        }
    }
    perm(&mut [0, 1, 2, 3, 4, 5], 0);
}
#[test]
fn names() {
    assert!(Index::new("unknown", 0).is_none());
}
