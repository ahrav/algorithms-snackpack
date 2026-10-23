//! Byte dictionaries: comparison, uncompressed trie, and bulk adaptive radix layouts.
//!
//! Keys are arbitrary bytes. Last duplicate wins. Prefix output is ordered by key.
//! The radix candidate is immutable and bulk built, not a concurrent mutable ART.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::mem::size_of;

/// Owned byte key and nonzero-or-zero payload.
pub type Entry = (Vec<u8>, u64);

/// Independent latest-write model, using hashing and a final ordered scan.
#[must_use]
pub fn oracle(input: &[Entry], prefix: &[u8]) -> Vec<Entry> {
    let mut map = HashMap::new();
    for (k, v) in input {
        map.insert(k.clone(), *v);
    }
    let mut out: Vec<_> = map
        .into_iter()
        .filter(|(k, _)| k.starts_with(prefix))
        .collect();
    out.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    out
}

fn canonical(input: &[Entry]) -> Vec<Entry> {
    let mut rows = input.to_vec();
    rows.sort_by(|a, b| a.0.cmp(&b.0)); // stable: retain last input duplicate
    let mut out: Vec<Entry> = Vec::new();
    for row in rows {
        if let Some(last) = out.last_mut().filter(|x| x.0 == row.0) {
            last.1 = row.1;
        } else {
            out.push(row);
        }
    }
    out
}

#[derive(Default)]
struct ByteNode {
    value: Option<u64>,
    edges: Vec<(u8, usize)>,
}

/// Uncompressed byte trie with ordered sparse edge vectors.
pub struct ByteTrie {
    nodes: Vec<ByteNode>,
}
impl ByteTrie {
    fn new(input: &[Entry]) -> Self {
        let mut this = Self {
            nodes: vec![ByteNode::default()],
        };
        for (key, value) in input {
            let mut at = 0;
            for byte in key {
                at = match this.nodes[at].edges.binary_search_by_key(byte, |e| e.0) {
                    Ok(i) => this.nodes[at].edges[i].1,
                    Err(i) => {
                        let next = this.nodes.len();
                        this.nodes.push(ByteNode::default());
                        this.nodes[at].edges.insert(i, (*byte, next));
                        next
                    }
                };
            }
            this.nodes[at].value = Some(*value);
        }
        this
    }
    fn locate(&self, key: &[u8]) -> Option<usize> {
        let mut at = 0;
        for b in key {
            let i = self.nodes[at].edges.binary_search_by_key(b, |e| e.0).ok()?;
            at = self.nodes[at].edges[i].1;
        }
        Some(at)
    }
    fn collect(&self, at: usize, out: &mut Vec<u64>) {
        if let Some(v) = self.nodes[at].value {
            out.push(v);
        }
        for &(_, child) in &self.nodes[at].edges {
            self.collect(child, out);
        }
    }
}

enum Children {
    Small(Vec<(u8, usize)>), // reserved capacity 4 or 16; scalar ordered lookup
    Indexed {
        slots: Box<[u8; 256]>,
        edges: Vec<(u8, usize)>,
    },
    Direct(Box<[usize; 256]>),
}
impl Children {
    fn new(edges: Vec<(u8, usize)>) -> Self {
        match edges.len() {
            0 => Self::Small(Vec::new()),
            1..=16 => {
                let mut small = Vec::with_capacity(if edges.len() <= 4 { 4 } else { 16 });
                small.extend(edges);
                Self::Small(small)
            }
            17..=48 => {
                let mut slots = Box::new([u8::MAX; 256]);
                for (i, &(b, _)) in edges.iter().enumerate() {
                    slots[usize::from(b)] = u8::try_from(i).expect("at most 48 slots");
                }
                Self::Indexed { slots, edges }
            }
            _ => {
                let mut direct = Box::new([usize::MAX; 256]);
                for (b, child) in edges {
                    direct[usize::from(b)] = child;
                }
                Self::Direct(direct)
            }
        }
    }
    fn get(&self, b: u8) -> Option<usize> {
        match self {
            Self::Small(e) => e.iter().find(|e| e.0 == b).map(|e| e.1),
            Self::Indexed { slots, edges } => {
                let i = slots[usize::from(b)];
                (i != u8::MAX).then(|| edges[usize::from(i)].1)
            }
            Self::Direct(d) => (d[usize::from(b)] != usize::MAX).then(|| d[usize::from(b)]),
        }
    }
    fn heap_bytes(&self) -> usize {
        match self {
            Self::Small(e) => e.capacity() * size_of::<(u8, usize)>(),
            Self::Indexed { edges, .. } => 256 + edges.capacity() * size_of::<(u8, usize)>(),
            Self::Direct(_) => 256 * size_of::<usize>(),
        }
    }
}
struct RadixNode {
    prefix: Vec<u8>,
    value: Option<u64>,
    children: Children,
}

