//! Exact, overlapping byte matching for a fixed set of nonempty patterns.
#![forbid(unsafe_code)]
use std::collections::VecDeque;

/// One match with half-open byte offsets and its original pattern ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Hit {
    /// Original pattern position, including distinct duplicate IDs.
    pub id: usize,
    /// First matched byte.
    pub start: usize,
    /// Byte immediately after the match.
    pub end: usize,
}

/// The index of a rejected empty pattern.
#[derive(Debug, Eq, PartialEq)]
pub struct EmptyPattern(pub usize);

fn validate(patterns: &[Vec<u8>]) -> Result<(), EmptyPattern> {
    patterns
        .iter()
        .position(Vec::is_empty)
        .map_or(Ok(()), |i| Err(EmptyPattern(i)))
}

/// Simple repeated comparisons. Emits every overlapping match.
///
/// # Errors
/// Returns the first empty pattern ID.
pub fn direct(
    patterns: &[Vec<u8>],
    text: &[u8],
    mut emit: impl FnMut(Hit),
) -> Result<(), EmptyPattern> {
    validate(patterns)?;
    for end in 1..=text.len() {
        for (id, p) in patterns.iter().enumerate() {
            if end >= p.len() && text[end - p.len()..end] == p[..] {
                emit(Hit {
                    id,
                    start: end - p.len(),
                    end,
                });
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct Node {
    edges: Vec<(u8, usize)>,
    fail: usize,
    dict: Option<usize>,
    own: Vec<usize>,
}

/// Sparse Aho-Corasick with linear edge lookup and shared output links.
///
/// ```
/// use topic043::Sparse;
/// let ac = Sparse::new(&[b"aa".to_vec()]).unwrap();
/// let mut starts = Vec::new();
/// ac.scan(b"aaa", |hit| starts.push(hit.start));
/// assert_eq!(starts, vec![0, 1]);
/// ```
pub struct Sparse {
    nodes: Vec<Node>,
    lengths: Vec<usize>,
}

impl Sparse {
    /// Build a fixed pattern generation. Duplicate IDs remain distinct.
    ///
    /// # Errors
    /// Returns the first empty pattern ID.
    pub fn new(patterns: &[Vec<u8>]) -> Result<Self, EmptyPattern> {
        validate(patterns)?;
        let mut a = Self {
            nodes: vec![Node::default()],
            lengths: patterns.iter().map(Vec::len).collect(),
        };
        for (id, p) in patterns.iter().enumerate() {
            let mut s = 0;
            for &b in p {
                s = if let Some(t) = a.edge(s, b) {
                    t
                } else {
                    let t = a.nodes.len();
                    a.nodes.push(Node::default());
                    a.nodes[s].edges.push((b, t));
                    t
                };
            }
            a.nodes[s].own.push(id);
        }
        let mut queue: VecDeque<usize> = a.nodes[0].edges.iter().map(|&(_, t)| t).collect();
        while let Some(s) = queue.pop_front() {
            for (b, t) in a.nodes[s].edges.clone() {
                let f = a.advance(a.nodes[s].fail, b);
                a.nodes[t].fail = f;
                a.nodes[t].dict = if a.nodes[f].own.is_empty() {
                    a.nodes[f].dict
                } else {
                    Some(f)
                };
                queue.push_back(t);
            }
        }
        Ok(a)
    }

    fn edge(&self, s: usize, b: u8) -> Option<usize> {
        self.nodes[s]
            .edges
            .iter()
            .find(|&&(c, _)| c == b)
            .map(|&(_, t)| t)
    }

    fn advance(&self, mut s: usize, b: u8) -> usize {
        loop {
            if let Some(t) = self.edge(s, b) {
                return t;
            }
            if s == 0 {
                return 0;
            }
            s = self.nodes[s].fail;
        }
    }

    fn outputs(&self, s: usize, end: usize, emit: &mut impl FnMut(Hit)) {
        let mut current = Some(s);
        while let Some(t) = current {
            for &id in &self.nodes[t].own {
                emit(Hit {
                    id,
                    start: end - self.lengths[id],
                    end,
                });
            }
            current = self.nodes[t].dict;
        }
    }

    /// Number of trie states, including root.
    #[must_use]
    pub fn states(&self) -> usize {
        self.nodes.len()
    }

    /// Scan one resident buffer; output order is increasing end, with ties unspecified.
    pub fn scan(&self, text: &[u8], mut emit: impl FnMut(Hit)) {
        let mut cursor = self.cursor();
        cursor.feed(text, &mut emit);
    }

    /// Create an independent stream cursor. Empty chunks preserve state.
    #[must_use]
    pub fn cursor(&self) -> Cursor<'_> {
        Cursor {
            automaton: self,
            state: 0,
            offset: 0,
        }
    }
}

/// Persistent state for one logical byte stream.
pub struct Cursor<'a> {
    automaton: &'a Sparse,
    state: usize,
    offset: usize,
}

impl Cursor<'_> {
    /// Feed a chunk and emit absolute offsets, including matches crossing chunks.
    ///
    /// # Panics
    /// Panics before consuming this chunk if the cumulative length overflows `usize`.
    pub fn feed(&mut self, chunk: &[u8], mut emit: impl FnMut(Hit)) {
        self.offset
            .checked_add(chunk.len())
            .expect("stream offset overflow");
        for &b in chunk {
            self.state = self.automaton.advance(self.state, b);
            self.offset += 1;
            self.automaton.outputs(self.state, self.offset, &mut emit);
        }
    }
}

/// A completed 256-column transition table plus sparse construction metadata.
pub struct Dense {
    sparse: Sparse,
    table: Vec<[usize; 256]>,
}

impl Dense {
    /// Build all transitions. Resident storage retains the sparse metadata.
    ///
    /// # Errors
    /// Returns the first empty pattern ID.
    pub fn new(patterns: &[Vec<u8>]) -> Result<Self, EmptyPattern> {
        let sparse = Sparse::new(patterns)?;
        let mut table = vec![[0; 256]; sparse.states()];
        for &(b, t) in &sparse.nodes[0].edges {
            table[0][usize::from(b)] = t;
        }
        let mut queue: VecDeque<usize> = sparse.nodes[0].edges.iter().map(|&(_, t)| t).collect();
        while let Some(s) = queue.pop_front() {
            table[s] = table[sparse.nodes[s].fail];
            for &(b, t) in &sparse.nodes[s].edges {
                table[s][usize::from(b)] = t;
                queue.push_back(t);
            }
        }
        Ok(Self { sparse, table })
    }

    /// Transition payload bytes; allocator, sparse metadata and spare capacity are extra.
    #[must_use]
    pub fn table_bytes(&self) -> usize {
        self.table.len() * std::mem::size_of::<[usize; 256]>()
    }

    /// Scan with exactly one completed-table transition per input byte.
    pub fn scan(&self, text: &[u8], mut emit: impl FnMut(Hit)) {
        let mut s = 0;
        for (i, &b) in text.iter().enumerate() {
            s = self.table[s][usize::from(b)];
            self.sparse.outputs(s, i + 1, &mut emit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Different iteration order and scalar equality from the direct baseline.
    fn oracle(p: &[Vec<u8>], text: &[u8]) -> Vec<Hit> {
        let mut result = Vec::new();
        for (id, pat) in p.iter().enumerate() {
            for start in 0..=text.len() {
                if pat.len() <= text.len() - start
                    && pat.iter().enumerate().all(|(j, b)| *b == text[start + j])
                {
                    result.push(Hit {
                        id,
                        start,
                        end: start + pat.len(),
                    });
                }
            }
        }
        result.sort_unstable();
        result
    }
    fn check(p: &[Vec<u8>], text: &[u8]) {
        let want = oracle(p, text);
        let sparse = Sparse::new(p).unwrap();
        let dense = Dense::new(p).unwrap();
        for mode in 0..3 {
            let mut got = Vec::new();
            match mode {
                0 => direct(p, text, |h| got.push(h)).unwrap(),
                1 => sparse.scan(text, |h| got.push(h)),
                _ => dense.scan(text, |h| got.push(h)),
            }
            got.sort_unstable();
            assert_eq!(got, want);
        }
        for split in 0..=text.len() {
            let mut got = Vec::new();
            let mut c = sparse.cursor();
            c.feed(&text[..split], |h| got.push(h));
            c.feed(&[], |h| got.push(h));
            c.feed(&text[split..], |h| got.push(h));
            got.sort_unstable();
            assert_eq!(got, want);
        }
        let mut got = Vec::new();
        let mut c = sparse.cursor();
        for b in text {
            c.feed(&[*b], |h| got.push(h));
        }
        got.sort_unstable();
        assert_eq!(got, want);
    }
    #[test]
    fn suffixes_duplicates_chunks_and_binary() {
        check(
            &[
                b"he".to_vec(),
                b"she".to_vec(),
                b"hers".to_vec(),
                b"his".to_vec(),
                b"he".to_vec(),
            ],
            b"ushers",
        );
        check(
            &[
                b"a".to_vec(),
                b"aa".to_vec(),
                b"aaa".to_vec(),
                b"aa".to_vec(),
            ],
            b"aaaaa",
        );
        check(&[vec![0, 255], vec![255]], &[0, 255, 0, 255]);
        check(&[], b"abc");
        check(&[b"long".to_vec()], b"");
        let all: Vec<u8> = (0..=255).collect();
        check(&[all.clone(), vec![255]], &all);
    }
    #[test]
    fn independent_small_domain() {
        let words: Vec<Vec<u8>> = (1..=3)
            .flat_map(|n| {
                (0..(1 << n)).map(move |bits| {
                    (0..n)
                        .map(|i| b'a' + u8::from(bits & (1 << i) != 0))
                        .collect()
                })
            })
            .collect();
        for a in &words {
            for b in &words {
                for n in 0..=5 {
                    for bits in 0..(1 << n) {
                        let text: Vec<u8> = (0..n)
                            .map(|i| b'a' + u8::from(bits & (1 << i) != 0))
                            .collect();
                        check(&[a.clone(), b.clone()], &text);
                    }
                }
            }
        }
    }
    #[test]
    fn rejects_empty_patterns() {
        let p = vec![b"a".to_vec(), Vec::new()];
        assert_eq!(Sparse::new(&p).err(), Some(EmptyPattern(1)));
        assert_eq!(Dense::new(&p).err(), Some(EmptyPattern(1)));
        let mut count = 0;
        assert_eq!(direct(&p, b"abc", |_| count += 1), Err(EmptyPattern(1)));
        assert_eq!(count, 0);
    }
}
