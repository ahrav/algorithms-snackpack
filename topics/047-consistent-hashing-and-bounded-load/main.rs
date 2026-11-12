//! Standalone timing entry point.
#[path = "src/lib.rs"]
mod placement;
use placement::{Map, Policy, hash};
use std::{hint::black_box, time::Instant};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let name = &args[1];
    let n: usize = args[2].parse().unwrap();
    let k: u64 = args[3].parse().unwrap();
    let shape = &args[4];
    let phase = &args[5];
    let reps: u32 = args[6].parse().unwrap();
    let p = match name.as_str() {
        "modulo" => Policy::Modulo,
        "ring1" => Policy::Ring1,
        "ring32" => Policy::Ring32,
        "hrw" => Policy::Rendezvous,
        "bounded" => Policy::Bounded,
        _ => panic!("candidate"),
    };
    let ids: Vec<_> = (0..n as u64).map(|i| i * 17 + 7).collect();
    let map = Map::new(p, &ids).unwrap();
    let keys: Vec<_> = if shape == "cluster" {
        let ring = Map::new(Policy::Ring32, &ids).unwrap();
        (0..)
            .filter(|&x| ring.assign(&[x])[0] == 7)
            .take(k as usize)
            .collect()
    } else {
        (0..k).map(hash).collect()
    };
    let owners = map.assign(&keys);
    let start = Instant::now();
    for _ in 0..reps {
        if phase == "build" {
            black_box(Map::new(black_box(p), black_box(&ids)).unwrap());
        } else {
            black_box(map.assign(black_box(&keys)));
        }
    }
    let ns = start.elapsed().as_nanos() as f64 / f64::from(reps);
    let added_ids: Vec<_> = ids.iter().copied().chain([n as u64 * 17 + 7]).collect();
    let removed_ids: Vec<_> = ids.iter().copied().filter(|&id| id != 24).collect();
    let replacement_ids: Vec<_> = removed_ids
        .iter()
        .copied()
        .chain([n as u64 * 17 + 7])
        .collect();
    let added = Map::new(p, &added_ids).unwrap().assign(&keys);
    let removed = Map::new(p, &removed_ids).unwrap().assign(&keys);
    let replaced = Map::new(p, &replacement_ids).unwrap().assign(&keys);
    println!(
        "{{\"candidate\":{name:?},\"n\":{n},\"k\":{k},\"shape\":{shape:?},\"phase\":{phase:?},\"reps\":{reps},\"ns\":{ns},\"bytes\":{},\"keys\":{keys:?},\"owners\":{owners:?},\"added\":{added:?},\"removed\":{removed:?},\"replaced\":{replaced:?}}}",
        map.payload_bytes()
    );
}
