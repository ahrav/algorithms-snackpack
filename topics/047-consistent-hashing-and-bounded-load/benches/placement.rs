//! Cargo smoke benchmark; the balanced process campaign uses main.rs and run.py.
use std::{hint::black_box, time::Instant};
use topic047_placement::{Map, Policy, hash};
fn main() {
    let keys: Vec<_> = (0..4096).map(hash).collect();
    for policy in [
        Policy::Modulo,
        Policy::Ring1,
        Policy::Ring32,
        Policy::Rendezvous,
        Policy::Bounded,
    ] {
        let ids: Vec<_> = (0..32).map(|i| i * 17 + 7).collect();
        let map = Map::new(policy, &ids).unwrap();
        let start = Instant::now();
        for _ in 0..8 {
            black_box(map.assign(black_box(&keys)));
        }
        println!("{policy:?} smoke {:?}", start.elapsed());
    }
}