/// Full-prefix, scalar, bulk-built adaptive radix model with arena indices.
pub struct Radix {
    nodes: Vec<RadixNode>,
    root: usize,
}
impl Radix {
    fn new(input: &[Entry]) -> Self {
        let rows = canonical(input);
        let mut this = Self {
            nodes: Vec::new(),
            root: 0,
        };
        this.root = this.build(&rows, 0);
        this
    }
    fn build(&mut self, rows: &[Entry], depth: usize) -> usize {
        let mut end = depth;
        if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
            while end < first.0.len() && end < last.0.len() && first.0[end] == last.0[end] {
                end += 1;
            }
        }
        let prefix = rows
            .first()
            .map_or_else(Vec::new, |e| e.0[depth..end].to_vec());
        let value = rows.first().filter(|e| e.0.len() == end).map(|e| e.1);
        let mut i = usize::from(value.is_some());
        let mut edges = Vec::new();
        while i < rows.len() {
            let byte = rows[i].0[end];
            let mut j = i + 1;
            while j < rows.len() && rows[j].0.get(end) == Some(&byte) {
                j += 1;
            }
            let child = self.build(&rows[i..j], end + 1);
            edges.push((byte, child));
            i = j;
        }
        let at = self.nodes.len();
        self.nodes.push(RadixNode {
            prefix,
            value,
            children: Children::new(edges),
        });
        at
    }
    fn locate(&self, key: &[u8], prefix_query: bool) -> Option<usize> {
        let mut at = self.root;
        let mut pos = 0;
        loop {
            let n = &self.nodes[at];
            let remaining = &key[pos..];
            let compared = remaining.len().min(n.prefix.len());
            if remaining[..compared] != n.prefix[..compared] {
                return None;
            }
            if remaining.len() < n.prefix.len() {
                return prefix_query.then_some(at);
            }
            pos += n.prefix.len();
            if pos == key.len() {
                return Some(at);
            }
            at = n.children.get(key[pos])?;
            pos += 1;
        }
    }
    fn collect(&self, at: usize, out: &mut Vec<u64>) {
        if let Some(v) = self.nodes[at].value {
            out.push(v);
        }
        match &self.nodes[at].children {
            Children::Small(e) | Children::Indexed { edges: e, .. } => {
                for &(_, child) in e {
                    self.collect(child, out);
                }
            }
            Children::Direct(d) => {
                for &child in d.iter().filter(|&&i| i != usize::MAX) {
                    self.collect(child, out);
                }
            }
        }
    }
}

