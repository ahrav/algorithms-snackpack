//! Replays one candidate and optionally emits complete oracle inputs and outputs.
use std::{hint::black_box, time::Instant};
use topic051_buddy_allocation::{Op, Pool, workload};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let kind = &args[1];
    let n: usize = args[2].parse().unwrap();
    let shape = &args[3];
    let check = args.get(4).is_some_and(|x| x == "check");
    let ops = workload(n, shape);
    let start = Instant::now();
    let mut pool = Pool::new(kind, n, n);
    let setup_ns = start.elapsed().as_nanos();
    let mut results = Vec::with_capacity(ops.len());
    let start = Instant::now();
    for op in &ops {
        let result = match *op {
            Op::Alloc(id, request) => pool
                .alloc(id, request)
                .map_or(-1_i64, |a| i64::try_from(a).unwrap()),
            Op::Free(id) => i64::from(pool.free(id)),
        };
        results.push(result);
    }
    let ns = start.elapsed().as_nanos();
    black_box(&pool);
    if check {
        for (op, result) in ops.iter().zip(&results) {
            match op {
                Op::Alloc(id, r) => println!("A {id} {r} {result}"),
                Op::Free(id) => println!("F {id} 0 {result}"),
            }
        }
        println!("STATE {:?}", pool.occupied());
    }
    println!(
        "{{\"kind\":\"{kind}\",\"n\":{n},\"shape\":\"{shape}\",\"ns\":{ns},\"setup_ns\":{setup_ns},\"ops\":{},\"work\":{},\"digest\":{}}}",
        ops.len(),
        pool.work(),
        results.iter().fold(0_u64, |h, x| h
            .wrapping_mul(31)
            .wrapping_add(x.unsigned_abs()))
    );
}
