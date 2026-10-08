//! Safe radix partitioning with explicit bucket boundaries and stability contracts.
#![forbid(unsafe_code)]

/// A prehashed key, unique source identity, and inline payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Row<const P: usize> {
    /// Prehashed key. Hash computation is outside this lab.
    pub key: u64,
    /// Unique source identity, used to check permutation and stability.
    pub id: u64,
    /// Inline payload, copied together with key and identity.
    pub payload: [u64; P],
}

impl<const P: usize> Default for Row<P> {
    fn default() -> Self {
        Self {
            key: 0,
            id: 0,
            payload: [0; P],
        }
    }
}

/// Owned rows ordered by digit, and `buckets + 1` exclusive boundaries.
#[derive(Debug, Eq, PartialEq)]
pub struct Partition<const P: usize> {
    /// Partitioned rows.
    pub rows: Vec<Row<P>>,
    /// Bucket `b` occupies `bounds[b]..bounds[b + 1]`, including empty buckets.
    pub bounds: Vec<usize>,
}

/// A contiguous digit with bounded fanout. Zero bits denotes one bucket.
#[derive(Clone, Copy, Debug)]
pub struct Digit {
    shift: u32,
    bits: u32,
}

impl Digit {
    /// Reject more than 12 bits or shifts outside a 64-bit key.
    ///
    /// # Panics
    /// Panics if `bits > 12`, `shift >= 64`, or `shift + bits > 64`.
    #[must_use]
    pub fn new(shift: u32, bits: u32) -> Self {
        assert!(bits <= 12 && shift < 64 && shift + bits <= 64);
        Self { shift, bits }
    }
    /// Number of possible buckets.
    #[must_use]
    pub fn buckets(self) -> usize {
        1_usize << self.bits
    }
    /// Bucket of a key.
    ///
    /// # Panics
    /// The checked conversion cannot fail for the validated 12-bit digit.
    #[must_use]
    pub fn bucket(self, key: u64) -> usize {
        usize::try_from((key >> self.shift) & ((1_u64 << self.bits) - 1)).expect("bounded digit")
    }
}

/// Independent oracle: stable comparison sort; digit arithmetic is not shared.
#[must_use]
pub fn oracle<const P: usize>(input: &[Row<P>], digit: Digit) -> Partition<P> {
    let divisor = 2_u64.pow(digit.shift);
    let modulus = 2_u64.pow(digit.bits);
    let mut rows = input.to_vec();
    rows.sort_by_key(|r| (r.key / divisor) % modulus);
    let mut bounds = Vec::with_capacity(digit.buckets() + 1);
    let mut end = 0;
    for b in 0..digit.buckets() {
        bounds.push(end);
        while end < rows.len() && (rows[end].key / divisor) % modulus == b as u64 {
            end += 1;
        }
    }
    bounds.push(end);
    Partition { rows, bounds }
}

/// Simple stable baseline: append to growable bucket vectors, then flatten.
#[must_use]
pub fn buckets<const P: usize>(input: &[Row<P>], digit: Digit) -> Partition<P> {
    let mut bins = vec![Vec::new(); digit.buckets()];
    for &row in input {
        bins[digit.bucket(row.key)].push(row);
    }
    let mut rows = Vec::with_capacity(input.len());
    let mut bounds = vec![0];
    for bin in bins {
        rows.extend(bin);
        bounds.push(rows.len());
    }
    Partition { rows, bounds }
}

fn boundaries<const P: usize>(input: &[Row<P>], digit: Digit) -> Vec<usize> {
    let mut counts = vec![0; digit.buckets() + 1];
    for row in input {
        counts[digit.bucket(row.key) + 1] += 1;
    }
    for b in 1..counts.len() {
        counts[b] += counts[b - 1];
    }
    counts
}

/// Stable histogram, exclusive prefix sum, and direct scatter.
/// Output initialization, histograms, and cursor allocation are included.
#[must_use]
pub fn scatter<const P: usize>(input: &[Row<P>], digit: Digit) -> Partition<P> {
    let bounds = boundaries(input, digit);
    let mut next = bounds[..digit.buckets()].to_vec();
    let mut rows = vec![Row::default(); input.len()];
    for &row in input {
        let b = digit.bucket(row.key);
        rows[next[b]] = row;
        next[b] += 1;
    }
    debug_assert_eq!(next, bounds[1..]);
    Partition { rows, bounds }
}

/// Stable two-pass partition: high digit first, then low digit inside each group.
/// This recursive order gives ascending full-digit buckets without a transpose.
#[must_use]
pub fn two_pass<const P: usize>(input: &[Row<P>], digit: Digit) -> Partition<P> {
    if digit.bits <= 1 {
        return scatter(input, digit);
    }
    let low_bits = digit.bits / 2;
    let coarse = scatter(
        input,
        Digit::new(digit.shift + low_bits, digit.bits - low_bits),
    );
    let mut rows = Vec::with_capacity(input.len());
    let mut bounds = vec![0];
    for pair in coarse.bounds.windows(2) {
        let fine = scatter(
            &coarse.rows[pair[0]..pair[1]],
            Digit::new(digit.shift, low_bits),
        );
        let base = rows.len();
        bounds.extend(fine.bounds[1..].iter().map(|x| base + x));
        rows.extend(fine.rows);
    }
    Partition { rows, bounds }
}

