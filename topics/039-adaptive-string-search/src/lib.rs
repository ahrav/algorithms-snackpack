//! First-occurrence exact byte search with five separately prepared candidates.
//!
//! Empty patterns match at zero. Returned offsets index bytes in the original
//! resident text. Search performs no allocation. Pattern tables borrow the pattern.
//!
//! ```
//! use topic_039_adaptive_string_search::{Candidate, Finder};
//! let finder = Finder::new(b"ABCD", Candidate::TwoWay);
//! assert_eq!(finder.find(b"ABAAABCD"), Some(4));
//! ```

/// The measured search implementations.
#[derive(Clone, Copy, Debug)]
pub enum Candidate {
    /// Left-to-right verification at every start.
    Direct,
    /// Right-to-left verification and last-aligned-byte shifts.
    Horspool,
    /// Bad-character and strong good-suffix shifts.
    BoyerMoore,
    /// Critical factorization with periodic comparison memory.
    TwoWay,
    /// Short direct search, otherwise bounded Horspool followed by Two-Way.
    Adaptive,
}

/// Diagnostic scan work, excluding pattern preparation.
#[derive(Default, Debug, Clone, Copy)]
pub struct Stats {
    /// Pattern versus text byte comparisons.
    pub comparisons: usize,
    /// Visited windows, including a repeated window at fallback.
    pub windows: usize,
    /// Permanent transitions from Horspool to Two-Way.
    pub fallbacks: usize,
}

enum Plan {
    Direct,
    Horspool(Box<[usize; 256]>),
    BoyerMoore {
        last: Box<[usize; 256]>,
        good: Vec<usize>,
    },
    TwoWay {
        cut: usize,
        period: usize,
        periodic: bool,
    },
    Adaptive(Box<[usize; 256]>),
}

/// A prepared search borrowing its pattern. Preparation may allocate tables.
pub struct Finder<'a> {
    pattern: &'a [u8],
    plan: Plan,
}

impl<'a> Finder<'a> {
    /// Prepares the chosen candidate for repeated searches.
    #[must_use]
    pub fn new(pattern: &'a [u8], candidate: Candidate) -> Self {
        let m = pattern.len();
        let plan = if m == 0 {
            Plan::Direct
        } else {
            match candidate {
                Candidate::Direct => Plan::Direct,
                Candidate::Horspool => Plan::Horspool(skip_table(pattern)),
                Candidate::BoyerMoore => {
                    let mut last = Box::new([0; 256]);
                    for (i, &b) in pattern.iter().enumerate() {
                        last[usize::from(b)] = i + 1;
                    }
                    Plan::BoyerMoore {
                        last,
                        good: good_suffix(pattern),
                    }
                }
                Candidate::TwoWay => {
                    let (cut, period, periodic) = factorize(pattern);
                    Plan::TwoWay {
                        cut,
                        period,
                        periodic,
                    }
                }
                Candidate::Adaptive if m < 4 => Plan::Direct,
                Candidate::Adaptive => Plan::Adaptive(skip_table(pattern)),
            }
        };
        Self { pattern, plan }
    }

    /// Returns the earliest matching byte start, including zero for an empty pattern.
    #[must_use]
    #[inline(never)]
    pub fn find(&self, text: &[u8]) -> Option<usize> {
        self.search::<false>(text, &mut Stats::default())
    }

    /// Runs the same search with byte-comparison and window counters enabled.
    #[must_use]
    pub fn diagnose(&self, text: &[u8]) -> (Option<usize>, Stats) {
        let mut stats = Stats::default();
        let answer = self.search::<true>(text, &mut stats);
        (answer, stats)
    }

    fn search<const COUNT: bool>(&self, text: &[u8], stats: &mut Stats) -> Option<usize> {
        let p = self.pattern;
        if p.is_empty() {
            return Some(0);
        }
        if p.len() > text.len() {
            return None;
        }
        match &self.plan {
            Plan::Direct => direct::<COUNT>(text, p, stats),
            Plan::Horspool(table) => horspool::<COUNT, false>(text, p, table, stats),
            Plan::BoyerMoore { last, good } => boyer_moore::<COUNT>(text, p, last, good, stats),
            Plan::TwoWay {
                cut,
                period,
                periodic,
            } => two_way::<COUNT>(text, p, *cut, *period, *periodic, stats),
            Plan::Adaptive(_) if text.len() < 64 => direct::<COUNT>(text, p, stats),
            Plan::Adaptive(table) => horspool::<COUNT, true>(text, p, table, stats),
        }
    }
}

/// Independent standard-library window oracle for the first byte match.
#[must_use]
pub fn oracle(text: &[u8], pattern: &[u8]) -> Option<usize> {
    if pattern.is_empty() {
        return Some(0);
    }
    text.windows(pattern.len())
        .position(|window| window == pattern)
}

fn equal<const COUNT: bool>(a: u8, b: u8, stats: &mut Stats) -> bool {
    if COUNT {
        stats.comparisons += 1;
    }
    a == b
}

