use std::hint::black_box;
use std::time::Instant;
use topic048_eviction::{Cache, Policy};

fn workload(name: &str, cap: usize, n: usize) -> Vec<usize> {
    let hot = (cap / 2).max(1);
    let mut rng = 0x1234_5678_9abc_def0u64;
    (0..n)
        .map(|i| {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let random = usize::try_from(rng % u64::try_from(4 * cap + 1).unwrap()).unwrap();
            match name {
                "tiny" => [0, 1, 0, 1, 2, 3, 4, 0, 1][i % 9],
                "hot" => {
                    if i % 16 == 15 {
                        cap + random
                    } else {
                        i % hot
                    }
                }
                "scan" => {
                    let p = i % (8 * cap);
                    if p < 4 * cap || p >= 6 * cap {
                        p % hot
                    } else {
                        cap + p
                    }
                }
                "loop" => i % (cap + 1),
                "phase" => (i % hot) + (i / (n / 4).max(1)) * cap,
                "uniform" => random,
                _ => panic!("unknown workload"),
            }
        })
        .collect()
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let policy_name = args.get(1).map_or("linked", String::as_str);
    let name = args.get(2).map_or("tiny", String::as_str);
    let cap = args.get(3).map_or(Ok(3), |x| x.parse::<usize>())?;
    let n = args.get(4).map_or(Ok(9), |x| x.parse::<usize>())?;
    let policy = match policy_name {
        "scan" => Policy::Scan,
        "linked" => Policy::Linked,
        "clock" => Policy::Clock,
        "segmented" => Policy::Segmented,
        _ => return Err("unknown policy".into()),
    };
    let trace = workload(name, cap, n);
    let universe = trace.iter().max().map_or(0, |x| x + 1);
    let start = Instant::now();
    let mut c = Cache::new(policy, cap, universe);
    let setup_ns = start.elapsed().as_nanos();
    let mut hits = Vec::with_capacity(n);
    let start = Instant::now();
    for &key in &trace {
        hits.push(black_box(c.access(black_box(key))?));
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let state = c.snapshot();
    let hit_count = hits.iter().filter(|&&h| h).count();
    let hit_string: String = hits
        .into_iter()
        .map(|h| if h { '1' } else { '0' })
        .collect();
    println!(
        "{{\"policy\":\"{policy_name}\",\"workload\":\"{name}\",\"capacity\":{cap},\"n\":{n},\"universe\":{universe},\"setup_ns\":{setup_ns},\"elapsed_ns\":{elapsed_ns},\"hits\":{hit_count},\"inspections\":{},\"list_operations\":{},\"hit_string\":\"{hit_string}\",\"queues\":{:?},\"slots\":{:?},\"bits\":{:?},\"hand\":{}}}",
        c.inspections, c.list_operations, state.queues, state.slots, state.bits, state.hand
    );
    Ok(())
}
