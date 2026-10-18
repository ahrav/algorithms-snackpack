//! Native process benchmark with independent preflight oracle outside timing.
#![allow(
    dead_code,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[path = "../src/lib.rs"]
mod index;
use index::{Flat, Key, Oracle, OrderedIndex, Skip, Tree};
use std::{hint::black_box, time::Instant};
#[derive(Clone, Copy)]
enum Op {
    Put(u64, i64),
    Remove(u64),
    Get(u64),
    Rank(u64),
    Select(usize),
    Range(i64),
}
fn next(r: &mut u64) -> u64 {
    *r ^= *r << 13;
    *r ^= *r >> 7;
    *r ^= *r << 17;
    *r
}
fn fixture(cell: &str) -> (Vec<Key>, Vec<Op>, bool) {
    let (n, count, mode) = match cell {
        "tiny" => (16, 12000, 0),
        "lookup" => (4096, 12000, 1),
        "rank" => (4096, 12000, 2),
        "update" => (4096, 12000, 3),
        "ties" => (4096, 12000, 4),
        "range" => (4096, 12000, 5),
        "build_ascending" => (4096, 1, 6),
        "build_descending" => (4096, 1, 7),
        _ => panic!("unknown cell"),
    };
    let mut r = 22018;
    let rows = (0..n)
        .map(|i| {
            let score = if mode == 4 {
                (i % 4) as i64
            } else if mode == 6 {
                i as i64
            } else if mode == 7 {
                -(i as i64)
            } else {
                (next(&mut r) % 1_000_000) as i64
            };
            (score, i as u64)
        })
        .collect();
    let mut ops = vec![];
    for i in 0..count {
        let a = next(&mut r);
        let id = a % (n as u64 + n as u64 / 8);
        let score = if mode == 4 {
            (a % 4) as i64
        } else {
            (a % 1_000_000) as i64
        };
        let op = match mode {
            1 => {
                if i % 10 == 0 {
                    Op::Put(id, score)
                } else {
                    Op::Get(id)
                }
            }
            2 => match i % 10 {
                0 => Op::Put(id, score),
                1 => Op::Remove(id),
                2..=5 => Op::Rank(id),
                _ => Op::Select((a as usize) % (n + 1)),
            },
            3 => match i % 10 {
                0 => Op::Get(id),
                1 => Op::Rank(id),
                2 => Op::Remove(id),
                _ => Op::Put(id, score),
            },
            5 => {
                if i % 10 == 0 {
                    Op::Put(id, score)
                } else {
                    Op::Range(score)
                }
            }
            _ => match i % 5 {
                0 => Op::Put(id, score),
                1 => Op::Remove(id),
                2 => Op::Get(id),
                3 => Op::Rank(id),
                _ => Op::Select((a as usize) % (n + 1)),
            },
        };
        ops.push(op);
    }
    (rows, ops, mode >= 6)
}
fn new(kind: &str) -> Box<dyn OrderedIndex> {
    match kind {
        "flat" => Box::<Flat>::default(),
        "tree" => Box::<Tree>::default(),
        "skip" => Box::<Skip>::default(),
        _ => panic!("unknown candidate"),
    }
}
fn setup(kind: &str, rows: &[Key]) -> Box<dyn OrderedIndex> {
    let mut c = new(kind);
    for &(s, id) in rows {
        c.put(id, s);
    }
    c
}
#[allow(clippy::cast_lossless)]
fn apply(c: &mut dyn OrderedIndex, op: Op) -> Vec<Key> {
    match op {
        Op::Put(id, s) => {
            c.put(id, s);
            vec![]
        }
        Op::Remove(id) => vec![(i64::from(c.remove(id)), id)],
        Op::Get(id) => c.get(id).map(|s| (s, id)).into_iter().collect(),
        Op::Rank(id) => c.rank(id).map(|r| (r as i64, id)).into_iter().collect(),
        Op::Select(r) => c.select(r).into_iter().collect(),
        Op::Range(low) => c.range(low, 20),
    }
}
fn fold(h: u64, rows: &[Key]) -> u64 {
    rows.iter().fold(
        h.wrapping_mul(31).wrapping_add(rows.len() as u64),
        |v, &(s, id)| v.rotate_left(7) ^ s as u64 ^ id.wrapping_mul(0x9e37_79b9_7f4a_7c15),
    )
}
fn execute(c: &mut dyn OrderedIndex, ops: &[Op]) -> u64 {
    let mut h = 0;
    for &op in ops {
        h = fold(h, &black_box(apply(c, black_box(op))));
    }
    black_box(h)
}
fn main() {
    // `cargo bench` appends `--bench` after user arguments.
    let args: Vec<_> = std::env::args().filter(|a| a != "--bench").collect();
    if args.len() == 1 {
        println!("Use: ordered CANDIDATE CELL");
        return;
    }
    assert_eq!(args.len(), 3);
    let (kind, cell) = (&args[1], &args[2]);
    let (rows, ops, build) = fixture(cell);
    // Independent oracle, including every response and final state, before warmup.
    let mut oracle = Oracle::default();
    for &(s, id) in &rows {
        oracle.put(id, s);
    }
    let mut c = setup(kind, &rows);
    for &op in &ops {
        assert_eq!(apply(c.as_mut(), op), apply(&mut oracle, op));
    }
    assert_eq!(c.snapshot(), oracle.snapshot());
    drop(c);
    drop(oracle);
    let mut measured = vec![];
    let mut checksum = None;
    for sample in 0..4 {
        let (elapsed, h) = if build {
            let start = Instant::now();
            let c = setup(kind, black_box(&rows));
            let h = fold(0, &black_box(c.range(i64::MIN, 20)));
            drop(c);
            (start.elapsed().as_nanos(), h)
        } else {
            let mut c = setup(kind, &rows);
            let start = Instant::now();
            let h = execute(c.as_mut(), &ops);
            let elapsed = start.elapsed().as_nanos();
            drop(c);
            (elapsed, h)
        };
        assert!(checksum.is_none_or(|v| v == h));
        checksum = Some(h);
        if sample > 0 {
            measured.push(elapsed);
        }
    }
    measured.sort_unstable();
    println!(
        "{{\"ns\":{},\"checksum\":{},\"operations\":{},\"initial_records\":{},\"subsample_min_ns\":{},\"subsample_max_ns\":{}}}",
        measured[1],
        checksum.unwrap(),
        if build { rows.len() } else { ops.len() },
        rows.len(),
        measured[0],
        measured[2]
    );
}
