//! Batch lab for insertion-only frequency, cardinality, and rank summaries.
//! Inputs are at most one million `u64` observations. Hash seeds are explicit.
//! Error guarantees depend on the hash assumptions documented in the guide.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "fixed lab dimensions and at most one million observations"
)]
use std::collections::{HashMap, HashSet};

/// A batch's answers and inspectable state. Byte counts cover retained entries,
/// excluding container capacity, allocator overhead and the input buffer.
#[derive(Debug)]
pub struct Report {
    /// Frequency for each supplied query, or 101 nearest-rank quantiles.
    pub values: Vec<u64>,
    /// Count-Min counters, HLL registers, or rank-summary triples.
    pub state: Vec<u64>,
    /// Distinct estimate, or zero for other contracts.
    pub estimate: f64,
    /// Logical retained data bytes, excluding container overhead.
    pub retained_bytes: usize,
}

/// Deterministic `SplitMix64` finalizer with an additive seed.
/// This is a reproducible mixer, with empirical rather than proven independent rows.
#[must_use]
pub fn mix(value: u64, seed: u64) -> u64 {
    let mut x = value.wrapping_add(seed).wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

const WIDTH: usize = 256;
const DEPTH: usize = 5;
const REGISTERS: usize = 1024;
const P: u32 = 10;

struct CountMin {
    counters: Vec<u64>,
    seed: u64,
}
impl CountMin {
    fn new(seed: u64) -> Self {
        Self {
            counters: vec![0; WIDTH * DEPTH],
            seed,
        }
    }
    fn index(&self, value: u64, row: usize) -> usize {
        row * WIDTH + (mix(value, self.seed.wrapping_add(row as u64)) & 255) as usize
    }
    fn add(&mut self, value: u64) {
        for row in 0..DEPTH {
            let i = self.index(value, row);
            self.counters[i] += 1;
        }
    }
    fn query(&self, value: u64) -> u64 {
        (0..DEPTH)
            .map(|row| self.counters[self.index(value, row)])
            .min()
            .unwrap_or(0)
    }
    fn merge(&mut self, other: &Self) -> bool {
        if self.seed != other.seed {
            return false;
        }
        for (a, b) in self.counters.iter_mut().zip(&other.counters) {
            *a += b;
        }
        true
    }
}

struct Hll {
    registers: Vec<u8>,
    seed: u64,
}
impl Hll {
    fn new(seed: u64) -> Self {
        Self {
            registers: vec![0; REGISTERS],
            seed,
        }
    }
    fn add(&mut self, value: u64) {
        let hash = mix(value, self.seed);
        let index = (hash >> (64 - P)) as usize;
        let rank = ((hash << P).leading_zeros() + 1).min(65 - P) as u8;
        self.registers[index] = self.registers[index].max(rank);
    }
    #[allow(
        clippy::naive_bytecount,
        reason = "scan only 1024 bytes without an added dependency"
    )]
    fn estimate(&self) -> f64 {
        let m = REGISTERS as f64;
        let harmonic: f64 = self
            .registers
            .iter()
            .map(|&r| 2.0_f64.powi(-i32::from(r)))
            .sum();
        let raw = (0.7213 / (1.0 + 1.079 / m)) * m * m / harmonic;
        let zeros = self.registers.iter().filter(|&&r| r == 0).count();
        if raw <= 2.5 * m && zeros > 0 {
            m * (m / zeros as f64).ln()
        } else {
            raw
        }
    }
    fn merge(&mut self, other: &Self) -> bool {
        if self.seed != other.seed {
            return false;
        }
        for (a, b) in self.registers.iter_mut().zip(&other.registers) {
            *a = (*a).max(*b);
        }
        true
    }
}

