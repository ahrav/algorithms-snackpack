//! Fixed multi-pattern experiment. Run `example` or `candidate case endpoint`.
use std::hint::black_box;
use std::time::Instant;
use topic043::{Dense, Hit, Sparse, direct};

fn fixture(case: usize) -> (Vec<Vec<u8>>, Vec<u8>) {
    let (n, r, kind) = [
        (64, 2, 0),
        (64, 32, 1),
        (32768, 2, 0),
        (32768, 32, 0),
        (32768, 128, 0),
        (262_144, 128, 0),
        (32768, 32, 1),
        (32768, 128, 1),
        (4096, 8, 2),
        (4096, 32, 2),
        (32768, 32, 2),
    ][case];
    let mut seed = 0x79ab_1357_u64;
    let mut byte = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed.to_le_bytes()[0]
    };
    let patterns = (0..r)
        .map(|id| match kind {
            0 => (0..12).map(|_| byte()).collect(),
            1 => {
                let mut p = vec![b'a'; 31];
                p.push(u8::try_from(id).unwrap() + 128);
                p
            }
            _ => vec![b'a'; id + 1],
        })
        .collect();
    let text = if kind == 0 {
        (0..n).map(|_| byte()).collect()
    } else {
        vec![b'a'; n]
    };
    (patterns, text)
}

fn oracle(p: &[Vec<u8>], text: &[u8]) -> Vec<Hit> {
    let mut result = Vec::new();
    for (id, pat) in p.iter().enumerate() {
        for start in 0..=text.len() {
            if pat.len() <= text.len() - start
                && pat.iter().enumerate().all(|(j, b)| *b == text[start + j])
            {
                result.push(Hit {
                    id,
                    start,
                    end: start + pat.len(),
                });
            }
        }
    }
    result.sort_unstable();
    result
}
fn digest(h: Hit) -> u64 {
    (u64::try_from(h.id).unwrap() + 1)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(u64::try_from(h.start).unwrap().rotate_left(17))
        .wrapping_add(u64::try_from(h.end).unwrap().rotate_left(31))
}
/// Execute the fixed experiment.
pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_none_or(|s| s == "example") {
        let p: Vec<Vec<u8>> = ["he", "she", "hers", "his", "he"]
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect();
        let mut got = Vec::new();
        Sparse::new(&p).unwrap().scan(b"ushers", |h| got.push(h));
        let mut want = oracle(&p, b"ushers");
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want);
        println!("{got:?}");
        return;
    }
    let candidate = &args[1];
    let case: usize = args[2].parse().unwrap();
    let endpoint = &args[3];
    let (p, text) = fixture(case);
    let want = oracle(&p, &text);
    let sparse = Sparse::new(&p).unwrap();
    let dense = Dense::new(&p).unwrap();
    let mut got = Vec::new();
    match candidate.as_str() {
        "direct" => direct(&p, &text, |h| got.push(h)).unwrap(),
        "sparse" => sparse.scan(&text, |h| got.push(h)),
        "dense" => dense.scan(&text, |h| got.push(h)),
        _ => panic!("unknown candidate"),
    }
    got.sort_unstable();
    assert_eq!(got, want, "complete independent oracle");
    let expected = want
        .iter()
        .fold(0_u64, |sum, &h| sum.wrapping_add(digest(h)));
    let matches = want.len();
    drop(got);
    drop(want);
    let reps = if endpoint == "scan" {
        (2_000_000 / (text.len() * p.len())).clamp(1, 2000)
    } else {
        (20000 / (p.iter().map(Vec::len).sum::<usize>() + 256 * sparse.states())).clamp(1, 1000)
    };
    let mut checksum = 0_u64;
    let start = Instant::now();
    for _ in 0..reps {
        if endpoint == "scan" {
            let mut sum = 0_u64;
            let mut emit = |h| sum = sum.wrapping_add(digest(h));
            match candidate.as_str() {
                "direct" => direct(black_box(&p), black_box(&text), &mut emit).unwrap(),
                "sparse" => sparse.scan(black_box(&text), &mut emit),
                _ => dense.scan(black_box(&text), &mut emit),
            }
            checksum = checksum.wrapping_add(black_box(sum));
        } else {
            match candidate.as_str() {
                "direct" => {
                    let copy = black_box(p.clone());
                    black_box(&copy);
                }
                "sparse" => {
                    black_box(Sparse::new(black_box(&p)).unwrap());
                }
                _ => {
                    black_box(Dense::new(black_box(&p)).unwrap());
                }
            }
        }
    }
    let elapsed = start.elapsed().as_nanos();
    if endpoint == "scan" {
        assert_eq!(
            checksum,
            expected.wrapping_mul(u64::try_from(reps).unwrap())
        );
    }
    println!(
        "{{\"candidate\":\"{candidate}\",\"case\":{case},\"endpoint\":\"{endpoint}\",\"n\":{},\"r\":{},\"states\":{},\"table_bytes\":{},\"matches\":{matches},\"reps\":{reps},\"elapsed_ns\":{elapsed},\"checksum\":{checksum}}}",
        text.len(),
        p.len(),
        sparse.states(),
        dense.table_bytes()
    );
}
