//! Standalone candidate process. See BENCHMARK.md for timing boundaries.
#[path = "../src/lib.rs"]
#[allow(dead_code, unused_imports)]
mod implementation;
use implementation::{HashMode, LinearMap, Policy};
use std::{
    collections::{BTreeMap, VecDeque},
    hint::black_box,
    time::Instant,
};
#[derive(Clone, Copy)]
enum Op {
    Get(u64),
    Put(u64, u64),
    Del(u64),
}
fn apply(map: &mut LinearMap, op: Op) -> Option<u64> {
    match op {
        Op::Get(k) => map.get(k),
        Op::Put(k, v) => map.insert(k, v),
        Op::Del(k) => map.remove(k),
    }
}
fn reference(init: &[u64], ops: &[Op]) -> (Vec<Option<u64>>, BTreeMap<u64, u64>) {
    let mut map: BTreeMap<_, _> = init.iter().map(|&k| (k, k.wrapping_add(1))).collect();
    let result = ops
        .iter()
        .map(|&op| match op {
            Op::Get(k) => map.get(&k).copied(),
            Op::Put(k, v) => map.insert(k, v),
            Op::Del(k) => map.remove(&k),
        })
        .collect();
    (result, map)
}
fn checksum(values: impl Iterator<Item = Option<u64>>) -> u64 {
    values.fold(0_u64, |acc, v| {
        acc.wrapping_add(v.map_or(0x7ed5_5d16, |x| x.wrapping_add(1)))
    })
}
fn prepare(name: &str) -> (usize, HashMode, Vec<u64>, Vec<Op>, bool, usize) {
    let (cap, n, mode, churn, mixed, build, cold, reps) = match name {
        "tiny" => (32, 8, HashMode::Mixed, false, false, false, false, 128),
        "clean25" => (16384, 4096, HashMode::Mixed, false, false, false, false, 4),
        "clean75" => (16384, 12288, HashMode::Mixed, false, false, false, false, 4),
        "clean87" => (16384, 14336, HashMode::Mixed, false, false, false, false, 4),
        "churn_read" => (16384, 12288, HashMode::Mixed, true, false, false, false, 4),
        "mixed25" => (16384, 4096, HashMode::Mixed, false, true, false, false, 1),
        "mixed75" => (16384, 12288, HashMode::Mixed, false, true, false, false, 1),
        "cluster" => (512, 256, HashMode::Identity, false, true, false, false, 1),
        "build" => (8, 8192, HashMode::Mixed, false, false, true, false, 1),
        "first_lookup" => (16384, 12288, HashMode::Mixed, false, false, false, true, 1),
        _ => panic!("unknown workload"),
    };
    let n: u64 = n;
    let stride = if matches!(mode, HashMode::Identity) {
        cap as u64
    } else {
        1
    };
    let init: Vec<u64> = if build {
        vec![]
    } else {
        (0..n).map(|i| i * stride).collect()
    };
    let mut ops = Vec::new();
    if build {
        for k in 0..n {
            ops.push(Op::Put(k, k + 1));
        }
    } else if mixed {
        let mut active: VecDeque<_> = init.iter().copied().collect();
        let cycles = if name == "cluster" { 4096 } else { 16384 };
        for i in 0..cycles {
            let old = active.pop_front().unwrap();
            let new = (n + i) * stride;
            ops.extend([
                Op::Del(old),
                Op::Put(new, new + 1),
                Op::Get(new),
                Op::Get((1_u64 << 40) + new),
            ]);
            active.push_back(new);
        }
    } else {
        let base = if churn { 2 * n } else { 0 };
        let count = if name == "tiny" { 512 } else { 65536 };
        for i in 0..count {
            let key = base + i % n;
            ops.push(Op::Get(if i % 2 == 0 { key } else { (1_u64 << 40) + key }));
        }
    }
    (cap, mode, init, ops, cold, reps)
}
fn populated(cap: usize, mode: HashMode, policy: Policy, init: &[u64], churn: bool) -> LinearMap {
    let mut map = LinearMap::new(cap, policy, mode);
    for &k in init {
        map.insert(k, k.wrapping_add(1));
    }
    if churn {
        let n = init.len() as u64;
        for base in [0, n] {
            for i in 0..n {
                map.remove(base + i);
                map.insert(base + n + i, base + n + i + 1);
            }
        }
    }
    map
}
fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    if args.len() == 1 {
        args.extend([String::from("tiny"), String::from("shift")]);
    }
    let name = &args[1];
    let policy = match args[2].as_str() {
        "lazy" => Policy::Lazy,
        "rebuild" => Policy::Rebuild,
        "shift" => Policy::Shift,
        _ => panic!("candidate"),
    };
    let setup = Instant::now();
    let (cap, mode, init, ops, cold, reps) = prepare(name);
    let churn = name == "churn_read";
    let actual_init: Vec<_> = if churn {
        (2 * init.len() as u64..3 * init.len() as u64).collect()
    } else {
        init.clone()
    };
    let (expected, expected_final) = reference(&actual_init, &ops);
    let expected_sum = checksum(expected.iter().copied());
    let mut checked = populated(cap, mode, policy, &init, churn);
    let actual: Vec<_> = ops.iter().map(|&op| apply(&mut checked, op)).collect();
    assert_eq!(actual, expected);
    let final_rows: BTreeMap<_, _> = checked.iter().map(|(k, v)| (*k, *v)).collect();
    assert_eq!(final_rows, expected_final);
    // A separate prepared map is never oracle-executed before the measured calls.
    let setup_ns = setup.elapsed().as_nanos();
    if !cold {
        for _ in 0..2 {
            let mut map = populated(cap, mode, policy, &init, churn);
            black_box(checksum(ops.iter().map(|&op| apply(&mut map, op))));
        }
    }
    let mut total = 0_u128;
    let mut output = 0_u64;
    let mut work = (0, 0, 0);
    let mut probes = 0_usize;
    let mut start_deleted = 0;
    for _ in 0..reps {
        let mut map = if name == "build" {
            None
        } else {
            Some(populated(cap, mode, policy, &init, churn))
        };
        if let Some(m) = &map {
            start_deleted = m.deleted();
        }
        let start = Instant::now();
        if map.is_none() {
            map = Some(LinearMap::new(cap, policy, mode));
        }
        let m = map.as_mut().unwrap();
        let sum = checksum(ops.iter().map(|&op| apply(m, black_box(op))));
        black_box(sum);
        output = output.wrapping_add(sum);
        if name == "build" {
            drop(map.take());
        }
        total += start.elapsed().as_nanos();
        assert_eq!(sum, expected_sum);
        if let Some(m) = map {
            probes = ops
                .iter()
                .filter_map(|op| {
                    if let Op::Get(k) = op {
                        Some(m.lookup_probes(*k))
                    } else {
                        None
                    }
                })
                .sum();
            let w = m.work();
            work = (w.rebuilds, w.reinserted, w.shifted);
        }
    }
    println!(
        "{{\"workload\":\"{name}\",\"candidate\":\"{}\",\"ns\":{},\"reps\":{reps},\"ops\":{},\"setup_oracle_ns\":{setup_ns},\"checksum\":{output},\"lookup_probe_sum\":{probes},\"start_deleted\":{start_deleted},\"rebuilds\":{},\"reinserted\":{},\"shifted\":{},\"oracle\":true}}",
        args[2],
        total / (reps as u128),
        ops.len(),
        work.0,
        work.1,
        work.2
    );
}
