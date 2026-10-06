//! One candidate process for the frozen sorting experiment.
use sorting_lab::{agrees, comparison_count, is_stable, sort, workload};
use std::{hint::black_box, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        println!("Run via measurements/campaign.py for the frozen process schedule.");
        return;
    }
    let method = &args[1];
    let n = args[2].parse().unwrap();
    let shape = &args[3];
    let reps: usize = args[4].parse().unwrap();
    let seed = args[5].parse().unwrap();
    let input = workload(n, shape, seed);
    if shape != "cold" {
        let mut check = input.clone();
        sort(&mut check, method).unwrap();
        assert!(agrees(&input, &check, is_stable(method)));
    }
    let warm = if shape == "cold" { 0 } else { 3 };
    for _ in 0..warm {
        let mut a = input.clone();
        sort(black_box(&mut a), method).unwrap();
        black_box(a);
    }
    let (mut ns, mut setup, mut checksum) = (0_u128, 0_u128, 0_u64);
    for _ in 0..reps {
        let s = Instant::now();
        let mut a = black_box(&input).clone();
        setup += s.elapsed().as_nanos();
        let s = Instant::now();
        sort(black_box(&mut a), method).unwrap();
        ns += s.elapsed().as_nanos();
        if shape == "cold" {
            assert!(agrees(&input, &a, is_stable(method)));
        }
        checksum = checksum.wrapping_add(
            black_box(&a)
                .iter()
                .fold(0_u64, |acc, r| acc.wrapping_add(r.key ^ u64::from(r.id))),
        );
        black_box(a);
    }
    println!(
        "{ns} {setup} {checksum} {} {}",
        comparison_count(&input, false),
        comparison_count(&input, true)
    );
}
