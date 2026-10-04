//! Parameterized complete-operation driver for the frozen process runner.
use std::{collections::BTreeMap, hint::black_box, time::Instant};
use top_k_portfolio::{CANDIDATES, select_three_way};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || (args.len() == 2 && args[1] == "--bench") {
        for (_, candidate) in CANDIDATES {
            assert_eq!(candidate(&[9, 1, 7, 3, 3, 8, 2, 6], 3), Some(vec![1, 2, 3]));
        }
        println!("Top-K smoke passed; use scripts/run.py for the frozen campaign.");
        return;
    }
    assert_eq!(args.len(), 7, "candidate n k shape boundary calls");
    let name = &args[1];
    let n: usize = args[2].parse().unwrap();
    let k: usize = args[3].parse().unwrap();
    let shape = &args[4];
    let boundary = &args[5];
    let calls: u32 = args[6].parse().unwrap();
    assert!(calls > 0 && k <= n);
    let f = CANDIDATES.iter().find(|(x, _)| *x == name).unwrap().1;
    let mut state = 0xa11c_0008_6a30_u64;
    let input: Vec<i64> = (0..n)
        .map(|i| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            match shape.as_str() {
                "random" => i64::from_ne_bytes(state.to_ne_bytes()),
                "ascending" => i64::try_from(i).unwrap(),
                "descending" => i64::try_from(n - i).unwrap(),
                "duplicates" => i64::try_from(state % 8).unwrap(),
                "equal" => 5,
                "organ" => i64::try_from(i.min(n - 1 - i)).unwrap(),
                _ => panic!("unknown shape"),
            }
        })
        .collect();
    // A first-use measurement excludes this untimed preparation and uses a fresh
    // process without calling the candidate beforehand. It is not cache-flushed.
    let mut counts = BTreeMap::new();
    for &x in &input {
        *counts.entry(x).or_insert(0_usize) += 1;
    }
    let expected: Vec<i64> = counts
        .into_iter()
        .flat_map(|(v, c)| std::iter::repeat_n(v, c))
        .take(k)
        .collect();
    if boundary == "warm" {
        for _ in 0..4 {
            assert_eq!(f(black_box(&input), k).unwrap(), expected);
        }
    } else {
        assert_eq!(boundary, "first");
        assert_eq!(calls, 1);
    }
    let start = Instant::now();
    let mut checksum = 0_i64;
    for _ in 0..calls {
        let result = f(black_box(&input), black_box(k)).unwrap();
        checksum = checksum.wrapping_add(result[0]);
        checksum = checksum.wrapping_add(*result.last().unwrap());
        black_box(&result);
        // Result drop is inside this loop and therefore inside the boundary.
    }
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(f(&input, k).unwrap(), expected);
    let mut copy = input.clone();
    let counters = select_three_way(&mut copy, k - 1).unwrap();
    assert!(elapsed > 0);
    println!(
        "{{\"candidate\":\"{name}\",\"n\":{n},\"k\":{k},\"shape\":\"{shape}\",\"boundary\":\"{boundary}\",\"calls\":{calls},\"elapsed_ns\":{elapsed},\"ns_per_call\":{},\"checksum\":{checksum},\"classified\":{},\"partitions\":{},\"swaps\":{},\"fallback\":{}}}",
        f64::from(u32::try_from(elapsed).expect("timed batch must fit in u32 nanoseconds"))
            / f64::from(calls),
        counters.classified,
        counters.partitions,
        counters.swaps,
        counters.fallback
    );
}