#[derive(Clone, Copy)]
struct Entry {
    value: u64,
    gap: usize,
    delta: usize,
}
// GK-style rank intervals, with an eager adjacent-pair compressor. The paper's
// banded compression space theorem is outside this simplified implementation.
struct RankSummary {
    entries: Vec<Entry>,
    n: usize,
}
impl RankSummary {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            n: 0,
        }
    }
    fn add(&mut self, value: u64) {
        self.n += 1;
        let pos = self.entries.partition_point(|e| e.value <= value);
        let bound = self.n / 25; // floor(2 epsilon N), epsilon = 1/50
        let delta = if pos == 0 || pos == self.entries.len() {
            0
        } else {
            bound.saturating_sub(1)
        };
        self.entries.insert(
            pos,
            Entry {
                value,
                gap: 1,
                delta,
            },
        );
        if self.entries.len() > 2 {
            for i in (1..self.entries.len() - 1).rev() {
                if self.entries[i].gap + self.entries[i + 1].gap + self.entries[i + 1].delta
                    <= bound
                {
                    self.entries[i + 1].gap += self.entries[i].gap;
                    self.entries.remove(i);
                }
            }
        }
    }
    fn query(&self, rank: usize) -> u64 {
        if self.entries.is_empty() {
            return 0;
        }
        if rank == 1 {
            return self.entries[0].value;
        }
        if rank == self.n {
            return self.entries[self.entries.len() - 1].value;
        }
        let mut lower = 0;
        let mut previous = self.entries[0].value;
        for e in &self.entries {
            lower += e.gap;
            if lower + e.delta > rank + self.n / 50 {
                return previous;
            }
            previous = e.value;
        }
        previous
    }
}

