//! Report fixture identity, score, and elapsed solver time for one process.

use std::{hint::black_box, time::Instant};

use topic_054_dynamic_programming::{Item, Strategy, solve};

fn fixture(name: &str) -> (u16, Vec<Item>) {
    let (n, budget) = match name {
        "tiny" => (3, 6),
        "tight" => (18, 20),
        "medium" => (18, 212),
        "small-value" | "wide-sparse" | "wide-dense" => (16, 49_151),
        _ => panic!("unknown fixture {name}"),
    };
    let pairs = (0_u16..n).map(|i| {
        let weight = (i * 73 + i * i * 7 + 13) % 47 + 1;
        let value = (i * 31 + i * i * 3 + 17) % 181 + 1;
        let (weight, value) = match name {
            "tiny" => [(2, 4), (3, 5), (4, 8)][usize::from(i)],
            "small-value" => (weight * 181, i % 3 + 1),
            "wide-sparse" => (4096 * (i % 7 + 1), value * 37),
            "wide-dense" => (1 << i, 1 << i),
            _ => (weight, value),
        };
        Item { weight, value }
    });
    (budget, pairs.collect())
}

pub(crate) fn main() {
    let args: Vec<_> = std::env::args().collect();
    let (strategy, fixture_name, repetitions) = match args.as_slice() {
        [_] => (Strategy::Rolling, "tiny", 1_000),
        [_, name, fixture_name, repetitions] => {
            let strategy = match name.as_str() {
                "exhaustive" => Strategy::Exhaustive,
                "table" => Strategy::Table,
                "rolling" => Strategy::Rolling,
                "value" => Strategy::Value,
                "frontier" => Strategy::Frontier,
                _ => panic!("unknown candidate {name}"),
            };
            (
                strategy,
                fixture_name.as_str(),
                repetitions.parse().unwrap(),
            )
        }
        _ => panic!("usage: compare [candidate fixture repetitions]"),
    };
    assert!(repetitions > 0);
    let (budget, items) = fixture(fixture_name);
    let start = Instant::now();
    let mut sum = 0_u64;
    for _ in 0..repetitions {
        let score = black_box(solve(
            black_box(strategy),
            black_box(&items),
            black_box(budget),
        ));
        sum += u64::from(score);
    }
    let elapsed = start.elapsed().as_nanos();
    let pairs: Vec<_> = items.iter().map(|item| [item.weight, item.value]).collect();
    println!("{elapsed}\t{sum}\t{budget}\t{pairs:?}");
}
