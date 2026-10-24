//! Oracle validation is outside timing; fresh processes are the replicate unit.
#![allow(
    clippy::cast_precision_loss,
    reason = "Bounded operation counts become descriptive rates"
)]
use algorithms_ring::{Deque, Fifo, Ring, Shift};
use std::{collections::BTreeMap, hint::black_box, time::Instant};
#[derive(Clone, Copy)]
enum Op {
    Push(u64),
    Pop,
}
struct Work {
    cap: usize,
    preload: usize,
    ops: Vec<Op>,
    repeats: usize,
    lifecycle: bool,
}
fn workload(case: &str) -> Work {
    let (cap, preload, count, repeats, lifecycle) = match case {
        "tiny" => (4, 0, 64, 1000, false),
        "steady_power2" => (1024, 512, 8192, 8, false),
        "steady_odd" => (1000, 500, 8192, 8, false),
        "burst" => (1024, 0, 1024, 32, false),
        "large" => (32768, 32768, 4096, 2, false),
        "full" => (1024, 1024, 8192, 16, false),
        "lifecycle" => (31, 0, 64, 1000, true),
        _ => panic!("unknown case"),
    };
    let mut ops = Vec::new();
    if case == "burst" {
        for i in 0..cap + 8 {
            ops.push(Op::Push(u64::try_from(i).unwrap()));
        }
        for _ in 0..cap + 8 {
            ops.push(Op::Pop);
        }
    } else {
        for i in 0..count {
            let value = u64::try_from(i).unwrap();
            if case == "full" {
                ops.push(Op::Push(value));
            } else if preload > 0 {
                ops.push(Op::Pop);
                ops.push(Op::Push(value));
            } else {
                ops.push(Op::Push(value));
                ops.push(Op::Pop);
            }
        }
    }
    Work {
        cap,
        preload,
        ops,
        repeats,
        lifecycle,
    }
}
fn digest(hash: &mut u64, value: u64) {
    *hash = hash.wrapping_mul(0x0100_0000_01b3).wrapping_add(value);
}
fn oracle(w: &Work) -> (u64, Vec<u64>) {
    let mut map = BTreeMap::new();
    let mut ticket = 0_u64;
    let mut hash = 0_u64;
    for _ in 0..w.repeats {
        if w.lifecycle || ticket == 0 {
            for i in 0..w.preload {
                map.insert(ticket, u64::try_from(i).unwrap());
                ticket += 1;
            }
        }
        for op in &w.ops {
            match *op {
                Op::Push(v) => {
                    if map.len() == w.cap {
                        digest(&mut hash, v ^ 0xffff);
                    } else {
                        map.insert(ticket, v);
                        ticket += 1;
                        digest(&mut hash, 1);
                    }
                }
                Op::Pop => digest(&mut hash, map.pop_first().map_or(0x00ab_cdef, |(_, v)| v)),
            }
        }
        if w.lifecycle {
            map.clear();
        }
    }
    (hash, map.into_values().collect())
}
fn fill<Q: Fifo>(q: &mut Q, w: &Work) {
    for i in 0..w.preload {
        q.push(u64::try_from(i).unwrap()).unwrap();
    }
}
#[inline(never)]
fn execute<Q: Fifo>(w: &Work, expected: &(u64, Vec<u64>)) -> (u128, u64) {
    let mut queue = if w.lifecycle {
        None
    } else {
        let mut q = Q::new(w.cap);
        fill(&mut q, w);
        Some(q)
    };
    let mut hash = 0_u64;
    let start = Instant::now();
    for _ in 0..w.repeats {
        if w.lifecycle {
            let mut q = Q::new(black_box(w.cap));
            fill(&mut q, w);
            queue = Some(q);
        }
        let q = queue.as_mut().unwrap();
        for op in &w.ops {
            match *black_box(op) {
                Op::Push(v) => digest(
                    &mut hash,
                    match q.push(v) {
                        Ok(()) => 1,
                        Err(v) => v ^ 0xffff,
                    },
                ),
                Op::Pop => digest(&mut hash, q.pop().unwrap_or(0x00ab_cdef)),
            }
        }
        if w.lifecycle {
            drop(queue.take());
        }
    }
    black_box(&mut queue);
    black_box(hash);
    let elapsed = start.elapsed().as_nanos();
    let mut final_values = Vec::new();
    if let Some(mut q) = queue {
        while let Some(v) = q.pop() {
            final_values.push(v);
        }
    }
    assert_eq!((hash, final_values), *expected);
    (elapsed, hash)
}
fn main() {
    let args = std::env::args()
        .filter(|a| a != "--bench")
        .collect::<Vec<_>>();
    let name = args.get(1).map_or("deque", String::as_str);
    let case = args.get(2).map_or("tiny", String::as_str);
    let w = workload(case);
    let expected = oracle(&w);
    macro_rules! run {
        ($q:ty) => {{
            let warm = Work {
                repeats: 1,
                ..workload(case)
            };
            execute::<$q>(&warm, &oracle(&warm));
            execute::<$q>(&w, &expected)
        }};
    }
    let (ns, checksum) = match name {
        "shift" => run!(Shift),
        "deque" => run!(Deque),
        "modulo" => run!(Ring<u64,0>),
        "branch" => run!(Ring<u64,1>),
        "mask" => run!(Ring<u64,2>),
        _ => panic!("unknown candidate"),
    };
    let operations = w.ops.len() * w.repeats;
    println!(
        "{{\"candidate\":\"{name}\",\"case\":\"{case}\",\"ns\":{ns},\"operations\":{operations},\"ns_per_op\":{},\"checksum\":{checksum}}}",
        ns as f64 / operations as f64
    );
}
