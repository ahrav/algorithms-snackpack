//! Fixed connectivity experiment. See BENCHMARK.md for the timing boundary.
#![forbid(unsafe_code)]
#[allow(dead_code)]
#[path = "lib.rs"]
mod uf;
use std::{hint::black_box, time::Instant};
use uf::{Op, Partition};

fn workload(name: &str) -> (usize, Vec<Op>, Vec<Op>, bool) {
    let (n, count) = match name {
        "tiny" => (8, 64),
        "small" => (256, 2048),
        _ => (4096, 8192),
    };
    let mut prefix = Vec::new();
    let mut ops = Vec::new();
    if name == "first" || name == "repeat" {
        let mut step = 1;
        while step < n {
            for a in (0..n).step_by(step * 2) {
                prefix.push(Op {
                    merge: true,
                    a,
                    b: a + step,
                });
            }
            step *= 2;
        }
        for i in 0..count {
            ops.push(Op {
                merge: false,
                a: n - 1 - i % n,
                b: i % n,
            });
        }
    } else {
        let mut seed = 0x1731_8839_u64;
        for i in 0..count {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let mut a = usize::try_from(seed % u64::try_from(n).unwrap()).unwrap();
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let mut b = usize::try_from(seed % u64::try_from(n).unwrap()).unwrap();
            let merge = match name {
                "reads" => i % 20 == 0,
                "unions" => i % 20 != 0,
                "duplicates" => true,
                _ => i % 2 == 0,
            };
            if name == "local" {
                a %= 64;
                b %= 64;
            }
            if name == "duplicates" {
                a = 0;
                b = i % 8;
            }
            ops.push(Op { merge, a, b });
        }
    }
    (n, prefix, ops, name == "repeat")
}
#[inline(never)]
fn run<P: Partition>(
    n: usize,
    prefix: &[Op],
    ops: &[Op],
    warm: bool,
    expected: &[bool],
) -> (u128, u128, u128) {
    let setup = Instant::now();
    let mut state = P::new(n);
    black_box(uf::apply(&mut state, prefix));
    if warm {
        black_box(uf::apply(&mut state, ops));
    }
    let setup_ns = setup.elapsed().as_nanos();
    let start = Instant::now();
    let actual = black_box(uf::apply(&mut state, black_box(ops)));
    let ns = start.elapsed().as_nanos();
    assert_eq!(actual, expected);
    let drop_start = Instant::now();
    drop(state);
    drop(actual);
    (setup_ns, ns, drop_start.elapsed().as_nanos())
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let candidate = args.get(1).expect("candidate");
    let cell = args.get(2).expect("workload");
    assert!(
        [
            "tiny",
            "small",
            "mixed",
            "reads",
            "unions",
            "local",
            "duplicates",
            "first",
            "repeat"
        ]
        .contains(&cell.as_str())
    );
    let (n, prefix, ops, warm) = workload(cell);
    let mut all = prefix.clone();
    all.extend_from_slice(&ops);
    let expected = uf::oracle(n, &all).split_off(prefix.len());
    let (setup, ns, teardown) = match candidate.as_str() {
        "labels" => run::<uf::QuickFind>(n, &prefix, &ops, warm, &expected),
        "size" => run::<uf::Size>(n, &prefix, &ops, warm, &expected),
        "full" => run::<uf::Full>(n, &prefix, &ops, warm, &expected),
        "half" | "half_aa" => run::<uf::Half>(n, &prefix, &ops, warm, &expected),
        _ => panic!("unknown candidate"),
    };
    println!(
        "{{\"candidate\":\"{candidate}\",\"workload\":\"{cell}\",\"n\":{n},\"ops\":{},\"setup_ns\":{setup},\"ns\":{ns},\"teardown_ns\":{teardown},\"oracle\":true}}",
        ops.len()
    );
}
