//! Educational immutable membership filters and a small binary-equation witness.
//! The filters can admit false positives. Check positives against exact keys.
#![allow(clippy::cast_possible_truncation)] // Low bits and modulo-reduced indices are intentional.
use std::collections::VecDeque;

/// Deterministic noncryptographic mixer for trusted laboratory keys.
#[must_use]
pub fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Insert-only Bloom filter, either global or restricted to one 64-bit word.
#[derive(Clone, Debug)]
pub struct Bloom {
    words: Vec<u64>,
    probes: usize,
    blocked: bool,
}
impl Bloom {
    /// Allocate at least one word. `probes` must be in 1..=16.
    ///
    /// # Panics
    /// Panics on invalid probe count or zero requested bits.
    #[must_use]
    pub fn new(bits: usize, probes: usize, blocked: bool) -> Self {
        assert!(bits > 0 && (1..=16).contains(&probes));
        Self {
            words: vec![0; bits.div_ceil(64)],
            probes,
            blocked,
        }
    }
    fn locations(&self, key: u64) -> impl Iterator<Item = (usize, u64)> + '_ {
        let h = mix(key);
        let step = mix(key ^ 0x1234_5678_90ab_cdef) | 1;
        (0..self.probes).map(move |j| {
            let v = h.wrapping_add((j as u64).wrapping_mul(step));
            if self.blocked {
                (
                    (mix(key ^ 0xfeed) % self.words.len() as u64) as usize,
                    1 << (v & 63),
                )
            } else {
                let bit = (v % (self.words.len() as u64 * 64)) as usize;
                (bit / 64, 1 << (bit % 64))
            }
        })
    }
    /// Add evidence. Repeated insertion is idempotent. There is no delete API.
    pub fn insert(&mut self, key: u64) {
        let locations: Vec<_> = self.locations(key).collect();
        for (word, mask) in locations {
            self.words[word] |= mask;
        }
    }
    /// Return false only when a required bit is absent.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        self.locations(key)
            .all(|(word, mask)| self.words[word] & mask != 0)
    }
    /// Resident word payload, excluding vector header and allocator metadata.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.words.len() * 8
    }
}

/// Four-slot, two-choice cuckoo fingerprint filter with transactional admission.
/// Zero is empty. Tags are uniform-model 12-bit nonzero values, stored in u16.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cuckoo {
    buckets: Vec<[u16; 4]>,
}
impl Cuckoo {
    /// Allocate a power-of-two bucket count.
    ///
    /// # Panics
    /// Panics unless `buckets` is a positive power of two.
    #[must_use]
    pub fn new(buckets: usize) -> Self {
        assert!(buckets.is_power_of_two());
        Self {
            buckets: vec![[0; 4]; buckets],
        }
    }
    fn tag(key: u64) -> u16 {
        (mix(key ^ 0xcafe) % 4095 + 1) as u16
    }
    fn home(&self, key: u64) -> usize {
        mix(key) as usize & (self.buckets.len() - 1)
    }
    fn alternate(&self, bucket: usize, tag: u16) -> usize {
        bucket ^ (mix(u64::from(tag)) as usize & (self.buckets.len() - 1))
    }
    /// Admit one occurrence, or leave every bucket unchanged on failure.
    /// `budget` bounds visited buckets, including the two roots.
    pub fn insert(&mut self, key: u64, budget: usize) -> bool {
        let tag = Self::tag(key);
        let a = self.home(key);
        let b = self.alternate(a, tag);
        let mut nodes = Vec::<(usize, Option<(usize, usize)>)>::new();
        let mut queue = VecDeque::new();
        for bucket in [a, b] {
            if nodes.len() < budget && !nodes.iter().any(|x| x.0 == bucket) {
                nodes.push((bucket, None));
                queue.push_back(nodes.len() - 1);
            }
        }
        while let Some(node) = queue.pop_front() {
            let bucket = nodes[node].0;
            if let Some(mut slot) = self.buckets[bucket].iter().position(|x| *x == 0) {
                let mut cur = node;
                while let Some((parent, source_slot)) = nodes[cur].1 {
                    self.buckets[nodes[cur].0][slot] = self.buckets[nodes[parent].0][source_slot];
                    slot = source_slot;
                    cur = parent;
                }
                self.buckets[nodes[cur].0][slot] = tag;
                return true;
            }
            for slot in 0..4 {
                let next = self.alternate(bucket, self.buckets[bucket][slot]);
                if nodes.len() < budget && !nodes.iter().any(|x| x.0 == next) {
                    nodes.push((next, Some((node, slot))));
                    queue.push_back(nodes.len() - 1);
                }
            }
        }
        false
    }
    /// Query both legal buckets. A match is only approximate membership.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        let tag = Self::tag(key);
        let a = self.home(key);
        self.buckets[a].contains(&tag) || self.buckets[self.alternate(a, tag)].contains(&tag)
    }
    /// Remove one fingerprint occurrence.
    ///
    /// Caller must know from exact state that this key has an outstanding admitted
    /// occurrence. `contains(key)` cannot establish that precondition.
    pub fn remove_known(&mut self, key: u64) -> bool {
        let tag = Self::tag(key);
        let a = self.home(key);
        for bucket in [a, self.alternate(a, tag)] {
            if let Some(slot) = self.buckets[bucket].iter().position(|x| *x == tag) {
                self.buckets[bucket][slot] = 0;
                return true;
            }
        }
        false
    }
    /// Resident bucket payload, excluding transient search paths.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.buckets.len() * 8
    }
}

