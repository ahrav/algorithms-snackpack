//! Validated warm dictionary portfolio; independent process samples are run by run.py.
use algorithms_fst::{Index, Kind, Record, oracle};
use std::{hint::black_box, time::Instant};
fn data(case: &str) -> Vec<Record> {
    let n: u64 = if case == "tiny" { 8 } else { 2048 };
    (0..n)
        .map(|i| {
            let group = i / 8;
            let suffix = i % 8;
            let key = if case == "entropy" {
                format!(
                    "{:016x}/{:016x}",
                    i.wrapping_mul(0x9e37_79b9_7f4a_7c15),
                    i.wrapping_mul(0x0123_4567_89ab_cdef)
                )
            } else {
                format!("{group:04x}/shared-tail/{suffix:02x}")
            };
            let value = if case == "random-values" {
                i.wrapping_mul(0x9e37_79b9_7f4a_7c15)
            } else {
                group * 100 + suffix
            };
            (key.into_bytes(), value)
        })
        .collect()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let name = args.get(1).map_or("pushed", String::as_str);
    let case = args.get(2).map_or("tiny", String::as_str);
    let kind = match name {
        "flat" => Kind::Flat,
        "trie" => Kind::Trie,
        "final" => Kind::Final,
        "pushed" => Kind::Pushed,
        _ => panic!("candidate"),
    };
    let rows = data(case);
    let expected = oracle(&rows, b"");
    let index = Index::build(&rows, kind).unwrap();
    assert_eq!(index.prefix(b""), expected);
    for (k, v) in &expected {
        assert_eq!(index.get(k), Some(*v));
        let mut miss = k.clone();
        miss.push(255);
        assert_eq!(index.get(&miss), None);
    }
    let queries: Vec<_> = expected
        .iter()
        .step_by(3)
        .map(|(k, _)| {
            if case == "miss" {
                let mut k = k.clone();
                k[0] = 255;
                k
            } else {
                k.clone()
            }
        })
        .collect();
    let answers: Vec<_> = queries
        .iter()
        .map(|k| expected.iter().find(|(r, _)| r == k).map(|(_, v)| *v))
        .collect();
    for (k, a) in queries.iter().zip(&answers) {
        assert_eq!(index.get(k), *a);
    }
    let prefix = if case == "broad" {
        b"".as_slice()
    } else {
        b"0000/"
    };
    assert_eq!(index.prefix(prefix), oracle(&rows, prefix));
    let reps = if case == "build" {
        4
    } else if case == "broad" {
        40
    } else if case == "prefix" {
        400
    } else {
        250
    };
    let run = || -> u64 {
        if case == "build" {
            let x = Index::build(black_box(&rows), kind).unwrap();
            return black_box(x.get(&rows[0].0)).unwrap();
        }
        if case == "prefix" || case == "broad" {
            return black_box(index.prefix(black_box(prefix)))
                .iter()
                .fold(0, |a, (k, v)| {
                    a.wrapping_add(*v).wrapping_add(k.len() as u64)
                });
        }
        queries.iter().fold(0, |a, k| {
            a.wrapping_add(black_box(index.get(black_box(k))).unwrap_or(0))
        })
    };
    for _ in 0..3 {
        black_box(run());
    }
    let start = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..reps {
        checksum = checksum.wrapping_add(run());
    }
    let elapsed = start.elapsed();
    let units = if case == "build" || case == "prefix" || case == "broad" {
        reps
    } else {
        reps * queries.len()
    };
    let stats = index.stats();
    println!(
        "{{\"candidate\":\"{name}\",\"case\":\"{case}\",\"ns_per_unit\":{},\"checksum\":{checksum},\"states\":{},\"arcs\":{},\"heap_capacity_bytes\":{},\"units\":{units}}}",
        elapsed.as_secs_f64() * 1e9 / f64::from(u32::try_from(units).unwrap()),
        stats.0,
        stats.1,
        stats.2
    );
}
