//! Sampling index streams and retained weighted priorities.
//! Ideal distribution claims require independent uniform draws. `SplitMix64` is
//! a deterministic teaching generator, not a cryptographic or independence proof.
#![forbid(unsafe_code)]
// Count-to-float conversions are bounded by the u32 domain; word conversion is 52-bit.
#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
// Single-letter symbols follow the probability equations in the teaching selector.
#![allow(clippy::many_single_char_names)]
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Frozen `SplitMix64` teaching generator.
#[derive(Clone)]
pub struct Rng(u64);
impl Rng {
    /// Initialize an exact 64-bit state.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }
    /// Produce a word with wrapping arithmetic.
    pub fn word(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// Unbiased range reduction conditional on ideal uniform source words.
    /// # Panics
    /// Panics for a zero bound.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0);
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let x = self.word();
            if x >= threshold {
                return x % bound;
            }
        }
    }
    /// Open unit interval on an exactly representable 52-bit midpoint grid.
    pub fn open(&mut self) -> f64 {
        open_word(self.word())
    }
}
/// Map high bits to (0,1), including both extreme words safely.
#[must_use]
pub fn open_word(word: u64) -> f64 {
    ((word >> 12) as f64 + 0.5) * (1.0 / 4_503_599_627_370_496.0)
}
/// Sampling implementation; X uses finite-precision sequential skip inversion.
#[derive(Clone, Copy, Debug)]
pub enum Uniform {
    /// Algorithm R with rejection range reduction.
    R,
    /// Bounded Algorithm X; source exhaustion caps even k=1 skips.
    X,
    /// Retain smallest word/ID priorities by full sorting.
    PrioritySort,
    /// Retain smallest word/ID priorities in a bounded max heap.
    PriorityHeap,
}
/// Sample distinct indices, returning min(n,k) sorted IDs.
/// Initialization, RNG, materialization and output sorting belong to this call.
/// # Panics
/// Panics when n exceeds `u32::MAX`; X counts then remain exact in `f64`.
#[must_use]
pub fn uniform(n: usize, k: usize, seed: u64, method: Uniform) -> Vec<usize> {
    assert!(u32::try_from(n).is_ok());
    let k = k.min(n);
    if k == 0 {
        return Vec::new();
    }
    let mut rng = Rng::new(seed);
    let mut out = match method {
        Uniform::R => {
            let mut v: Vec<_> = (0..k).collect();
            for i in k..n {
                let j = rng.below((i + 1) as u64) as usize;
                if j < k {
                    v[j] = i;
                }
            }
            v
        }
        Uniform::X => {
            let mut v: Vec<_> = (0..k).collect();
            let mut t = k;
            while t < n {
                let u = rng.open();
                let mut survival = 1.0;
                loop {
                    t += 1;
                    survival *= (t - k) as f64 / t as f64;
                    if survival <= u || t == n {
                        break;
                    }
                }
                if survival <= u {
                    v[rng.below(k as u64) as usize] = t - 1;
                }
            }
            v
        }
        Uniform::PrioritySort => {
            let mut keys: Vec<_> = (0..n).map(|i| (rng.word(), i)).collect();
            keys.sort_unstable();
            keys.truncate(k);
            keys.into_iter().map(|(_, i)| i).collect()
        }
        Uniform::PriorityHeap => {
            let mut heap = BinaryHeap::with_capacity(k);
            for i in 0..n {
                let key = (rng.word(), i);
                if heap.len() < k {
                    heap.push(key);
                } else if key < *heap.peek().expect("nonempty") {
                    *heap.peek_mut().expect("nonempty") = key;
                }
            }
            heap.into_iter().map(|(_, i)| i).collect()
        }
    };
    out.sort_unstable();
    out
}
/// Retained log exponential priority. Smaller scores win; ID breaks ties.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    /// Stable distinct item ID.
    pub id: usize,
    /// Finite log-clock score.
    pub score: f64,
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .total_cmp(&other.score)
            .then(self.id.cmp(&other.id))
    }
}
/// Validate weights and generate retained keys in input-ID order.
/// Rejects zero, negative, infinite and NaN weights, including for k=0 callers.
/// Log clocks avoid direct division overflow for positive finite weights.
#[must_use]
pub fn weighted_keys(weights: &[f64], seed: u64) -> Option<Vec<Key>> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(weights.len());
    for (id, &w) in weights.iter().enumerate() {
        if !(w > 0.0 && w.is_finite()) {
            return None;
        }
        out.push(Key {
            id,
            score: (-rng.open().ln()).ln() - w.ln(),
        });
    }
    Some(out)
}
/// Full-sort weighted baseline, including keys, allocation and ID sorting.
#[must_use]
pub fn weighted_sort(weights: &[f64], k: usize, seed: u64) -> Option<Vec<Key>> {
    let mut keys = weighted_keys(weights, seed)?;
    keys.sort_unstable();
    keys.truncate(k.min(weights.len()));
    keys.sort_unstable_by_key(|x| x.id);
    Some(keys)
}
/// Streaming bounded heap; validates every weight and keeps only k keys.
#[must_use]
pub fn weighted_heap(weights: &[f64], k: usize, seed: u64) -> Option<Vec<Key>> {
    let mut rng = Rng::new(seed);
    let k = k.min(weights.len());
    let mut heap = BinaryHeap::with_capacity(k);
    for (id, &w) in weights.iter().enumerate() {
        if !(w > 0.0 && w.is_finite()) {
            return None;
        }
        let key = Key {
            id,
            score: (-rng.open().ln()).ln() - w.ln(),
        };
        if k == 0 {
            continue;
        }
        if heap.len() < k {
            heap.push(key);
        } else if let Some(mut root) = heap.peek_mut()
            && key < *root
        {
            *root = key;
        }
    }
    let mut out = heap.into_vec();
    out.sort_unstable_by_key(|x| x.id);
    Some(out)
}
/// Keep global k smallest retained keys. Disjoint unique IDs and finite scores required.
/// Each shard must retain at least global k keys (or all its items).
/// Caller owns these preconditions; never regenerate keys during combination.
#[must_use]
pub fn combine(shards: &[Vec<Key>], k: usize) -> Vec<Key> {
    let mut heap = BinaryHeap::with_capacity(k);
    if k == 0 {
        return Vec::new();
    }
    for &key in shards.iter().flatten() {
        if heap.len() < k {
            heap.push(key);
        } else if let Some(mut root) = heap.peek_mut()
            && key < *root
        {
            *root = key;
        }
    }
    let mut out = heap.into_vec();
    out.sort_unstable_by_key(|x| x.id);
    out
}
/// Independent quadratic retained-key oracle, no priority sort or heap.
#[must_use]
pub fn oracle(keys: &[Key], k: usize) -> Vec<Key> {
    let mut remaining = keys.to_vec();
    let mut out = Vec::new();
    for _ in 0..k.min(keys.len()) {
        let mut best = 0;
        for i in 1..remaining.len() {
            // Independent monotone IEEE encoding, including signed zeros.
            let rank = |score: f64| {
                let bits = score.to_bits();
                if bits >> 63 == 1 {
                    !bits
                } else {
                    bits | (1_u64 << 63)
                }
            };
            if (rank(remaining[i].score), remaining[i].id)
                < (rank(remaining[best].score), remaining[best].id)
            {
                best = i;
            }
        }
        out.push(remaining.swap_remove(best));
    }
    out.sort_unstable_by_key(|x| x.id);
    out
}