/// Static quotient/remainder runs with an explicit directory.
/// This demonstrates quotient organization, NOT compact in-place quotient metadata.
#[derive(Debug)]
pub struct QuotientRuns {
    offsets: Vec<usize>,
    remainders: Vec<u16>,
}
impl QuotientRuns {
    /// Build from keys with a power-of-two quotient count and 12-bit remainders.
    /// Duplicate fingerprints are normalized because this object is immutable.
    ///
    /// # Panics
    /// Panics unless `quotients` is a positive power of two.
    #[must_use]
    pub fn build(keys: &[u64], quotients: usize) -> Self {
        assert!(quotients.is_power_of_two());
        let mut pairs: Vec<_> = keys
            .iter()
            .map(|&key| Self::split(key, quotients))
            .collect();
        pairs.sort_unstable();
        pairs.dedup();
        let mut offsets = vec![0; quotients + 1];
        for &(q, _) in &pairs {
            offsets[q + 1] += 1;
        }
        for q in 1..=quotients {
            offsets[q] += offsets[q - 1];
        }
        Self {
            offsets,
            remainders: pairs.into_iter().map(|x| x.1).collect(),
        }
    }
    fn split(key: u64, quotients: usize) -> (usize, u16) {
        let h = mix(key);
        (((h >> 12) as usize) & (quotients - 1), (h & 4095) as u16)
    }
    /// Search only the remainder run belonging to this quotient.
    #[must_use]
    pub fn contains(&self, key: u64) -> bool {
        let (q, r) = Self::split(key, self.offsets.len() - 1);
        self.remainders[self.offsets[q]..self.offsets[q + 1]]
            .binary_search(&r)
            .is_ok()
    }
    /// Directory plus remainder payload. Includes directory overhead.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.offsets.len() * size_of::<usize>() + self.remainders.len() * 2
    }
}

/// Solve at most 64 binary variables with byte-valued right sides.
/// XOR is addition in the binary field. This is a small Ribbon mechanism witness,
/// not a scalable Ribbon implementation. Returns None on contradictory equations.
#[must_use]
pub fn solve_binary(rows: &[(u64, u8)]) -> Option<[u8; 64]> {
    let mut pivots = [(0_u64, 0_u8); 64];
    for &(mut mask, mut value) in rows {
        while mask != 0 {
            let p = mask.trailing_zeros() as usize;
            if pivots[p].0 == 0 {
                pivots[p] = (mask, value);
                break;
            }
            mask ^= pivots[p].0;
            value ^= pivots[p].1;
        }
        if mask == 0 && value != 0 {
            return None;
        }
    }
    let mut solution = [0; 64];
    for p in (0..64).rev() {
        let (mut mask, mut value) = pivots[p];
        if mask == 0 {
            continue;
        }
        mask &= !(1 << p);
        while mask != 0 {
            let j = mask.trailing_zeros() as usize;
            value ^= solution[j];
            mask &= mask - 1;
        }
        solution[p] = value;
    }
    Some(solution)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oracle_members_and_exact_rechecks() {
        use std::collections::BTreeSet;
        for n in [0, 1, 31, 256, 4096] {
            let keys: Vec<_> = (0..n).map(|x| mix(x + 9)).collect();
            let oracle: BTreeSet<_> = keys.iter().copied().collect();
            let mut full = Bloom::new((n as usize * 10).max(64), 7, false);
            let mut blocked = Bloom::new((n as usize * 10).max(64), 7, true);
            let mut cuckoo = Cuckoo::new((n as usize).div_ceil(2).max(1).next_power_of_two());
            for &key in &keys {
                full.insert(key);
                blocked.insert(key);
                assert!(cuckoo.insert(key, 128));
            }
            let quotient = QuotientRuns::build(&keys, (n as usize).max(1).next_power_of_two());
            for key in keys
                .iter()
                .copied()
                .chain((0..10000).map(|x| mix(x + 100_000)))
            {
                let truth = oracle.contains(&key);
                for maybe in [
                    full.contains(key),
                    blocked.contains(key),
                    cuckoo.contains(key),
                    quotient.contains(key),
                ] {
                    assert!(!truth || maybe, "false negative");
                    assert_eq!(maybe && oracle.contains(&key), truth);
                }
            }
        }
    }
    #[test]
    fn failed_admission_is_unchanged() {
        let mut filter = Cuckoo::new(1);
        for key in 0..4 {
            assert!(filter.insert(key, 1));
        }
        let before = filter.clone();
        assert!(!filter.insert(999, 128));
        assert_eq!(filter, before);
        assert!(!filter.insert(999, 0));
        assert_eq!(filter, before);
    }
    #[test]
    fn deletion_requires_exact_evidence_and_multiplicity() {
        let mut filter = Cuckoo::new(1);
        assert!(filter.insert(7, 1));
        assert!(filter.insert(7, 1));
        assert!(filter.remove_known(7));
        assert!(filter.contains(7));
        let alias = (8..100_000)
            .find(|&x| Cuckoo::tag(x) == Cuckoo::tag(7))
            .unwrap();
        assert!(filter.contains(alias));
        // Deliberately violates the documented precondition, reproducing data loss.
        assert!(filter.remove_known(alias));
        assert!(!filter.contains(7));
    }
    #[test]
    fn ribbon_equations_and_contradictions() {
        let rows = [(0b0011, 5), (0b0110, 9), (0b1100, 12)];
        let s = solve_binary(&rows).unwrap();
        for &(mask, value) in &rows {
            let actual = (0..64)
                .filter(|j| mask & (1 << j) != 0)
                .fold(0, |a, j| a ^ s[j]);
            assert_eq!(actual, value);
        }
        assert!(solve_binary(&[(3, 5), (3, 6)]).is_none());
        assert!(solve_binary(&[(0, 1)]).is_none());
        assert!(solve_binary(&[(0, 0)]).is_some());
    }
}