fn direct<const COUNT: bool>(t: &[u8], p: &[u8], s: &mut Stats) -> Option<usize> {
    for start in 0..=t.len() - p.len() {
        if COUNT {
            s.windows += 1;
        }
        let mut j = 0;
        while j < p.len() && equal::<COUNT>(p[j], t[start + j], s) {
            j += 1;
        }
        if j == p.len() {
            return Some(start);
        }
    }
    None
}

fn skip_table(p: &[u8]) -> Box<[usize; 256]> {
    let mut table = Box::new([p.len(); 256]);
    for (i, &b) in p[..p.len() - 1].iter().enumerate() {
        table[usize::from(b)] = p.len() - 1 - i;
    }
    table
}

fn horspool<const COUNT: bool, const ADAPT: bool>(
    t: &[u8],
    p: &[u8],
    table: &[usize; 256],
    s: &mut Stats,
) -> Option<usize> {
    let mut start = 0;
    let mut work = 0;
    let budget = t.len().saturating_mul(2);
    while start <= t.len() - p.len() {
        if COUNT {
            s.windows += 1;
        }
        let mut j = p.len();
        while j > 0 {
            if ADAPT && work == budget {
                if COUNT {
                    s.fallbacks += 1;
                }
                let (cut, period, periodic) = factorize(p);
                // Every earlier start was resolved by a safe Horspool shift.
                return two_way::<COUNT>(&t[start..], p, cut, period, periodic, s)
                    .map(|i| start + i);
            }
            work += usize::from(ADAPT);
            if !equal::<COUNT>(p[j - 1], t[start + j - 1], s) {
                break;
            }
            j -= 1;
        }
        if j == 0 {
            return Some(start);
        }
        start += table[usize::from(t[start + p.len() - 1])];
    }
    None
}

fn good_suffix(p: &[u8]) -> Vec<usize> {
    let m = p.len();
    let mut shift = vec![0; m + 1];
    let mut border = vec![0; m + 1];
    let mut i = m;
    let mut j = m + 1;
    border[i] = j;
    // Border links describe matching suffixes. Record the first safe strong shift.
    while i > 0 {
        while j <= m && p[i - 1] != p[j - 1] {
            if shift[j] == 0 {
                shift[j] = j - i;
            }
            j = border[j];
        }
        i -= 1;
        j -= 1;
        border[i] = j;
    }
    j = border[0];
    for (i, value) in shift.iter_mut().enumerate() {
        if *value == 0 {
            *value = j;
        }
        if i == j {
            j = border[j];
        }
    }
    shift
}

fn boyer_moore<const COUNT: bool>(
    t: &[u8],
    p: &[u8],
    last: &[usize; 256],
    good: &[usize],
    s: &mut Stats,
) -> Option<usize> {
    let mut start = 0;
    while start <= t.len() - p.len() {
        if COUNT {
            s.windows += 1;
        }
        let mut j = p.len();
        while j > 0 && equal::<COUNT>(p[j - 1], t[start + j - 1], s) {
            j -= 1;
        }
        if j == 0 {
            return Some(start);
        }
        let bad = j.saturating_sub(last[usize::from(t[start + j - 1])]).max(1);
        start += bad.max(good[j]);
    }
    None
}

// Maximal suffix start and its period, using one of two alphabet orders.
fn maximal_suffix(p: &[u8], reverse: bool) -> (usize, usize) {
    let (mut best, mut candidate, mut offset, mut period) = (0, 1, 0, 1);
    while candidate + offset < p.len() {
        let a = p[candidate + offset];
        let b = p[best + offset];
        if if reverse { b < a } else { a < b } {
            candidate += offset + 1;
            offset = 0;
            period = candidate - best;
        } else if a == b {
            if offset + 1 == period {
                candidate += period;
                offset = 0;
            } else {
                offset += 1;
            }
        } else {
            best = candidate;
            candidate += 1;
            offset = 0;
            period = 1;
        }
    }
    (best, period)
}

fn factorize(p: &[u8]) -> (usize, usize, bool) {
    let a = maximal_suffix(p, false);
    let b = maximal_suffix(p, true);
    let (cut, period) = if a.0 > b.0 { a } else { b };
    let periodic = period + cut <= p.len() && p[..cut] == p[period..period + cut];
    (
        cut,
        if periodic {
            period
        } else {
            cut.max(p.len() - cut) + 1
        },
        periodic,
    )
}

fn two_way<const COUNT: bool>(
    t: &[u8],
    p: &[u8],
    cut: usize,
    period: usize,
    periodic: bool,
    s: &mut Stats,
) -> Option<usize> {
    let mut start = 0;
    let mut memory = 0;
    while start <= t.len() - p.len() {
        if COUNT {
            s.windows += 1;
        }
        let mut j = cut.max(memory);
        while j < p.len() && equal::<COUNT>(p[j], t[start + j], s) {
            j += 1;
        }
        if j < p.len() {
            start += j - cut + 1;
            memory = 0;
        } else {
            j = cut;
            while j > memory && equal::<COUNT>(p[j - 1], t[start + j - 1], s) {
                j -= 1;
            }
            if j <= memory {
                return Some(start);
            }
            start += period;
            memory = if periodic { p.len() - period } else { 0 };
        }
    }
    None
}
