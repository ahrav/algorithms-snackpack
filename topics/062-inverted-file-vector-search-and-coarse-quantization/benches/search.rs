//! Fresh-process query batches and untimed exact-oracle quality checks.

use std::{env, hint::black_box, time::Instant};

use inverted_file_vector_search_and_coarse_quantization::{
    Hit, Index, Mode, Search, Vector, reference,
};

const K: usize = 5;
const QUERIES: usize = 96;
const MODES: [(&str, Mode); 7] = [
    ("flat", Mode::Flat),
    ("raw1", Mode::Raw(1)),
    ("raw4", Mode::Raw(4)),
    ("pq1", Mode::Pq(1)),
    ("pq4", Mode::Pq(4)),
    (
        "rr1",
        Mode::Rerank {
            probes: 1,
            shortlist: 16,
        },
    ),
    (
        "rr4",
        Mode::Rerank {
            probes: 4,
            shortlist: 16,
        },
    ),
];

fn next(state: &mut u64, bound: u64) -> i16 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    i16::try_from((*state >> 32) % bound).unwrap()
}

fn workload(label: &str) -> (Vec<Vector>, Vec<Vector>, Vec<Vector>, usize) {
    let (n, spacing, repetitions) = match label {
        "tiny" => (256, 96, 10),
        "balanced" | "skew" => (16_384, 96, 3),
        "boundary" => (16_384, 16, 3),
        _ => panic!("unknown workload {label}"),
    };
    let centers: Vec<Vector> = (0_i16..16)
        .map(|j| [j * spacing, 0, 0, 0, 0, 0, 0, 0])
        .collect();
    let vectors = |mut state, count, query: bool| {
        (0..count)
            .map(|_| {
                let mut group = next(&mut state, 16);
                if label == "skew" && next(&mut state, 100) < if query { 75 } else { 90 } {
                    group = 0;
                }
                let center = centers[usize::try_from(group).unwrap()];
                let mut vector = center.map(|x| {
                    x + if label == "balanced" {
                        (next(&mut state, 3) - 1) * 12
                    } else if label == "boundary" {
                        next(&mut state, 47) - 23
                    } else {
                        next(&mut state, 29) - 14
                    }
                });
                if query && label == "boundary" && group < 15 {
                    vector[0] = center[0] + 8;
                }
                vector
            })
            .collect::<Vec<_>>()
    };
    (
        vectors(23, n, false),
        centers.clone(),
        vectors(97, QUERIES, true),
        repetitions,
    )
}

fn fold(mut signature: u64, result: &Search) -> u64 {
    for &(score, id) in &result.hits {
        signature = signature.rotate_left(7) ^ u64::try_from(score).unwrap();
        signature = signature.rotate_left(9) ^ u64::try_from(id).unwrap();
    }
    signature ^ u64::try_from(result.scanned).unwrap()
}

fn coverage(hits: &[Hit], truth: &[Hit]) -> usize {
    truth
        .iter()
        .filter(|&&(_, id)| hits.iter().any(|&(_, found)| found == id))
        .count()
}

fn quality(rows: &[Vector], index: &Index, queries: &[Vector]) {
    let truth: Vec<_> = queries.iter().map(|q| reference(rows, q, K)).collect();
    let controls = [("raw_all", Mode::Raw(16)), ("pq_all", Mode::Pq(16))];
    for (name, mode) in MODES.into_iter().chain(controls) {
        let mut scans = Vec::new();
        let mut recall = Vec::new();
        let mut reranked = 0;
        let mut signature = 0;
        for (q, exact) in queries.iter().zip(&truth) {
            let result = index.search(q, K, mode);
            if matches!(mode, Mode::Flat | Mode::Raw(16)) {
                assert_eq!(&result.hits, exact);
            }
            if let Mode::Rerank { probes, shortlist } = mode {
                let pool = index.search(q, shortlist, Mode::Pq(probes)).hits;
                let mut ids: Vec<_> = pool.iter().map(|&(_, id)| id).collect();
                ids.sort_unstable();
                let originals: Vec<_> = ids.iter().map(|&id| rows[id]).collect();
                let ordered: Vec<_> = reference(&originals, q, K)
                    .into_iter()
                    .map(|(score, local_id)| (score, ids[local_id]))
                    .collect();
                assert_eq!(result.hits, ordered);
                assert_eq!(coverage(&pool, exact), coverage(&result.hits, exact));
            }
            signature = fold(signature, &result);
            scans.push(result.scanned);
            recall.push(coverage(&result.hits, exact));
            reranked += result.reranked;
        }
        scans.sort_unstable();
        recall.sort_unstable();
        println!(
            "{name},{},{},{},{},{},{signature}",
            recall.iter().sum::<usize>(),
            recall[QUERIES / 10],
            scans.iter().sum::<usize>(),
            scans[(QUERIES * 95).div_ceil(100) - 1],
            reranked
        );
    }
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.is_empty() || args == ["--bench"] {
        return;
    }
    let [cell, variant] = args.as_slice() else {
        panic!("pass workload and variant");
    };
    let (rows, centers, queries, repetitions) = workload(cell);
    if variant == "quality" {
        let index = Index::new(rows.clone(), centers);
        quality(&rows, &index, &queries);
        return;
    }
    let mode = MODES
        .iter()
        .find(|(name, _)| *name == variant || (*name == "flat" && variant == "flat_b"))
        .unwrap()
        .1;
    let start = Instant::now();
    let index = Index::new(rows, centers);
    let build_ns = start.elapsed().as_nanos();
    for q in &queries {
        black_box(index.search(q, K, mode));
    }
    let start = Instant::now();
    let mut signature = 0;
    for _ in 0..repetitions {
        signature = 0;
        for q in &queries {
            signature = fold(signature, &black_box(&index).search(black_box(q), K, mode));
        }
        black_box(signature);
    }
    println!(
        "{},{build_ns},{},{}",
        start.elapsed().as_nanos(),
        repetitions * QUERIES,
        signature
    );
}
