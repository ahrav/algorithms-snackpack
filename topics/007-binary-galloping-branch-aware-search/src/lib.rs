//! Lower-bound searches over immutable, nondecreasing `u64` slices.
//!
//! All candidates return the first index with value at least the key, or the
//! slice length. Sorted input is a precondition; no candidate allocates.
#![forbid(unsafe_code)]

/// Independent oracle: count every value strictly below the key.
///
/// This deliberately does not stop early or share interval-update code.
#[must_use]
pub fn oracle(a: &[u64], key: u64) -> usize {
    a.iter().filter(|&&v| v < key).count()
}

/// Scan until the first value at least the key.
#[must_use]
#[inline(never)]
pub fn linear(a: &[u64], key: u64) -> usize {
    a.iter().position(|&v| v >= key).unwrap_or(a.len())
}

/// Halve the unresolved half-open interval; equality keeps the left side.
///
/// ```
/// use lower_bound_portfolio::binary;
/// let a = [2, 4, 4, 4, 9, 13, 18, 21];
/// assert_eq!(binary(&a, 4), 1);
/// assert_eq!(binary(&a, 22), 8);
/// ```
#[must_use]
#[inline(never)]
pub fn binary(a: &[u64], key: u64) -> usize {
    let (mut lo, mut hi) = (0, a.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if a[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Use a query-independent loop length and hint an unpredictable base update.
///
/// This is a source-level candidate, not a guarantee of branch-free assembly.
/// The last comparison distinguishes `base` from `base + 1`.
#[must_use]
#[inline(never)]
pub fn select(a: &[u64], key: u64) -> usize {
    if a.is_empty() {
        return 0;
    }
    let (mut base, mut size) = (0, a.len());
    while size > 1 {
        let half = size / 2;
        let mid = base + half;
        base = std::hint::select_unpredictable(a[mid] < key, mid, base);
        size -= half;
    }
    base + usize::from(a[base] < key)
}

/// Grow probes from the beginning, then binary-search the proven bracket.
///
/// Starting at zero avoids an unproven hint that could skip earlier duplicates.
/// Saturating doubling and clipping avoid wraparound and out-of-range reads.
#[must_use]
#[inline(never)]
pub fn gallop(a: &[u64], key: u64) -> usize {
    let (mut lo, mut probe) = (0, 1usize);
    while probe < a.len() && a[probe] < key {
        lo = probe + 1;
        probe = probe.saturating_mul(2);
    }
    let hi = probe.saturating_add(1).min(a.len());
    lo + binary(&a[lo..hi], key)
}

/// Standard-library lower bound with identical first-duplicate semantics.
#[must_use]
#[inline(never)]
pub fn standard(a: &[u64], key: u64) -> usize {
    a.partition_point(|&v| v < key)
}

/// A lower-bound candidate with the shared sorted-input contract.
pub type Search = fn(&[u64], u64) -> usize;

/// Five candidates named in the frozen experiment.
pub const CANDIDATES: [(&str, Search); 5] = [
    ("linear", linear),
    ("binary", binary),
    ("select", select),
    ("gallop", gallop),
    ("standard", standard),
];
