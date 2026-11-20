use stable_flag_compaction::{cursor, ranks, reference, runs, tiles};
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Instant;

fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    if args.last().is_some_and(|arg| arg == "--bench") {
        args.pop();
    }
    if args.len() == 1 {
        return;
    }
    let [_, candidate, size, shape, repeats] = args.as_slice() else {
        panic!(
            "usage: compact <cursor|ranks|tiles|runs> <size> <1|50|99|alternate|long> <repeats>"
        );
    };
    let n: usize = size.parse().unwrap();
    let repeats: usize = repeats.parse().unwrap();
    assert!(repeats > 0);
    let values: Vec<_> = (0..n).map(|i| (i % 1021) as u32).collect();
    let mut seed = 0x3534_2532_1530_7711_u64;
    let keep: Vec<_> = (0..n)
        .map(|i| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            match shape.as_str() {
                "1" => seed.is_multiple_of(100),
                "50" => seed % 100 < 50,
                "99" => seed % 100 < 99,
                "alternate" => i.is_multiple_of(2),
                "long" => (i / 256).is_multiple_of(2),
                _ => panic!("unknown shape"),
            }
        })
        .collect();
    let expected = reference(&values, &keep);
    let mut output = vec![u32::MAX; n];
    let width = NonZeroUsize::new(256).unwrap();
    let mut scratch = match candidate.as_str() {
        "ranks" => vec![0; n],
        "tiles" => vec![0; n.div_ceil(width.get())],
        _ => Vec::new(),
    };
    let mut run = |iterations| match candidate.as_str() {
        "cursor" => time(iterations, &mut output, |out| cursor(&values, &keep, out)),
        "ranks" => time(iterations, &mut output, |out| {
            ranks(&values, &keep, out, &mut scratch)
        }),
        "tiles" => time(iterations, &mut output, |out| {
            tiles(&values, &keep, out, width, &mut scratch)
        }),
        "runs" => time(iterations, &mut output, |out| runs(&values, &keep, out)),
        _ => panic!("unknown candidate"),
    };
    let (count, _) = run(3);
    let (last, elapsed) = run(repeats);
    assert_eq!(count, expected.len());
    assert_eq!(last, expected.len());
    assert_eq!(&output[..count], expected);
    assert!(output[count..].iter().all(|&value| value == u32::MAX));
    let selected_runs = keep
        .iter()
        .enumerate()
        .filter(|&(i, selected)| *selected && (i == 0 || !keep[i - 1]))
        .count();
    println!("{count},{selected_runs},{elapsed}");
}

fn time(
    repeats: usize,
    output: &mut [u32],
    mut compact: impl FnMut(&mut [u32]) -> usize,
) -> (usize, u128) {
    let start = Instant::now();
    let mut count = 0;
    for _ in 0..repeats {
        count = black_box(compact(black_box(output)));
    }
    (count, start.elapsed().as_nanos())
}