/// Unstable in-place cycle placement with O(bucket count) metadata.
///
/// # Panics
/// Panics only if allocation fails or the stated digit contract is violated.
pub fn cycles_in_place<const P: usize>(rows: &mut [Row<P>], digit: Digit) -> Vec<usize> {
    let bounds = boundaries(rows, digit);
    let mut next = bounds[..digit.buckets()].to_vec();
    for b in 0..digit.buckets() {
        while next[b] < bounds[b + 1] {
            let target = digit.bucket(rows[next[b]].key);
            if target == b {
                next[b] += 1;
            } else {
                debug_assert!(next[target] < bounds[target + 1]);
                rows.swap(next[b], next[target]);
                next[target] += 1;
            }
        }
    }
    bounds
}

/// Immutable-input adapter for cycles. The source clone belongs to this API's cost.
#[must_use]
pub fn cycles<const P: usize>(input: &[Row<P>], digit: Digit) -> Partition<P> {
    let mut rows = input.to_vec();
    let bounds = cycles_in_place(&mut rows, digit);
    Partition { rows, bounds }
}

/// Run a named candidate. Stable candidates are `buckets`, `scatter`, `two_pass`.
///
/// # Panics
/// Panics for an unknown candidate name.
#[must_use]
pub fn run<const P: usize>(name: &str, input: &[Row<P>], digit: Digit) -> Partition<P> {
    match name {
        "buckets" => buckets(input, digit),
        "scatter" => scatter(input, digit),
        "two_pass" => two_pass(input, digit),
        "cycles" => cycles(input, digit),
        _ => panic!("unknown candidate"),
    }
}

/// Check exact bucket boundaries, payload-preserving permutation, and optional stability.
///
/// # Panics
/// Panics on any mismatch with the independent oracle.
pub fn check<const P: usize>(input: &[Row<P>], got: &Partition<P>, digit: Digit, stable: bool) {
    let expected = oracle(input, digit);
    assert_eq!(got.bounds, expected.bounds);
    let mut canonical = got.rows.clone();
    canonical.sort_by_key(|r| {
        (
            (r.key / 2_u64.pow(digit.shift)) % 2_u64.pow(digit.bits),
            r.id,
        )
    });
    let mut reference = expected.rows.clone();
    reference.sort_by_key(|r| {
        (
            (r.key / 2_u64.pow(digit.shift)) % 2_u64.pow(digit.bits),
            r.id,
        )
    });
    assert_eq!(canonical, reference);
    for b in 0..digit.buckets() {
        assert!(
            got.rows[got.bounds[b]..got.bounds[b + 1]]
                .iter()
                .all(|r| digit.bucket(r.key) == b)
        );
    }
    if stable {
        assert_eq!(got.rows, expected.rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn verify(keys: &[u64], shift: u32, bits: u32) {
        let input: Vec<_> = keys
            .iter()
            .enumerate()
            .map(|(i, &key)| Row {
                key,
                id: i as u64,
                payload: [key ^ 17; 6],
            })
            .collect();
        let digit = Digit::new(shift, bits);
        for name in ["buckets", "scatter", "two_pass", "cycles"] {
            check(&input, &run(name, &input, digit), digit, name != "cycles");
        }
    }
    #[test]
    fn running_example() {
        verify(&[6, 1, 7, 4, 1, 2, 5, 0], 0, 2);
    }
    #[test]
    fn empty_single_and_empty_buckets() {
        verify(&[], 0, 12);
        verify(&[u64::MAX], 52, 12);
    }
    #[test]
    fn zero_bits_high_shift_and_payload() {
        verify(&[u64::MAX, 0, 1, 1 << 63, 17], 63, 1);
        verify(&[u64::MAX, 0, 7], 63, 0);
    }
    #[test]
    fn skew_and_structured() {
        verify(&[5; 513], 0, 10);
        verify(&(0..512).map(|x| x << 12).collect::<Vec<_>>(), 0, 12);
    }
    #[test]
    fn exhaustive_small_digit() {
        for len in 0..=7_u32 {
            for code in 0..4_u64.pow(len) {
                let keys: Vec<_> = (0..len).map(|i| (code / 4_u64.pow(i)) % 4).collect();
                verify(&keys, 0, 2);
            }
        }
    }
    #[test]
    fn seeded_shapes() {
        let mut seed = 123_u64;
        for case in 0..128 {
            let len = case * 17;
            let keys: Vec<_> = (0..len)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed
                })
                .collect();
            verify(&keys, case % 53, case % 12);
        }
    }
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn reject_too_wide() {
        let _ = Digit::new(0, 13);
    }
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn reject_shift() {
        let _ = Digit::new(64, 0);
    }
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn reject_crossing_end() {
        let _ = Digit::new(63, 2);
    }
}
