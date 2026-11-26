//! One independent timing process. A separate script assigns process order.

use std::hint::black_box;
use std::time::Instant;

use associative_reductions_and_deferred_merges::{History, Policy, reference};

fn fixture(name: &str) -> (usize, usize, usize, usize) {
    // Values, measurements per retained interval, reads, transaction repeats.
    match name {
        "tiny" => (5, 2, 2, 20_000),
        "large-once" => (262_144, 65_536, 1, 64),
        "large-read" => (262_144, 65_536, 64, 8),
        "dense-cuts" => (16_384, 1, 16, 24),
        "medium-stripes" => (32_768, 128, 8, 96),
        "short-stripes" => (32_768, 8, 1, 96),
        _ => panic!("unknown fixture {name}"),
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 1 || args.get(1).is_some_and(|arg| arg == "--bench") {
        return;
    }
    let [_, name, arm] = args.as_slice() else {
        panic!("usage: reduction FIXTURE ARM");
    };
    let policy = match arm.as_str() {
        "raw" | "raw-a" | "raw-b" => Policy::Raw,
        "chunked" => Policy::Chunked,
        "compact" => Policy::Compact,
        _ => panic!("unknown arm {arm}"),
    };
    let (len, step, reads, repeats) = fixture(name);
    let values: Vec<_> = (0..len)
        .map(|i| i32::try_from((i * 7919 + 17) % 65521).unwrap() - 32_760)
        .collect();
    let ends: Vec<_> = (step..len).step_by(step).chain([len]).collect();
    let snapshots: Vec<_> = (0..reads)
        .map(|i| {
            if i % 2 == 0 {
                ends.len()
            } else {
                (i * 97) % ends.len() + 1
            }
        })
        .collect();
    let check = History::build(&values, &ends, policy).unwrap();
    let visits: usize = snapshots.iter().map(|&at| check.visits(at).unwrap()).sum();
    let (buffers, bytes) = check.reserved();
    let mut expected_digest = 0_u64;
    for &at in &snapshots {
        let expected = reference(&values, ends[at - 1]).unwrap();
        assert_eq!(check.read(at), Some(expected), "{name} snapshot {at}");
        expected_digest = expected_digest.rotate_left(5)
            ^ u64::from_ne_bytes(expected.0.to_ne_bytes())
            ^ expected.1;
    }
    drop(check);

    let (mut build_ns, mut read_ns, mut drop_ns, mut total_ns) = (0, 0, 0, 0);
    for _ in 0..repeats {
        let start = Instant::now();
        let history = History::build(black_box(&values), black_box(&ends), policy).unwrap();
        let built = Instant::now();
        let mut digest = 0_u64;
        for &at in black_box(&snapshots) {
            let (sum, count) = history.read(at).unwrap();
            digest = digest.rotate_left(5) ^ u64::from_ne_bytes(sum.to_ne_bytes()) ^ count;
        }
        black_box(digest);
        let queried = Instant::now();
        drop(black_box(history));
        let ended = Instant::now();
        assert_eq!(digest, expected_digest);
        build_ns += (built - start).as_nanos();
        read_ns += (queried - built).as_nanos();
        drop_ns += (ended - queried).as_nanos();
        total_ns += (ended - start).as_nanos();
    }
    println!(
        "{name} {arm} {len} {} {reads} {repeats} {visits} {buffers} {bytes} {expected_digest:016x} {build_ns} {read_ns} {drop_ns} {total_ns}",
        ends.len()
    );
}
