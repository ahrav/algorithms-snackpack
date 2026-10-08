//! Sorting portfolios for fixed-width records, with explicit stability and group contracts.
use std::collections::BTreeMap;

/// A fixed-width key and an identity payload. Identity is not part of the sort key.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Record {
    /// Unsigned order key: leading 32-bit prefix, then 32-bit suffix.
    pub key: u64,
    /// Payload used to detect record loss and equal-key reordering.
    pub id: u32,
}
/// Candidate names, in frozen schedule order.
pub const METHODS: [&str; 8] = [
    "unstable", "stable", "quick", "heap", "merge", "radix", "dispatch", "groups",
];
/// Whether the named implementation promises equal-key stability.
#[must_use]
pub fn is_stable(method: &str) -> bool {
    matches!(method, "stable" | "merge" | "radix" | "dispatch" | "groups")
}
/// Sort by `key`. `groups` rejects a prefix-order violation before changing input.
///
/// # Errors
/// Returns the first descending prefix boundary for `groups`.
/// # Panics
/// Panics for an unknown candidate name.
pub fn sort(records: &mut [Record], method: &str) -> Result<(), usize> {
    match method {
        "unstable" => records.sort_unstable_by_key(|r| r.key),
        "stable" => records.sort_by_key(|r| r.key),
        "quick" => quick(records, 2 * records.len().ilog2_or_zero()),
        "heap" => heap(records),
        "merge" => natural_merge(records),
        "radix" => radix(records),
        "groups" => return groups(records),
        "dispatch" => {
            if records.len() <= 32 {
                insertion(records);
            } else if groups(records).is_err() {
                records.sort_by_key(|r| r.key);
            }
        }
        _ => panic!("unknown candidate"),
    }
    Ok(())
}
trait LogZero {
    fn ilog2_or_zero(self) -> u32;
}
impl LogZero for usize {
    fn ilog2_or_zero(self) -> u32 {
        self.checked_ilog2().unwrap_or(0)
    }
}
fn insertion(a: &mut [Record]) {
    for i in 1..a.len() {
        let x = a[i];
        let mut j = i;
        while j > 0 && a[j - 1].key > x.key {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = x;
    }
}
fn quick(a: &mut [Record], depth: u32) {
    if a.len() <= 16 {
        insertion(a);
        return;
    }
    if depth == 0 {
        heap(a);
        return;
    }
    let mut keys = [a[0].key, a[a.len() / 2].key, a[a.len() - 1].key];
    keys.sort_unstable();
    let pivot = keys[1];
    let (mut lo, mut scan, mut hi) = (0, 0, a.len());
    while scan < hi {
        match a[scan].key.cmp(&pivot) {
            std::cmp::Ordering::Less => {
                a.swap(lo, scan);
                lo += 1;
                scan += 1;
            }
            std::cmp::Ordering::Greater => {
                hi -= 1;
                a.swap(scan, hi);
            }
            std::cmp::Ordering::Equal => {
                scan += 1;
            }
        }
    }
    quick(&mut a[..lo], depth - 1);
    quick(&mut a[hi..], depth - 1);
}
fn sift(a: &mut [Record], mut root: usize, end: usize) {
    while root < end / 2 {
        let left = root * 2 + 1;
        let mut child = left;
        if left + 1 < end && a[left].key < a[left + 1].key {
            child += 1;
        }
        if a[root].key >= a[child].key {
            break;
        }
        a.swap(root, child);
        root = child;
    }
}
fn heap(a: &mut [Record]) {
    for i in (0..a.len() / 2).rev() {
        sift(a, i, a.len());
    }
    for end in (1..a.len()).rev() {
        a.swap(0, end);
        sift(a, 0, end);
    }
}
fn natural_merge(a: &mut [Record]) {
    if a.len() < 2 {
        return;
    }
    let mut runs = Vec::new();
    let mut start = 0;
    while start < a.len() {
        let mut end = start + 1;
        if end < a.len() && a[start].key > a[end].key {
            while end < a.len() && a[end - 1].key > a[end].key {
                end += 1;
            }
            a[start..end].reverse(); // Strict decrease excludes equal-key reordering.
        } else {
            while end < a.len() && a[end - 1].key <= a[end].key {
                end += 1;
            }
        }
        runs.push((start, end));
        start = end;
    }
    if runs.len() == 1 {
        return;
    }
    let mut scratch = a.to_vec();
    while runs.len() > 1 {
        let mut next = Vec::with_capacity(runs.len().div_ceil(2));
        for pair in runs.chunks(2) {
            let (lo, mid) = pair[0];
            if pair.len() == 1 {
                scratch[lo..mid].copy_from_slice(&a[lo..mid]);
                next.push((lo, mid));
                continue;
            }
            let hi = pair[1].1;
            let (mut i, mut j) = (lo, mid);
            for out in &mut scratch[lo..hi] {
                if i < mid && (j == hi || a[i].key <= a[j].key) {
                    *out = a[i];
                    i += 1;
                } else {
                    *out = a[j];
                    j += 1;
                }
            }
            next.push((lo, hi));
        }
        a.copy_from_slice(&scratch);
        runs = next;
    }
}
fn radix(a: &mut [Record]) {
    if a.len() < 2 {
        return;
    }
    let mut scratch = a.to_vec();
    for byte in 0..8 {
        let shift = byte * 8;
        let mut offsets = [0_usize; 256];
        for r in a.iter() {
            offsets[usize::from(r.key.to_le_bytes()[byte])] += 1;
        }
        let mut total = 0;
        for count in &mut offsets {
            let n = *count;
            *count = total;
            total += n;
        }
        for r in a.iter() {
            let bucket = usize::try_from((r.key >> shift) & 255).expect("one byte");
            scratch[offsets[bucket]] = *r;
            offsets[bucket] += 1;
        }
        a.copy_from_slice(&scratch);
    }
}
fn groups(a: &mut [Record]) -> Result<(), usize> {
    if let Some(i) = a.windows(2).position(|w| w[0].key >> 32 > w[1].key >> 32) {
        return Err(i + 1);
    }
    let mut start = 0;
    while start < a.len() {
        let prefix = a[start].key >> 32;
        let mut end = start + 1;
        while end < a.len() && a[end].key >> 32 == prefix {
            end += 1;
        }
        a[start..end].sort_by_key(|r| r.key);
        start = end;
    }
    Ok(())
}
/// Independent tree-bucket oracle. Equal keys retain arrival order.
#[must_use]
pub fn oracle(input: &[Record]) -> Vec<Record> {
    let mut buckets: BTreeMap<u64, Vec<Record>> = BTreeMap::new();
    for &r in input {
        buckets.entry(r.key).or_default().push(r);
    }
    buckets.into_values().flatten().collect()
}
/// Check order, complete-record multiplicity, and optional arrival-order stability.
#[must_use]
pub fn agrees(input: &[Record], output: &[Record], stable: bool) -> bool {
    let expected = oracle(input);
    if stable {
        return expected == output;
    }
    if output.len() != input.len() || output.windows(2).any(|w| w[0].key > w[1].key) {
        return false;
    }
    let count = |a: &[Record]| {
        let mut m = BTreeMap::new();
        for r in a {
            *m.entry((r.key, r.id)).or_insert(0_usize) += 1;
        }
        m
    };
    count(input) == count(output)
}
/// Frozen deterministic workloads. Seed changes values within each process block.
/// # Panics
/// Panics for an unknown shape or lengths beyond `u32::MAX`.
#[must_use]
pub fn workload(n: usize, shape: &str, mut seed: u64) -> Vec<Record> {
    let mut random = || {
        seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut x = seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^ (x >> 31)
    };
    let mut a: Vec<_> = (0..n)
        .map(|i| {
            let i64 = u64::try_from(i).expect("length");
            let key = match shape {
                "random" | "cold" => random(),
                "equal" => 7,
                "duplicates" => random() % 8,
                "sorted" | "nearly" => i64,
                "reverse" => u64::try_from(n - i).expect("length"),
                "organ" => u64::try_from(i.min(n - 1 - i)).expect("length"),
                "runs" => u64::try_from((15 - (i / 256) % 16) * 256 + i % 256).expect("length"),
                "groups" => {
                    (u64::try_from(i / 16).expect("prefix") << 32) | (random() & 0xffff_ffff)
                }
                "skew" => (u64::from(i >= n * 3 / 4) << 32) | (random() & 0xffff_ffff),
                _ => panic!("unknown shape"),
            };
            Record {
                key,
                id: u32::try_from(i).expect("length must fit u32"),
            }
        })
        .collect();
    if shape == "nearly" {
        for i in (0..n.saturating_sub(1)).step_by(128) {
            a.swap(i, i + 1);
        }
    }
    a
}
/// Count comparisons in separately instrumented standard-library sorts, outside timing.
#[must_use]
pub fn comparison_count(input: &[Record], stable: bool) -> usize {
    let mut a = input.to_vec();
    let mut n = 0;
    let mut cmp = |x: &Record, y: &Record| {
        n += 1;
        x.key.cmp(&y.key)
    };
    if stable {
        a.sort_by(&mut cmp);
    } else {
        a.sort_unstable_by(&mut cmp);
    }
    n
}
