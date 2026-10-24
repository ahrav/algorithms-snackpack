//! End-to-end append/pop/clear workload; oracle and input setup are untimed.
#![allow(
    clippy::cast_precision_loss,
    reason = "Timing summaries convert bounded counts to descriptive floating-point rates"
)]
use algorithms_growth::{Array, Growth, Shrink, Stats};
use std::{collections::BTreeMap, hint::black_box, time::Instant};

#[derive(Clone, Copy)]
enum Op<const W: usize> {
    Push([u64; W]),
    Pop,
    Clear,
}

fn workload<const W: usize>(case: &str) -> (Vec<Op<W>>, usize, usize) {
    let (n, repeats) = match case {
        "tiny" => (17, 4000),
        "medium" => (8193, 32),
        "large" => (262_145, 2),
        "wide" => (65_537, 2),
        "reuse" | "drain" => (8193, 8),
        "oscillate" => (1025, 8),
        _ => panic!("unknown case"),
    };
    let mut ops = (0..n)
        .map(|i| Op::Push([u64::try_from(i).unwrap(); W]))
        .collect::<Vec<_>>();
    match case {
        "reuse" => {
            for _ in 0..16 {
                ops.push(Op::Clear);
                ops.extend((0..n).map(|i| Op::Push([u64::try_from(i).unwrap(); W])));
            }
        }
        "oscillate" => {
            for _ in 0..4096 {
                ops.push(Op::Pop);
                ops.push(Op::Push([777; W]));
            }
        }
        "drain" => {
            for _ in 0..4 {
                ops.extend((0..n).map(|_| Op::Pop));
                ops.extend((0..n).map(|i| Op::Push([u64::try_from(i).unwrap(); W])));
            }
        }
        _ => {}
    }
    (ops, n, repeats)
}

fn config(name: &str) -> (Growth, Shrink, bool) {
    match name {
        "std" => (Growth::Standard, Shrink::Retain, false),
        "exact" => (Growth::Exact, Shrink::Retain, false),
        "onehalf" => (Growth::OneHalf, Shrink::Retain, false),
        "double" => (Growth::Double, Shrink::Retain, false),
        "reserved" => (Growth::Standard, Shrink::Retain, true),
        "tight" => (Growth::Standard, Shrink::Tight, false),
        "quarter" => (Growth::Standard, Shrink::Quarter, false),
        _ => panic!("unknown candidate"),
    }
}

#[inline(never)]
fn execute<const W: usize>(ops: &[Op<W>], name: &str, hint: usize) -> (Array<W>, u64) {
    let (growth, shrink, reserved) = config(name);
    let mut a = Array::<W>::new(growth, shrink, if reserved { hint } else { 0 }).unwrap();
    let mut popped = 0_u64;
    for op in ops {
        match op {
            Op::Push(value) => a.try_push(*value).unwrap(),
            Op::Pop => {
                if let Some(value) = a.pop() {
                    popped = popped.rotate_left(7) ^ value[0];
                }
            }
            Op::Clear => a.clear(),
        }
    }
    (a, popped)
}

fn validate<const W: usize>(ops: &[Op<W>], name: &str, hint: usize) {
    // Independent ordered-map state, with exact pop comparisons at each step.
    let (growth, shrink, reserved) = config(name);
    let mut a = Array::<W>::new(growth, shrink, if reserved { hint } else { 0 }).unwrap();
    let mut model = BTreeMap::new();
    let mut popped = 0_u64;
    for op in ops {
        match op {
            Op::Push(value) => {
                a.try_push(*value).unwrap();
                model.insert(model.len(), *value);
            }
            Op::Pop => {
                let expected = model.pop_last().map(|(_, v)| v);
                assert_eq!(a.pop(), expected);
                if let Some(value) = expected {
                    popped = popped.rotate_left(7) ^ value[0]; // Same fold as execute.
                }
            }
            Op::Clear => {
                a.clear();
                model.clear();
            }
        }
    }
    assert_eq!(a.as_slice(), model.values().copied().collect::<Vec<_>>());
    let (actual, actual_popped) = execute(ops, name, hint);
    assert_eq!(actual.as_slice(), a.as_slice());
    assert_eq!(actual_popped, popped);
}

fn sample<const W: usize>(name: &str, case: &str) {
    let (ops, hint, repeats) = workload::<W>(case);
    validate(&ops, name, hint);
    drop(execute(black_box(&ops), name, hint)); // One warmup; not a sample.
    let mut checksum = 0_u64;
    let mut stats = Stats::default();
    let mut capacity = 0;
    let start = Instant::now();
    for _ in 0..repeats {
        let (a, popped) = execute(black_box(&ops), black_box(name), hint);
        let sum = a
            .as_slice()
            .iter()
            .flatten()
            .fold(popped, |acc, v| acc.wrapping_add(*v));
        checksum = checksum.wrapping_add(black_box(sum));
        stats = a.stats();
        capacity = a.capacity();
        drop(a); // Allocation, output scan and teardown are inside the boundary.
    }
    let ns = start.elapsed().as_nanos();
    println!(
        "{{\"candidate\":\"{name}\",\"case\":\"{case}\",\"ns_per_op\":{},\"checksum\":{checksum},\"ops\":{},\"repeats\":{repeats},\"width\":{},\"capacity\":{capacity},\"grows\":{},\"shrinks\":{},\"address_changes\":{},\"live_at_growth\":{},\"peak_capacity\":{}}}",
        ns as f64 / (ops.len() * repeats) as f64,
        ops.len(),
        W * 8,
        stats.grows,
        stats.shrinks,
        stats.address_changes,
        stats.live_at_growth,
        stats.peak_capacity
    );
}

fn main() {
    let args = std::env::args()
        .filter(|a| a != "--bench")
        .collect::<Vec<_>>();
    if args.len() == 1 {
        sample::<1>("std", "tiny");
        return;
    }
    assert_eq!(args.len(), 3, "usage: portfolio CANDIDATE CASE");
    if args[2] == "wide" {
        sample::<32>(&args[1], &args[2]);
    } else {
        sample::<1>(&args[1], &args[2]);
    }
}
