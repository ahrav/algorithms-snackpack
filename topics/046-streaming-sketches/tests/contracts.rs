//! Public batch boundary checks.
use topic_046_streaming_sketches::{evaluate, mix};

fn uniform(n: u64) -> Vec<u64> {
    (0..n).map(|i| mix(i, 123) % 4096).collect()
}

fn counter(key: u64, seed: u64, row: u64) -> usize {
    usize::try_from(row * 256 + mix(key, seed + row) % 256).unwrap()
}

fn hll_registers(data: &[u64], seed: u64) -> Vec<u64> {
    let mut registers = vec![0; 1024];
    for &key in data {
        let hash = mix(key, seed);
        let low = hash & ((1 << 54) - 1);
        let rank = u64::from(low.leading_zeros()) - 9;
        let index = usize::try_from(hash >> 54).unwrap();
        registers[index] = rank.max(registers[index]);
    }
    registers
}

#[test]
fn running_example_answers() {
    let data = [4, 7, 4, 9, 4];
    assert_eq!(
        evaluate(&data, &[4, 8], "exact-frequency", 7)
            .unwrap()
            .values,
        [3, 0]
    );
    assert_eq!(
        evaluate(&data, &[], "exact-distinct", 7)
            .unwrap()
            .estimate
            .to_bits(),
        3.0_f64.to_bits()
    );
    assert_eq!(
        evaluate(&data, &[], "rank-summary", 7).unwrap().values[50],
        4
    );
}

#[test]
fn empty_input_answers_are_exact_for_each_contract() {
    for (candidate, values, state_len) in [
        ("exact-frequency", vec![0, 0], 0),
        ("count-min", vec![0, 0], 1280),
        ("exact-distinct", vec![], 0),
        ("hll", vec![], 1024),
        ("exact-quantile", vec![], 0),
        ("rank-summary", vec![], 0),
    ] {
        let r = evaluate(&[], &[0, 9], candidate, 7).unwrap();
        assert_eq!(r.values, values, "{candidate}");
        assert_eq!(r.state, vec![0; state_len], "{candidate}");
        assert_eq!(r.estimate.to_bits(), 0.0_f64.to_bits(), "{candidate}");
    }
}

#[test]
fn count_min_counters_and_row_minimum_match_independent_table() {
    let queries: Vec<u64> = (0..4096).collect();
    for (data, seed) in [(vec![0, 0, 1], 3), (uniform(4096), 7)] {
        let mut table = vec![0_u64; 1280];
        for &key in &data {
            for row in 0..5 {
                table[counter(key, seed, row)] += 1;
            }
        }
        let minimum: Vec<u64> = queries
            .iter()
            .map(|&key| {
                (0..5)
                    .map(|row| table[counter(key, seed, row)])
                    .min()
                    .unwrap()
            })
            .collect();
        let r = evaluate(&data, &queries, "count-min", seed).unwrap();
        assert_eq!(r.state, table, "{seed}");
        assert_eq!(r.values, minimum, "{seed}");
    }
    let r = evaluate(&[0, 0, 1], &[0, 1, 2], "count-min", 3).unwrap();
    assert_eq!(r.values, [2, 1, 0]);
}

#[test]
fn hll_merge_keeps_register_maxima_and_estimates_from_them() {
    let r = evaluate(&[4, 4, 7], &[], "hll", 9).unwrap();
    assert_eq!(r.state, hll_registers(&[4, 7], 9));
    assert_eq!(r.state.iter().filter(|&&rank| rank != 0).count(), 2);
    let linear = 1024.0_f64 * (1024.0_f64 / 1022.0).ln();
    assert_eq!(r.estimate.to_bits(), linear.to_bits());

    let data: Vec<u64> = (0..16384).collect();
    let r = evaluate(&data, &[], "hll", 7).unwrap();
    let registers = hll_registers(&data, 7);
    assert_eq!(r.state, registers);
    let harmonic: f64 = registers
        .iter()
        .map(|&rank| 2.0_f64.powi(-i32::try_from(rank).unwrap()))
        .sum();
    let raw = 0.7213 / (1.0 + 1.079 / 1024.0) * 1024.0 * 1024.0 / harmonic;
    assert!(
        (r.estimate - raw).abs() <= 1e-9 * raw,
        "{} {raw}",
        r.estimate
    );
    assert!((r.estimate - 16384.0).abs() <= 1638.4, "{}", r.estimate);
}

#[test]
fn exact_quantile_returns_nearest_rank_values() {
    let r = evaluate(&[4, 7, 4, 9, 4], &[], "exact-quantile", 7).unwrap();
    assert_eq!(r.values, [vec![4; 75], vec![7; 25], vec![9]].concat());
}

#[test]
fn compressed_rank_summary_keeps_101_observed_values_and_exact_endpoints() {
    for shape in 0..4 {
        let data: Vec<u64> = (0..4096)
            .map(|i| match shape {
                0 => i,
                1 => 4095 - i,
                2 => mix(i, 9) % 23,
                _ => mix(i, 2),
            })
            .collect();
        let r = evaluate(&data, &[], "rank-summary", 0).unwrap();
        assert!(r.state.len() < 3 * data.len(), "{shape}");
        let gaps: u64 = r.state.chunks_exact(3).map(|entry| entry[1]).sum();
        assert_eq!(gaps, 4096, "{shape}");
        assert_eq!(r.values.len(), 101, "{shape}");
        assert_eq!(r.values[0], *data.iter().min().unwrap(), "{shape}");
        assert_eq!(r.values[100], *data.iter().max().unwrap(), "{shape}");
        assert!(r.values.iter().all(|v| data.contains(v)), "{shape}");
    }
}
