//! One process is one sample; correctness is outside the timed section.
use fenwick_trees::{Fenwick, Index, Prefix, Scan, Segment};
use std::{hint::black_box, time::Instant};
#[derive(Clone, Copy)]
enum Op {
    Add(usize, u64),
    Query(usize, usize),
}
fn workload(name: &str) -> (Vec<u64>, Vec<Op>, bool) {
    let (length, count, reads, short, hot, life) = match name {
        "tiny" => (16, 32768, 50, false, false, false),
        "read_only" => (4096, 8192, 100, false, false, false),
        "mixed" => (4096, 8192, 50, false, false, false),
        "update_heavy" => (4096, 8192, 5, false, false, false),
        "large" => (262_144, 2048, 95, false, false, false),
        "short" => (262_145, 4096, 50, true, false, false),
        "hot" => (262_144, 4096, 50, true, true, false),
        "lifecycle" => (4096, 16, 100, false, false, true),
        _ => panic!("unknown workload"),
    };
    let values = (0..length)
        .map(|i| u64::try_from(i % 17).unwrap())
        .collect();
    let mut seed = 0x0123_4567_89ab_cdef_u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        usize::try_from(seed).unwrap()
    };
    let mut ops = Vec::new();
    for _ in 0..count {
        let position = if hot { next() % 16 } else { next() % length };
        if next() % 100 < reads {
            let end = if short {
                (position + 8).min(length)
            } else {
                position + next() % (length - position + 1)
            };
            ops.push(Op::Query(position, end));
        } else {
            ops.push(Op::Add(
                position,
                if next() % 2 == 0 {
                    1
                } else {
                    1_u64.wrapping_neg()
                },
            ));
        }
    }
    (values, ops, life)
}
fn apply<T: Index>(index: &mut T, ops: &[Op]) -> u64 {
    let mut sum = 0_u64;
    for &op in ops {
        match op {
            Op::Add(position, delta) => index.add(position, delta),
            Op::Query(start, end) => sum = sum.wrapping_add(index.range(start, end)),
        }
    }
    black_box(sum)
}
fn run<T: Index>(name: &str) {
    let (values, ops, life) = workload(name);
    // Compare each observed query, not just an aggregate checksum.
    let mut reference = Scan::build(&values);
    let mut candidate = T::build(&values);
    for &op in &ops {
        match op {
            Op::Add(position, delta) => {
                reference.add(position, delta);
                candidate.add(position, delta);
            }
            Op::Query(start, end) => {
                assert_eq!(candidate.range(start, end), reference.range(start, end));
            }
        }
    }
    let mut warm = T::build(&values);
    black_box(apply(&mut warm, &ops));
    drop(warm);
    let (ns, checksum) = if life {
        let start = Instant::now();
        let mut index = T::build(black_box(&values));
        let checksum = apply(&mut index, black_box(&ops));
        drop(index);
        (start.elapsed().as_nanos(), checksum)
    } else {
        let mut index = T::build(&values);
        let start = Instant::now();
        let checksum = apply(&mut index, black_box(&ops));
        (start.elapsed().as_nanos(), checksum)
    };
    println!("{{\"ns\":{ns},\"checksum\":{checksum},\"queries_checked\":true}}");
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        return;
    }
    match args[1].as_str() {
        "scan" => run::<Scan>(&args[2]),
        "prefix" => run::<Prefix>(&args[2]),
        "fenwick" => run::<Fenwick>(&args[2]),
        "segment" => run::<Segment>(&args[2]),
        _ => panic!("candidate"),
    }
}
