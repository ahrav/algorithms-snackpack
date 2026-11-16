//! Lowest-address placement when free blocks of several orders can serve a request.
use topic051_buddy_allocation::Pool;
#[test]
fn lowest_address_wins_over_a_smaller_free_block() {
    for kind in ["scan", "buddy", "tree"] {
        let mut p = Pool::new(kind, 8, 5);
        assert_eq!(p.alloc(0, 4), Some(0));
        assert_eq!(p.alloc(1, 1), Some(4));
        assert_eq!(p.alloc(2, 1), Some(5));
        assert_eq!(p.alloc(3, 2), Some(6));
        assert!(p.free(0));
        assert!(p.free(2));
        // Free: four units at 0 and one unit at 5. The one-unit request takes address 0.
        assert_eq!(p.alloc(4, 1), Some(0), "{kind}");
        assert_eq!(p.occupied(), vec![1, 0, 0, 0, 1, 0, 1, 1]);
    }
}
