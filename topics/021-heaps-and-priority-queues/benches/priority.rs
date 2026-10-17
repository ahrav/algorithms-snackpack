//! Standalone runner: rustc --edition=2024 -O benches/priority.rs -o heap-bench.
#![allow(dead_code)]
#[path = "../src/lib.rs"]
mod queues;
use queues::{Entry, Kind, Op, Oracle, Queue, apply};
use std::hint::black_box;
use std::time::Instant;

fn next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state ^ (*state >> 29)
}
fn workload(name: &str) -> (Vec<u64>, Vec<Op>, bool) {
    let (n, count, set_pct, equal): (usize, usize, u64, bool) = match name {
        "tiny" => (16, 4096, 70, false),
        "small" => (256, 8192, 70, false),
        "mixed" => (4096, 16384, 70, false),
        "updates" => (16384, 24576, 99, false),
        "ties" => (4096, 16384, 70, true),
        "stale" => (1024, 24576, 100, false),
        "build_random" | "build_descending" => (65536, 0, 100, false),
        _ => panic!("unknown workload"),
    };
    let mut state = 21017;
    let initial = (0..n)
        .map(|i| {
            if name == "build_descending" {
                (n - i) as u64
            } else if equal {
                7
            } else {
                next(&mut state) % 1_000_000 + 100_000
            }
        })
        .collect();
    let mut ops = Vec::with_capacity(count);
    for i in 0..count {
        if name == "stale" {
            ops.push(Op::Set(0, i as u64));
            continue;
        }
        let id = usize::try_from(next(&mut state) % u64::try_from(n).unwrap()).unwrap();
        let choose = next(&mut state) % 100;
        if choose < set_pct {
            ops.push(Op::Set(
                id,
                if equal {
                    7
                } else {
                    next(&mut state) % 1_000_000
                },
            ));
        } else if choose < set_pct + (100 - set_pct) * 2 / 3 {
            ops.push(Op::Pop);
        } else {
            ops.push(Op::Cancel(id));
        }
    }
    if name == "stale" {
        ops.push(Op::Pop);
    }
    (initial, ops, name.starts_with("build_"))
}
fn validate(kind: Kind, initial: &[u64], ops: &[Op]) {
    let mut q = Queue::from_priorities(kind, initial);
    let mut model = Oracle::new(initial);
    assert!(q.valid());
    for &op in ops {
        assert_eq!(apply(&mut q, op), model.apply(op));
    }
    assert_eq!(q.snapshot(), model.snapshot());
    assert!(q.valid());
    assert_eq!(q.peek(), model.snapshot().first().copied());
}
fn consume(sum: u64, e: Option<Entry>) -> u64 {
    e.map_or(sum.rotate_left(1), |e| {
        sum.wrapping_add(e.priority).rotate_left(7) ^ e.arrival ^ (e.id as u64) ^ e.stamp
    })
}
fn run(
    kind: Kind,
    initial: &[u64],
    ops: &[Op],
    lifecycle: bool,
) -> (u128, u128, u128, u64, (usize, usize, usize)) {
    let setup = Instant::now();
    let mut q = if lifecycle {
        None
    } else {
        Some(Queue::from_priorities(kind, initial))
    };
    let setup_ns = setup.elapsed().as_nanos();
    let start = Instant::now();
    if lifecycle {
        q = Some(Queue::from_priorities(kind, black_box(initial)));
    }
    let queue = q.as_mut().unwrap();
    let mut sum = 0;
    for &op in black_box(ops) {
        sum = consume(sum, apply(queue, op));
    }
    sum = consume(sum, queue.peek());
    let counters = queue.counters();
    black_box(sum);
    if lifecycle {
        drop(q.take());
    }
    let elapsed = start.elapsed().as_nanos();
    let teardown = Instant::now();
    drop(q);
    (
        elapsed,
        setup_ns,
        teardown.elapsed().as_nanos(),
        sum,
        counters,
    )
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let kind_name = args.get(1).map_or("four", String::as_str);
    let cell = args.get(2).map_or("tiny", String::as_str);
    let kind = Kind::ALL
        .into_iter()
        .find(|k| k.name() == kind_name)
        .expect("candidate");
    let (initial, ops, lifecycle) = workload(cell);
    validate(kind, &initial, &ops);
    let _ = run(kind, &initial, &ops, lifecycle); // one fresh-state warmup, discarded
    let mut total = 0;
    let mut setup = 0;
    let mut teardown = 0;
    let mut checksum = 0;
    let mut counters = (0, 0, 0);
    for _ in 0..3 {
        let r = run(kind, &initial, &ops, lifecycle);
        total += r.0;
        setup += r.1;
        teardown += r.2;
        checksum ^= r.3;
        counters = r.4;
    }
    println!(
        "{{\"candidate\":\"{kind_name}\",\"cell\":\"{cell}\",\"ns\":{},\"setup_ns\":{},\"teardown_ns\":{},\"checksum\":{checksum},\"peak_records\":{},\"rebuilds\":{},\"stale_pops\":{},\"initial\":{},\"ops\":{},\"repeats\":3,\"oracle\":true}}",
        total / 3,
        setup / 3,
        teardown / 3,
        counters.0,
        counters.1,
        counters.2,
        initial.len(),
        ops.len()
    );
}
