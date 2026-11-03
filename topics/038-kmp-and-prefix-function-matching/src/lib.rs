//! Exact overlapping byte matching with explicit preparation and stream state.
//! Empty patterns are rejected. Offsets count bytes. Inputs may contain any byte.
//!
//! ```
//! use topic038_kmp_and_prefix_function_matching::Kmp;
//! let matcher = Kmp::new(b"ababaca").unwrap();
//! assert_eq!(matcher.find_all(b"abababacababaca"), vec![2, 8]);
//! ```

/// Independent resident oracle using slice equality at every legal start.
#[must_use]
pub fn oracle(text: &[u8], pattern: &[u8]) -> Option<Vec<usize>> {
    if pattern.is_empty() {
        return None;
    }
    Some(
        text.windows(pattern.len())
            .enumerate()
            .filter_map(|(i, w)| (w == pattern).then_some(i))
            .collect(),
    )
}

/// Direct left-to-right candidate with every overlapping byte offset.
#[must_use]
pub fn direct(text: &[u8], pattern: &[u8]) -> Option<Vec<usize>> {
    direct_impl::<false>(text, pattern).map(|r| r.0)
}

/// Byte-comparison count from a separate instrumented direct traversal.
#[must_use]
pub fn direct_comparisons(text: &[u8], pattern: &[u8]) -> Option<usize> {
    direct_impl::<true>(text, pattern).map(|r| r.1)
}

fn direct_impl<const COUNT: bool>(text: &[u8], pattern: &[u8]) -> Option<(Vec<usize>, usize)> {
    if pattern.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    let mut comparisons = 0;
    if pattern.len() > text.len() {
        return Some((out, comparisons));
    }
    for start in 0..=text.len() - pattern.len() {
        let mut matched = true;
        for (j, &b) in pattern.iter().enumerate() {
            if COUNT {
                comparisons += 1;
            }
            if text[start + j] != b {
                matched = false;
                break;
            }
        }
        if matched {
            out.push(start);
        }
    }
    Some((out, comparisons))
}

/// Longest proper prefix that is also a suffix of each pattern prefix.
#[must_use]
pub fn prefix_function(pattern: &[u8]) -> Vec<usize> {
    let mut pi = vec![0; pattern.len()];
    for i in 1..pattern.len() {
        let mut j = pi[i - 1];
        while j > 0 && pattern[i] != pattern[j] {
            j = pi[j - 1];
        }
        if pattern[i] == pattern[j] {
            j += 1;
        }
        pi[i] = j;
    }
    pi
}

/// An owned pattern and its reusable linear-space failure table.
pub struct Kmp {
    pattern: Vec<u8>,
    pi: Vec<usize>,
}
impl Kmp {
    /// Reject an empty pattern; prepare every nonempty byte pattern.
    #[must_use]
    pub fn new(pattern: &[u8]) -> Option<Self> {
        (!pattern.is_empty()).then(|| Self {
            pattern: pattern.to_vec(),
            pi: prefix_function(pattern),
        })
    }
    fn scan(&self, text: &[u8], q: &mut usize, base: usize) -> Vec<usize> {
        let mut out = Vec::new();
        for (i, &b) in text.iter().enumerate() {
            while *q > 0 && self.pattern[*q] != b {
                *q = self.pi[*q - 1];
            }
            if self.pattern[*q] == b {
                *q += 1;
            }
            if *q == self.pattern.len() {
                out.push(base + i + 1 - self.pattern.len());
                *q = self.pi[*q - 1];
            }
        }
        out
    }
    /// Return every overlapping match in increasing byte-offset order.
    #[must_use]
    pub fn find_all(&self, text: &[u8]) -> Vec<usize> {
        self.scan(text, &mut 0, 0)
    }
    /// Start an independent stream sharing this prepared pattern.
    #[must_use]
    pub fn stream(&self) -> Stream<'_> {
        Stream {
            matcher: self,
            q: 0,
            consumed: 0,
        }
    }
    /// Count search byte comparisons in a separate diagnostic traversal.
    #[must_use]
    pub fn comparisons(&self, text: &[u8]) -> usize {
        let mut q = 0;
        let mut count = 0;
        for &b in text {
            while q > 0 {
                count += 1;
                if self.pattern[q] == b {
                    break;
                }
                q = self.pi[q - 1];
            }
            count += 1;
            if self.pattern[q] == b {
                q += 1;
            }
            if q == self.pattern.len() {
                q = self.pi[q - 1];
            }
        }
        count
    }
}

/// Matched-prefix length and absolute offset retained across chunks.
pub struct Stream<'a> {
    matcher: &'a Kmp,
    q: usize,
    consumed: usize,
}
impl Stream<'_> {
    /// Consume a chunk and return new absolute match starts.
    /// Return `None` before changing state if the absolute end overflows `usize`.
    /// Empty chunks return an empty vector. Ending the stream needs no flush.
    #[must_use]
    pub fn push(&mut self, chunk: &[u8]) -> Option<Vec<usize>> {
        let end = self.consumed.checked_add(chunk.len())?;
        let out = self.matcher.scan(chunk, &mut self.q, self.consumed);
        self.consumed = end;
        Some(out)
    }
}

/// Dense deterministic finite automaton with 256 transitions per state.
pub struct Dfa {
    rows: Vec<[usize; 256]>,
    len: usize,
}
impl Dfa {
    /// Reject an empty pattern. Store the accepting row to preserve overlaps.
    #[must_use]
    pub fn new(pattern: &[u8]) -> Option<Self> {
        if pattern.is_empty() {
            return None;
        }
        let pi = prefix_function(pattern);
        let mut rows = vec![[0; 256]; pattern.len() + 1];
        rows[0][usize::from(pattern[0])] = 1;
        for q in 1..=pattern.len() {
            rows[q] = rows[pi[q - 1]];
            if q < pattern.len() {
                rows[q][usize::from(pattern[q])] = q + 1;
            }
        }
        Some(Self {
            rows,
            len: pattern.len(),
        })
    }
    /// Return every overlapping match with one table transition per text byte.
    #[must_use]
    pub fn find_all(&self, text: &[u8]) -> Vec<usize> {
        let mut q = 0;
        let mut out = Vec::new();
        for (i, &b) in text.iter().enumerate() {
            q = self.rows[q][usize::from(b)];
            if q == self.len {
                out.push(i + 1 - self.len);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overflow_is_atomic() {
        // A two-byte pattern leaves `q == 1` visible if the scan ran before the check.
        let k = Kmp::new(b"ab").unwrap();
        let mut s = k.stream();
        s.consumed = usize::MAX;
        assert_eq!(s.push(b"a"), None);
        assert_eq!((s.q, s.consumed), (0, usize::MAX));
        assert_eq!(s.push(b""), Some(vec![]));
    }
}
