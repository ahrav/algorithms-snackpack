//! Deterministic correctness, operation-count, and process-timing driver.
use monotonic_stacks_deques::{
    next_greater_scan, next_greater_stack, next_greater_stack_counted, window_blocks,
    window_blocks_counted, window_deque, window_deque_counted, window_scan,
};
use std::hint::black_box;
use std::time::Instant;

fn input(shape: &str, n: usize, w: usize) -> Vec<i64> {
    let mut seed = 0x600d_2026_0930_u64;
    (0..n)
        .map(|i| match shape {
            "increasing" => i64::try_from(i).unwrap(),
            "decreasing" => i64::try_from(n - i).unwrap(),
            "equal" => 7,
            "burst" => {
                if i % (w + 1) == w {
                    i64::MAX
                } else {
                    i64::try_from(w - i % (w + 1)).unwrap()
                }
            }
            "random" => {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                i64::try_from(seed % 1024).unwrap() - 512
            }
            _ => panic!("unknown shape"),
        })
        .collect()
}

/// Run the focused example or one process sample.
pub fn main() {
    let start = Instant::now();
    // Cargo passes `--bench` to this driver when it runs as the bench target.
    let args: Vec<String> = std::env::args().filter(|a| a != "--bench").collect();
    if args.len() == 1 {
        let a = [5, 3, 3, 4, 2, 6, 1, 6];
        assert_eq!(window_deque(&a, 3), Some(vec![0, 3, 3, 5, 5, 7]));
        assert_eq!(window_blocks(&a, 3), window_scan(&a, 3));
        assert_eq!(next_greater_stack(&a), next_greater_scan(&a));
        println!(
            "window={:?} next={:?}",
            window_deque(&a, 3),
            next_greater_stack(&a)
        );
        println!(
            "deque={:?} stack={:?}",
            window_deque_counted(&a, 3).unwrap().1,
            next_greater_stack_counted(&a).1
        );
        return;
    }
    assert_eq!(
        args.len(),
        7,
        "--measure algorithm shape n width repetitions"
    );
    assert_eq!(args[1], "--measure");
    let algorithm = args[2].as_str();
    let n: usize = args[4].parse().unwrap();
    let w: usize = args[5].parse().unwrap();
    let reps: u32 = args[6].parse().unwrap();
    assert!(n > 0 && w > 0 && w <= n && reps > 0);
    let a = input(&args[3], n, w);
    let candidate = match algorithm {
        "scan" => window_scan,
        "deque" => window_deque,
        "blocks" => window_blocks,
        _ => panic!("unknown algorithm"),
    };
    let oracle = window_scan(&a, w);
    assert_eq!(candidate(&a, w), oracle);
    for _ in 0..3 {
        black_box(candidate(black_box(&a), w));
    }
    let setup_ns = start.elapsed().as_nanos();
    let timed = Instant::now();
    for _ in 0..reps {
        black_box(candidate(black_box(&a), black_box(w)));
    }
    let elapsed_ns = timed.elapsed().as_nanos();
    let counts = window_deque_counted(&a, w).unwrap().1;
    let block_comparisons = window_blocks_counted(&a, w).unwrap().1;
    println!(
        "{{\"algorithm\":\"{algorithm}\",\"n\":{n},\"w\":{w},\"reps\":{reps},\"setup_ns\":{setup_ns},\"elapsed_ns\":{elapsed_ns},\"scan_comparisons\":{},\"deque_comparisons\":{},\"block_comparisons\":{block_comparisons},\"peak\":{},\"burst\":{}}}",
        (n - w + 1) * (w - 1),
        counts.comparisons,
        counts.peak,
        counts.max_burst
    );
}
