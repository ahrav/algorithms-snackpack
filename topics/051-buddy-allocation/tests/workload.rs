//! Supported domain of the synthetic trace generator.
use topic051_buddy_allocation::{Op, workload};
const SHAPES: [&str; 4] = ["tiny", "mixed", "holes", "churn"];
fn requests(n: usize, shape: &str) -> Vec<usize> {
    let requested = |op: &Op| match *op {
        Op::Alloc(_, request) => Some(request),
        Op::Free(_) => None,
    };
    workload(n, shape).iter().filter_map(requested).collect()
}
#[test]
fn requests_are_positive_from_four_units() {
    for n in (2..=12).map(|k| 1_usize << k) {
        for shape in SHAPES {
            assert!(requests(n, shape).iter().all(|&r| r >= 1), "{n} {shape}");
        }
    }
}
#[test]
fn churn_requests_cycle_within_the_arena() {
    for (n, cycle) in [
        (4, [1, 3, 1, 2]),
        (64, [31, 3, 1, 17]),
        (4096, [2047, 3, 1, 1025]),
    ] {
        let expected: Vec<usize> = cycle.iter().copied().cycle().take(64).collect();
        assert_eq!(requests(n, "churn"), expected);
    }
    for n in (2..=12).map(|k| 1_usize << k) {
        assert!(requests(n, "churn").iter().all(|&r| r <= n), "{n}");
    }
}
#[test]
fn measured_sizes_request_at_most_the_arena() {
    for n in [64, 4096] {
        for shape in SHAPES {
            assert!(requests(n, shape).iter().all(|&r| r <= n), "{n} {shape}");
        }
    }
}
