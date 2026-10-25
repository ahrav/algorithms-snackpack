//! External rollback and class-membership contracts.
use topic029_union_find::{Full, Half, Partition, Rollback, Size};

#[test]
fn rollback_every_three_merge_history() {
    for mut code in 0..16_usize.pow(3) {
        let mut p = Rollback::new(4);
        p.union(0, 1);
        let snapshot = p.snapshot();
        for _ in 0..3 {
            let op = code % 16;
            code /= 16;
            p.union(op / 4, op % 4);
        }
        p.rollback(snapshot);
        for a in 0..4 {
            for b in 0..4 {
                assert_eq!(p.same(a, b), a == b || (a < 2 && b < 2));
            }
        }
    }
}

#[test]
fn class_sizes_and_groups_preserve_membership() {
    let mut full = Full::new(6);
    let mut half = Half::new(6);
    let mut size = Size::new(6);
    for (a, b) in [(5, 3), (2, 1), (3, 1), (1, 1)] {
        full.union(a, b);
        half.union(a, b);
        size.union(a, b);
    }
    let mut expected = vec![vec![0], vec![1, 2, 3, 5], vec![4]];
    expected.sort();
    for mut groups in [full.groups(), half.groups(), size.groups()] {
        groups.sort();
        assert_eq!(groups, expected);
    }
    for x in [1, 2, 3, 5] {
        assert_eq!(full.class_size(x), 4);
        assert_eq!(half.class_size(x), 4);
        assert_eq!(size.class_size(x), 4);
    }
}
