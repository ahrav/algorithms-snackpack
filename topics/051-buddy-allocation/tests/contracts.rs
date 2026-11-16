//! Placement, lifecycle, fragmentation and restoration contracts.
use topic051_buddy_allocation::Pool;
#[test]
fn rounding_and_lifecycle() {
    for kind in ["scan", "buddy", "tree"] {
        let mut p = Pool::new(kind, 16, 20);
        assert_eq!(p.alloc(0, 5), Some(0));
        assert_eq!(p.alloc(1, 3), Some(8));
        assert_eq!(p.alloc(1, 1), None);
        assert_eq!(p.alloc(2, 0), None);
        assert_eq!(p.alloc(2, usize::MAX), None);
        assert_eq!(p.alloc(99, 1), None);
        assert!(p.free(1));
        assert!(!p.free(1));
        assert!(!p.free(99));
        assert!(p.free(0));
        assert_eq!(p.alloc(2, 16), Some(0));
        assert_eq!(p.alloc(3, 1), None);
        assert!(p.free(2));
        assert_eq!(p.alloc(4, 17), None);
        assert_eq!(p.alloc(4, 1), Some(0));
    }
}
#[test]
fn alternating_holes_and_restoration() {
    for kind in ["scan", "buddy", "tree"] {
        let mut p = Pool::new(kind, 16, 32);
        for id in 0..16 {
            assert_eq!(p.alloc(id, 1), Some(id));
        }
        for id in (0..16).step_by(2) {
            assert!(p.free(id));
        }
        assert_eq!(
            p.occupied(),
            vec![0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1]
        );
        assert_eq!(p.alloc(20, 2), None);
        for id in (1..16).step_by(2) {
            assert!(p.free(id));
        }
        assert_eq!(p.alloc(20, 16), Some(0));
    }
}
#[test]
fn every_small_binary_history() {
    for mask in 0..256 {
        let mut pools: Vec<_> = ["scan", "buddy", "tree"].map(|k| Pool::new(k, 8, 8)).into();
        for step in 0..8 {
            let id = step % 4;
            let r = if mask & (1 << step) == 0 { 1 } else { 3 };
            let a = pools[0].alloc(id, r);
            for p in pools.iter_mut().skip(1) {
                assert_eq!(p.alloc(id, r), a);
            }
            if step % 3 == 2 {
                let f = pools[0].free(id);
                for p in pools.iter_mut().skip(1) {
                    assert_eq!(p.free(id), f);
                }
            }
        }
    }
}
