//! Demonstrate the example history and run a single benchmark process.

use k_way_merge_tombstones_and_snapshots::{
    Point, RangeDelete, Rows, Selector, measure_work, read, replay,
};
use std::hint::black_box;
use std::time::Instant;

const SELECTORS: [Selector; 3] = [Selector::Flat, Selector::Heap, Selector::Loser];

fn point(key: u32, seq: u64, value: Option<u32>) -> Point {
    Point { key, seq, value }
}

fn demo() {
    let runs = [
        vec![
            point(8, 2, Some(65)),
            point(10, 3, Some(66)),
            point(14, 12, None),
        ],
        vec![
            point(10, 9, Some(70)),
            point(11, 4, Some(67)),
            point(12, 5, Some(68)),
        ],
        vec![point(12, 11, Some(71)), point(14, 6, Some(69))],
    ];
    let ranges = [
        RangeDelete {
            start: 10,
            end: 13,
            seq: 7,
        },
        RangeDelete {
            start: 11,
            end: 14,
            seq: 10,
        },
    ];
    for snapshot in [6, 8, 10, 12] {
        let rows = replay(&runs, &ranges, snapshot);
        for selector in SELECTORS {
            assert_eq!(rows, read(&runs, &ranges, snapshot, selector));
        }
        let display: Vec<_> = rows
            .into_iter()
            .map(|(key, value)| (key, char::from_u32(value).unwrap()))
            .collect();
        println!("{snapshot}: {display:?}");
    }
}

fn word(seed: &mut u64) -> u32 {
    *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    u32::try_from(*seed >> 32).unwrap()
}

struct Fixture {
    runs: Vec<Vec<Point>>,
    ranges: Vec<RangeDelete>,
    snapshot: u64,
}

impl Fixture {
    fn new(case: &str) -> Self {
        let (k, n, t, keys, span, layout) = match case {
            "tiny" => (2, 32, 2, 32, 3, "mixed"),
            "low-fan-in" => (2, 16_384, 64, 8192, 8, "mixed"),
            "mixed-8" => (8, 16_384, 64, 8192, 8, "mixed"),
            "mixed-32" => (32, 16_384, 64, 8192, 8, "mixed"),
            "overlap-32" => (32, 16_384, 2048, 1024, 768, "mixed"),
            "skew-64" => (64, 16_384, 64, 8192, 8, "skew"),
            "disjoint-32" => (32, 16_384, 64, 8192, 8, "disjoint"),
            _ => panic!("unknown fixture {case}"),
        };
        let mut seed = 5917;
        let mut runs = vec![Vec::new(); k];
        for i in 0..n {
            let random = word(&mut seed);
            let run = if layout == "skew" {
                if i % 10 == 0 {
                    1 + (i / 10) % (k - 1)
                } else {
                    0
                }
            } else {
                i % k
            };
            let key = if layout == "disjoint" {
                u32::try_from(run).unwrap() * (keys / 32) + random % (keys / 32)
            } else {
                random % keys
            };
            let payload = word(&mut seed);
            runs[run].push(point(
                key,
                (2 * i) as u64,
                (!payload.is_multiple_of(11)).then_some(payload),
            ));
        }
        for run in &mut runs {
            run.sort_unstable_by_key(|p| p.order());
        }
        let mut ranges: Vec<_> = (0..t)
            .map(|i| {
                let start = word(&mut seed) % keys;
                RangeDelete {
                    start,
                    end: start + 1 + word(&mut seed) % span,
                    seq: (2 * (i * n / t) + 1) as u64,
                }
            })
            .collect();
        ranges.reverse();
        Self {
            runs,
            ranges,
            snapshot: (3 * n / 2) as u64,
        }
    }

    fn run(&self, candidate: &str) -> Rows {
        let runs = black_box(&self.runs);
        let ranges = black_box(&self.ranges);
        let snapshot = black_box(self.snapshot);
        let selector = match candidate {
            "replay" => return replay(runs, ranges, snapshot),
            "flat" => Selector::Flat,
            "heap" => Selector::Heap,
            "loser" => Selector::Loser,
            _ => panic!("unknown candidate {candidate}"),
        };
        read(runs, ranges, snapshot, selector)
    }

    fn check(&self) {
        let expected = replay(&self.runs, &self.ranges, self.snapshot);
        let mut comparisons = Vec::new();
        for selector in SELECTORS {
            let (rows, count) = measure_work(&self.runs, &self.ranges, self.snapshot, selector);
            assert_eq!(rows, expected, "{selector:?} counted");
            assert_eq!(
                read(&self.runs, &self.ranges, self.snapshot, selector),
                expected,
                "{selector:?}"
            );
            comparisons.push(count);
        }
        let rows: Vec<_> = expected
            .into_iter()
            .map(|(key, value)| [key, value])
            .collect();
        println!(
            "{{\"comparisons\":{comparisons:?},\"rows\":{rows:?},\"runs\":{},\"points\":{},\"ranges\":{}}}",
            self.runs.len(),
            self.runs.iter().map(Vec::len).sum::<usize>(),
            self.ranges.len()
        );
    }

    fn bench(&self, candidate: &str, repeats: u32) {
        for _ in 0..3 {
            black_box(self.run(candidate));
        }
        let start = Instant::now();
        for _ in 0..repeats {
            black_box(self.run(candidate));
        }
        let elapsed = start.elapsed().as_nanos();
        println!("{elapsed}");
    }
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd] if cmd == "demo" => demo(),
        [cmd, case] if cmd == "check" => Fixture::new(case).check(),
        [cmd, case, arm, repeats] if cmd == "bench" => {
            Fixture::new(case).bench(arm, repeats.parse().unwrap());
        }
        _ => panic!("usage: snapshot demo | check CASE | bench CASE ARM REPEATS"),
    }
}