/// The four implemented dictionary candidates.
pub enum Index {
    /// Sorted contiguous baseline.
    Flat(Vec<Entry>),
    /// Standard mutable comparison tree, queried as a frozen dictionary.
    Tree(BTreeMap<Vec<u8>, u64>),
    /// One node per byte.
    Trie(ByteTrie),
    /// Compressed adaptive model.
    Radix(Radix),
}
impl Index {
    /// Build a named candidate.
    ///
    /// # Panics
    /// Panics for an unknown candidate name.
    #[must_use]
    pub fn new(name: &str, input: &[Entry]) -> Self {
        match name {
            "flat" => Self::Flat(canonical(input)),
            "tree" => Self::Tree(input.iter().cloned().collect()),
            "trie" => Self::Trie(ByteTrie::new(input)),
            "radix" => Self::Radix(Radix::new(input)),
            _ => panic!("unknown candidate"),
        }
    }
    /// Exact lookup; an internal terminal is distinct from absence.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        match self {
            Self::Flat(e) => e
                .binary_search_by(|e| e.0.as_slice().cmp(key))
                .ok()
                .map(|i| e[i].1),
            Self::Tree(t) => t.get(key).copied(),
            Self::Trie(t) => t.locate(key).and_then(|i| t.nodes[i].value),
            Self::Radix(t) => t.locate(key, false).and_then(|i| t.nodes[i].value),
        }
    }
    /// Materialize matching values in byte-key order, including all output work.
    #[must_use]
    pub fn prefix(&self, key: &[u8]) -> Vec<u64> {
        match self {
            Self::Flat(e) => {
                let start = e.partition_point(|e| e.0.as_slice() < key);
                e[start..]
                    .iter()
                    .take_while(|e| e.0.starts_with(key))
                    .map(|e| e.1)
                    .collect()
            }
            Self::Tree(t) => t
                .range(key.to_vec()..)
                .take_while(|e| e.0.starts_with(key))
                .map(|e| *e.1)
                .collect(),
            Self::Trie(t) => {
                let mut out = Vec::new();
                if let Some(i) = t.locate(key) {
                    t.collect(i, &mut out);
                }
                out
            }
            Self::Radix(t) => {
                let mut out = Vec::new();
                if let Some(i) = t.locate(key, true) {
                    t.collect(i, &mut out);
                }
                out
            }
        }
    }
    /// Allocated capacity bytes for custom layouts; excludes allocator metadata and input.
    /// Standard B-tree internals are deliberately not estimated.
    #[must_use]
    pub fn layout(&self) -> (usize, usize, [usize; 4]) {
        match self {
            Self::Flat(e) => (
                e.len(),
                e.capacity() * size_of::<Entry>() + e.iter().map(|e| e.0.capacity()).sum::<usize>(),
                [0; 4],
            ),
            Self::Tree(t) => (t.len(), 0, [0; 4]),
            Self::Trie(t) => (
                t.nodes.len(),
                t.nodes.capacity() * size_of::<ByteNode>()
                    + t.nodes
                        .iter()
                        .map(|n| n.edges.capacity() * size_of::<(u8, usize)>())
                        .sum::<usize>(),
                [0; 4],
            ),
            Self::Radix(t) => {
                let mut counts = [0; 4];
                for n in &t.nodes {
                    let i = match &n.children {
                        Children::Small(e) => usize::from(e.len() > 4),
                        Children::Indexed { .. } => 2,
                        Children::Direct(_) => 3,
                    };
                    counts[i] += 1;
                }
                (
                    t.nodes.len(),
                    t.nodes.capacity() * size_of::<RadixNode>()
                        + t.nodes
                            .iter()
                            .map(|n| n.prefix.capacity() + n.children.heap_bytes())
                            .sum::<usize>(),
                    counts,
                )
            }
        }
    }
}

