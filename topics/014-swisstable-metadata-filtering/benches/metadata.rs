//! Controlled lookup and build-plus-lookup experiment; one sample per process.
#![allow(dead_code, clippy::cast_precision_loss, clippy::too_many_lines)]
use std::{collections::BTreeMap, hint::black_box, time::Instant};
use swisstable_metadata_filtering::{Filter, HashShape, Key, Table, hash};

// Independent experiment switches, rather than production protocol states.
#[allow(clippy::struct_excessive_bools)]
struct Workload {
    capacity: usize,
    count: usize,
    hit_percent: u64,
    wide: bool,
    shape: HashShape,
    deleted: bool,
    lifecycle: bool,
    first: bool,
}
fn workload(name: &str) -> Workload {
    let mut w = Workload {
        capacity: 8192,
        count: 6144,
        hit_percent: 50,
        wide: true,
        shape: HashShape::Mixed,
        deleted: false,
        lifecycle: false,
        first: false,
    };
    match name {
        "tiny" => {
            w.capacity = 16;
            w.count = 8;
            w.wide = false;
        }
        "small_hit" => {
            w.count = 4096;
            w.hit_percent = 100;
            w.wide = false;
        }
        "small_miss" => {
            w.count = 4096;
            w.hit_percent = 0;
            w.wide = false;
        }
        "wide_miss" => {
            w.hit_percent = 0;
        }
        "large87" => {
            w.capacity = 131_072;
            w.count = 114_688;
            w.hit_percent = 0;
        }
        "bad_tag" => {
            w.shape = HashShape::ConstantTag;
            w.hit_percent = 0;
        }
        "cluster" => {
            w.capacity = 512;
            w.count = 256;
            w.shape = HashShape::Clustered;
            w.hit_percent = 0;
        }
        "deleted" => {
            w.deleted = true;
            w.hit_percent = 0;
        }
        "build_probe" => {
            w.lifecycle = true;
        }
        "first_batch" => {
            w.first = true;
        }
        _ => panic!("unknown workload"),
    }
    w
}
fn build(w: &Workload) -> Table {
    let mut table = Table::new(w.capacity, w.wide);
    for id in 0..u64::try_from(w.count).unwrap() {
        table
            .insert(Key::new(id), hash(id, w.shape), id ^ 123)
            .unwrap();
    }
    if w.deleted {
        for id in (0..u64::try_from(w.count).unwrap()).step_by(3) {
            table.remove(Key::new(id), hash(id, w.shape));
        }
    }
    table
}
fn batch(table: &Table, queries: &[(Key, u64)], filter: Filter) -> u64 {
    let mut sum = 0_u64;
    for &(key, h) in queries {
        let got = table.get(black_box(key), black_box(h), filter);
        sum = sum.wrapping_add(got.value.unwrap_or(17));
    }
    black_box(sum)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        return;
    }
    let name = &args[1];
    let candidate = &args[2];
    let w = workload(name);
    let filter = match candidate.as_str() {
        "unfiltered" => Filter::Unfiltered,
        "scalar" => Filter::Scalar,
        "word" => Filter::Word,
        _ => panic!("unknown candidate"),
    };
    let table = build(&w);
    let mut oracle = BTreeMap::new();
    for id in 0..u64::try_from(w.count).unwrap() {
        if !w.deleted || id % 3 != 0 {
            oracle.insert(Key::new(id), id ^ 123);
        }
    }
    let qcount = if name == "cluster" { 1024 } else { 8192 };
    let mut queries = Vec::with_capacity(qcount);
    let mut expected = 0_u64;
    for i in 0..u64::try_from(qcount).unwrap() {
        let r = hash(i + 1_000_000, HashShape::Mixed);
        let id = if i % 100 < w.hit_percent {
            r % u64::try_from(w.count).unwrap()
        } else {
            u64::try_from(w.count).unwrap() + r % u64::try_from(w.count).unwrap()
        };
        let key = Key::new(id);
        let h = hash(id, w.shape);
        assert_eq!(table.get(key, h, filter).value, oracle.get(&key).copied());
        queries.push((key, h));
        expected = expected.wrapping_add(oracle.get(&key).copied().unwrap_or(17));
    }
    let mut groups = 0;
    let mut equalities = 0;
    for &(key, h) in &queries {
        let out = table.get(key, h, filter);
        groups += out.groups;
        equalities += out.equalities;
    }
    if !w.first {
        for _ in 0..2 {
            assert_eq!(batch(&table, &queries, filter), expected);
        }
    }
    let repeats = if w.first {
        1
    } else if w.lifecycle {
        4
    } else {
        32
    };
    let start = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..repeats {
        if w.lifecycle {
            let fresh = build(&w);
            checksum = checksum.wrapping_add(batch(&fresh, &queries, filter));
            drop(fresh);
        } else {
            checksum = checksum.wrapping_add(batch(&table, &queries, filter));
        }
    }
    let ns = start.elapsed().as_nanos();
    assert_eq!(checksum, expected.wrapping_mul(repeats));
    println!(
        "{{\"workload\":\"{name}\",\"candidate\":\"{candidate}\",\"ns\":{ns},\"queries\":{},\"repeats\":{repeats},\"groups\":{groups},\"equalities\":{equalities},\"oracle\":true,\"checksum\":{checksum}}}",
        queries.len()
    );
}
