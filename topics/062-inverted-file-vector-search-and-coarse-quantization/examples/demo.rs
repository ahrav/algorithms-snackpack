//! A boundary miss and a compressed-score tie in the same corpus.
use inverted_file_vector_search_and_coarse_quantization::{Index, Mode, Vector};

fn axis(x: i16) -> Vector {
    [x, 0, 0, 0, 0, 0, 0, 0]
}

fn main() {
    let index = Index::new(
        [0, 2, 3, 6, 10].map(axis).to_vec(),
        [0, 10].map(axis).to_vec(),
    );
    let q = axis(5);
    for (label, mode, expected) in [
        ("flat", Mode::Flat, 3),
        ("raw one list", Mode::Raw(1), 2),
        ("PQ one list", Mode::Pq(1), 0),
        (
            "rerank two",
            Mode::Rerank {
                probes: 1,
                shortlist: 2,
            },
            1,
        ),
        (
            "rerank three",
            Mode::Rerank {
                probes: 1,
                shortlist: 3,
            },
            2,
        ),
        ("raw both lists", Mode::Raw(2), 3),
    ] {
        let result = index.search(&q, 1, mode);
        assert_eq!(result.hits[0].1, expected);
        println!("{label}: ID {expected}, distance² {}", result.hits[0].0);
    }
}