/// Deterministic test and benchmark workload. Eight-byte suffix encodes an ordinal.
///
/// # Panics
/// Panics if an ordinal or length cannot be represented as `u64`.
#[must_use]
pub fn workload(shape: &str, n: usize) -> Vec<Entry> {
    let mut rows: Vec<_> = (0..n)
        .map(|i| {
            let v = u64::try_from(i).expect("ordinal fits u64");
            let mut k = if shape == "shared" {
                b"tenant/region/catalog/items/".to_vec()
            } else {
                Vec::new()
            };
            if shape == "wide" {
                k.push(u8::try_from(i % 256).expect("byte"));
            }
            k.extend_from_slice(&v.to_be_bytes());
            (k, v)
        })
        .collect();
    let mut seed = 24020_u64;
    for i in (1..rows.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j =
            usize::try_from(seed % u64::try_from(i + 1).expect("length fits")).expect("index fits");
        rows.swap(i, j);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_pages_and_owned_lookup_survive_removal() {
        use std::sync::{Arc, RwLock};
        // A userspace ownership model, not the Linux XArray algorithm or API.
        let mut pages: BTreeMap<u64, [Option<u64>; 64]> = BTreeMap::new();
        let mut model = HashMap::new();
        for key in [0, 63, 64, 1_u64 << 40, u64::MAX] {
            pages.entry(key / 64).or_insert([None; 64])[usize::try_from(key % 64).unwrap()] =
                Some(key);
            model.insert(key, key);
        }
        for key in [0, 1, 63, 64, 65, 1_u64 << 40, u64::MAX] {
            assert_eq!(
                pages
                    .get(&(key / 64))
                    .and_then(|p| p[usize::try_from(key % 64).unwrap()]),
                model.get(&key).copied()
            );
        }
        assert_eq!(pages.len(), 4);
        let map = RwLock::new(BTreeMap::from([(7, Arc::new(String::from("car")))]));
        let held = map.read().unwrap().get(&7).cloned().unwrap();
        map.write().unwrap().remove(&7);
        assert_eq!(held.as_str(), "car");
        assert!(map.read().unwrap().get(&7).is_none());
    }
    #[test]
    fn arbitrary_bytes_terminals_last_write_and_prefix_inside_path() {
        let input = vec![
            (vec![], 0),
            (b"car".to_vec(), 1),
            (b"cart".to_vec(), 2),
            (b"carbon".to_vec(), 3),
            (vec![0, 255], 4),
            (b"car".to_vec(), 9),
        ];
        for name in ["flat", "tree", "trie", "radix"] {
            let x = Index::new(name, &input);
            for q in [
                vec![],
                b"c".to_vec(),
                b"ca".to_vec(),
                b"car".to_vec(),
                b"cart".to_vec(),
                b"carton".to_vec(),
                vec![0],
                vec![0, 255],
                vec![255],
            ] {
                let expected = oracle(&input, &q);
                assert_eq!(
                    x.prefix(&q),
                    expected.iter().map(|e| e.1).collect::<Vec<_>>(),
                    "{name} {q:?}"
                );
                assert_eq!(
                    x.get(&q),
                    input.iter().rev().find(|e| e.0 == q).map(|e| e.1),
                    "{name} {q:?}"
                );
            }
            assert_eq!(Index::new(name, &[]).prefix(&[]), Vec::<u64>::new());
        }
    }
    #[test]
    fn every_fanout_boundary_and_miss_agrees_with_independent_model() {
        for n in [0, 1, 4, 5, 16, 17, 48, 49, 255, 256] {
            let rows: Vec<_> = (0..n)
                .map(|i| {
                    (
                        vec![u8::try_from(i).unwrap(), 0, 255],
                        u64::try_from(i).unwrap(),
                    )
                })
                .collect();
            for name in ["flat", "tree", "trie", "radix"] {
                let x = Index::new(name, &rows);
                assert_eq!(
                    x.prefix(&[]),
                    oracle(&rows, &[]).iter().map(|e| e.1).collect::<Vec<_>>()
                );
                for b in 0..=255 {
                    assert_eq!(
                        x.get(&[b, 0, 255]),
                        rows.iter().find(|e| e.0 == [b, 0, 255]).map(|e| e.1)
                    );
                }
            }
        }
    }
    #[test]
    fn sampled_binary_histories_match_hash_scan_sort() {
        let mut seed = 1919_u64;
        for _ in 0..80 {
            let mut rows = Vec::new();
            for i in 0..80 {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let bytes = seed.to_be_bytes();
                rows.push((bytes[..usize::from(bytes[0] % 7)].to_vec(), i));
            }
            for name in ["flat", "tree", "trie", "radix"] {
                let x = Index::new(name, &rows);
                for (k, _) in &rows {
                    for len in 0..=k.len() {
                        let q = &k[..len];
                        assert_eq!(
                            x.prefix(q),
                            oracle(&rows, q).iter().map(|e| e.1).collect::<Vec<_>>()
                        );
                    }
                }
            }
        }
    }
}