/// Execute one fresh summary plus all queries. Empty quantile output is empty.
/// `queries` controls frequency lookups; rank queries use 101 evenly spaced ranks.
/// Cardinality estimates support this lab's at-most-one-million-element range.
///
/// # Errors
/// Returns an error for an unknown candidate or a batch above one million items.
#[allow(
    clippy::cast_precision_loss,
    reason = "bounded lab cardinalities fit f64 exactly"
)]
pub fn evaluate(
    data: &[u64],
    queries: &[u64],
    candidate: &str,
    seed: u64,
) -> Result<Report, &'static str> {
    if data.len() > 1_000_000 {
        return Err("batch limit exceeded");
    }
    let mut out = Report {
        values: Vec::new(),
        state: Vec::new(),
        estimate: 0.0,
        retained_bytes: 0,
    };
    match candidate {
        "exact-frequency" => {
            let mut counts = HashMap::<u64, u64>::new();
            for &x in data {
                *counts.entry(x).or_default() += 1;
            }
            out.values = queries
                .iter()
                .map(|x| *counts.get(x).unwrap_or(&0))
                .collect();
            out.retained_bytes = counts.len() * 16;
        }
        "count-min" => {
            let middle = data.len() / 2;
            let mut a = CountMin::new(seed);
            let mut b = CountMin::new(seed);
            for &x in &data[..middle] {
                a.add(x);
            }
            for &x in &data[middle..] {
                b.add(x);
            }
            a.merge(&b);
            out.values = queries.iter().map(|&x| a.query(x)).collect();
            out.retained_bytes = a.counters.len() * 8;
            out.state = a.counters;
        }
        "exact-distinct" => {
            let distinct: HashSet<u64> = data.iter().copied().collect();
            out.estimate = distinct.len() as f64;
            out.retained_bytes = distinct.len() * 8;
        }
        "hll" => {
            let middle = data.len() / 2;
            let mut a = Hll::new(seed);
            let mut b = Hll::new(seed);
            for &x in &data[..middle] {
                a.add(x);
            }
            for &x in &data[middle..] {
                b.add(x);
            }
            a.merge(&b);
            out.estimate = a.estimate();
            out.retained_bytes = a.registers.len();
            out.state = a.registers.iter().map(|&x| u64::from(x)).collect();
        }
        "exact-quantile" => {
            let mut sorted = data.to_vec();
            sorted.sort_unstable();
            if !sorted.is_empty() {
                out.values = (0..=100)
                    .map(|q| sorted[(sorted.len() - 1) * q / 100])
                    .collect();
            }
            out.retained_bytes = sorted.len() * 8;
        }
        "rank-summary" => {
            let mut summary = RankSummary::new();
            for &x in data {
                summary.add(x);
            }
            if !data.is_empty() {
                out.values = (0..=100)
                    .map(|q| summary.query(1 + (data.len() - 1) * q / 100))
                    .collect();
            }
            out.retained_bytes = summary.entries.len() * std::mem::size_of::<Entry>();
            out.state = summary
                .entries
                .iter()
                .flat_map(|e| [e.value, e.gap as u64, e.delta as u64])
                .collect();
        }
        _ => return Err("unknown candidate"),
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_endpoints_and_bad_candidate() {
        for c in [
            "exact-frequency",
            "count-min",
            "exact-distinct",
            "hll",
            "exact-quantile",
            "rank-summary",
        ] {
            let r = evaluate(&[], &[0], c, 7).unwrap();
            assert_eq!(r.estimate.to_bits(), 0.0_f64.to_bits());
            assert!(r.values.is_empty() || r.values == [0]);
        }
        assert_eq!(
            evaluate(&[], &[], "bad", 0).unwrap_err(),
            "unknown candidate"
        );
        assert_eq!(
            evaluate(&vec![0; 1_000_001], &[], "hll", 0).unwrap_err(),
            "batch limit exceeded"
        );
        let r = evaluate(&[u64::MAX, 0, 0, 7], &[], "rank-summary", 0).unwrap();
        assert_eq!(r.values[0], 0);
        assert_eq!(r.values[100], u64::MAX);
    }
    #[test]
    fn compatible_merge_and_duplicate_semantics() {
        let mut c = CountMin::new(9);
        let mut d = CountMin::new(9);
        c.add(4);
        d.add(4);
        assert!(c.merge(&d));
        assert_eq!(c.query(4), 2);
        let saved = c.counters.clone();
        assert!(!c.merge(&CountMin::new(10)));
        assert_eq!(c.counters, saved);
        let mut h = Hll::new(9);
        h.add(4);
        let saved = h.registers.clone();
        h.add(4);
        assert_eq!(h.registers, saved);
        let mut d = Hll::new(9);
        d.add(7);
        let mut whole = Hll::new(9);
        whole.add(4);
        whole.add(7);
        assert!(h.merge(&d));
        assert_eq!(h.registers, whole.registers);
        let saved = h.registers.clone();
        assert!(!h.merge(&Hll::new(10)));
        assert_eq!(h.registers, saved);
    }
    #[test]
    fn exhaustive_small_rank_and_count_contracts() {
        // Independent counts by filtering and quantiles by sorted rank intervals.
        for n in 0..=8_u32 {
            for encoded in 0..3_usize.pow(n) {
                let mut v = encoded;
                let data: Vec<u64> = (0..n)
                    .map(|_| {
                        let x = (v % 3) as u64;
                        v /= 3;
                        x
                    })
                    .collect();
                let c = evaluate(&data, &[0, 1, 2, 3], "count-min", 3).unwrap();
                for x in 0..=3 {
                    assert!(c.values[x] >= data.iter().filter(|&&y| y == x as u64).count() as u64);
                }
                let q = evaluate(&data, &[], "rank-summary", 0).unwrap();
                let mut sorted = data.clone();
                sorted.sort_unstable();
                for (i, &value) in q.values.iter().enumerate() {
                    assert_eq!(value, sorted[(sorted.len() - 1) * i / 100]);
                }
            }
        }
    }
    #[test]
    fn large_rank_intervals_include_duplicates_and_order() {
        for shape in 0..4 {
            let data: Vec<u64> = (0..4096)
                .map(|i| match shape {
                    0 => i,
                    1 => 4095 - i,
                    2 => mix(i, 9) % 23,
                    _ => mix(i, 2),
                })
                .collect();
            let q = evaluate(&data, &[], "rank-summary", 0).unwrap();
            for (i, &value) in q.values.iter().enumerate() {
                let r = 1 + (data.len() - 1) * i / 100;
                let lo = data.iter().filter(|&&x| x < value).count() + 1;
                let hi = data.iter().filter(|&&x| x <= value).count();
                assert!(
                    lo <= r + data.len() / 50 && hi + data.len() / 50 >= r,
                    "{shape} {i} {lo} {hi} {r}"
                );
            }
        }
    }
}
