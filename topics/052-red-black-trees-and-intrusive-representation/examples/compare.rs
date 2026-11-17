//! Build, query, and teardown timing with complete semantic output in check mode.
use std::{hint::black_box, time::Instant};
use topic_052_red_black::{Index, workload};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let kind = args.get(1).map_or("arena", String::as_str);
    let n = args
        .get(2)
        .map_or(Ok(32), |x| x.parse())
        .expect("integer insertion attempts");
    let shape = args.get(3).map_or("ascending", String::as_str);
    let access = args.get(4).map_or("dispersed", String::as_str);
    let check = args.get(5).is_some_and(|x| x == "check");
    let (keys, queries) = workload(n, shape, access);
    let begin = Instant::now();
    let mut index = Index::new(kind, n).expect("flat,boxed,arena,btree");
    let mut added = 0;
    let mut inserts = Vec::new();
    for &key in &keys {
        let yes = index.insert(black_box(key));
        added += usize::from(yes);
        if check {
            inserts.push(yes);
            assert!(index.valid());
        }
    }
    let build_ns = begin.elapsed().as_nanos();
    let begin = Instant::now();
    let mut digest = 0_u64;
    let mut hits = String::new();
    for &q in &queries {
        let yes = black_box(index.contains(black_box(q)));
        digest = digest.wrapping_mul(31).wrapping_add(u64::from(yes));
        if check {
            hits.push(if yes { '1' } else { '0' });
        }
    }
    let query_ns = begin.elapsed().as_nanos();
    if check {
        println!(
            "KEYS {keys:?}\nQUERIES {queries:?}\nINSERTS {inserts:?}\nSORTED {:?}\nSLOTS {:?}\nHITS {hits}",
            index.keys(),
            index.slot_keys()
        );
    }
    let begin = Instant::now();
    drop(index);
    let drop_ns = begin.elapsed().as_nanos();
    println!(
        "{{\"kind\":\"{kind}\",\"n\":{n},\"shape\":\"{shape}\",\"access\":\"{access}\",\"added\":{added},\"digest\":{digest},\"build_ns\":{build_ns},\"query_ns\":{query_ns},\"drop_ns\":{drop_ns}}}"
    );
}
