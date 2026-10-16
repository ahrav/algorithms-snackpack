//! Fixed workload, independently validated before warmup and timing.
use segment_trees::{Kind, RangeSum, Tag};
use std::hint::black_box;
use std::time::Instant;
#[derive(Clone, Copy)]
struct Op {
    l: usize,
    r: usize,
    update: Option<Tag>,
}
fn rng(seed: &mut u64) -> usize {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    usize::try_from(*seed >> 16).expect("64-bit host")
}
fn workload(cell: &str) -> (Vec<u64>, Vec<Op>) {
    let (len, count, width, update_pct) = match cell {
        "tiny" => (32, 512, 32, 50),
        "read_only" => (32_768, 2048, 32_768, 0),
        "mixed" => (32_768, 1024, 32_768, 50),
        "short" | "hot" => (32_768, 2048, 8, 50),
        "update_heavy" => (32_768, 1024, 32_768, 90),
        "adversarial" => (32_769, 1024, 32_769, 67),
        "lifecycle" => (32_768, 16, 32_768, 50),
        _ => panic!("unknown cell"),
    };
    let mut seed = 20_u64;
    let input = (0..len)
        .map(|_| u64::try_from(rng(&mut seed) % 100).unwrap())
        .collect();
    let ops = (0..count)
        .map(|i| {
            let l = rng(&mut seed) % if cell == "hot" { 56 } else { len };
            let r = l + 1 + rng(&mut seed) % width.min(len - l);
            let update = if rng(&mut seed) % 100 < update_pct {
                let value = u64::try_from(rng(&mut seed) % 19).unwrap();
                Some(if i % 3 == 0 {
                    Tag::assigning(value)
                } else {
                    Tag::adding(value)
                })
            } else {
                None
            };
            if cell == "adversarial" && update.is_some() {
                Op {
                    l: 0,
                    r: len,
                    update,
                }
            } else {
                Op { l, r, update }
            }
        })
        .collect();
    (input, ops)
}
fn run(tree: &mut RangeSum, ops: &[Op]) -> u64 {
    let mut checksum = 0_u64;
    for op in ops {
        if let Some(t) = op.update {
            tree.update(op.l, op.r, t);
        } else {
            checksum = checksum.wrapping_add(tree.query(op.l, op.r));
        }
    }
    checksum
}
fn validate(a: &[u64], ops: &[Op], kind: Kind) -> u64 {
    let mut reference = a.to_vec();
    let mut tree = RangeSum::new(a, kind);
    let mut expected = 0_u64;
    for op in ops {
        if let Some(t) = op.update {
            for x in &mut reference[op.l..op.r] {
                *x = t.mul.wrapping_mul(*x).wrapping_add(t.add);
            }
            tree.update(op.l, op.r, t);
        } else {
            let answer = reference[op.l..op.r]
                .iter()
                .copied()
                .fold(0_u64, u64::wrapping_add);
            assert_eq!(tree.query(op.l, op.r), answer);
            expected = expected.wrapping_add(answer);
        }
    }
    assert_eq!(
        tree.query(0, a.len()),
        reference.iter().copied().fold(0_u64, u64::wrapping_add)
    );
    expected
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("portfolio: pass CANDIDATE CELL; see BENCHMARK.md");
        return;
    }
    let name = &args[1];
    let cell = &args[2];
    let kind = match name.as_str() {
        "scan" => Kind::Scan,
        "prefix" => Kind::Prefix,
        "eager" => Kind::Eager,
        "lazy" => Kind::Lazy,
        _ => panic!("candidate"),
    };
    let (a, ops) = workload(cell);
    let expected = validate(&a, &ops, kind); // All validation allocations drop on return.
    let mut warm = RangeSum::new(&a, kind);
    assert_eq!(run(&mut warm, &ops), expected);
    drop(warm);
    let payload = RangeSum::new(&a, kind).payload_bytes();
    let mut ns = 0_u128;
    let mut setup_ns = 0_u128;
    let mut checksum = 0_u64;
    for _ in 0..8 {
        let setup = Instant::now();
        let mut tree = RangeSum::new(black_box(&a), kind);
        setup_ns += setup.elapsed().as_nanos();
        if cell == "lifecycle" {
            drop(tree);
            let start = Instant::now();
            let mut fresh = RangeSum::new(black_box(&a), kind);
            checksum = checksum.wrapping_add(run(&mut fresh, black_box(&ops)));
            drop(fresh);
            ns += start.elapsed().as_nanos();
        } else {
            let start = Instant::now();
            checksum = checksum.wrapping_add(run(&mut tree, black_box(&ops)));
            ns += start.elapsed().as_nanos();
            drop(tree);
        }
    }
    assert_eq!(checksum, expected.wrapping_mul(8));
    black_box(checksum);
    let updates = ops.iter().filter(|o| o.update.is_some()).count();
    let touched: usize = ops.iter().map(|o| o.r - o.l).sum();
    println!(
        "{{\"ns\":{ns},\"setup_ns\":{setup_ns},\"checksum\":{checksum},\"n\":{},\"q\":{},\"updates\":{updates},\"logical_range_elements\":{touched},\"payload_bytes\":{payload},\"replays\":8,\"oracle\":true}}",
        a.len(),
        ops.len()
    );
}
