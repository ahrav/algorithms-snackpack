//! Immutable byte dictionaries: prefix sharing, residual outputs, and exact lookup.
use std::collections::{BTreeMap, HashMap};
/// Owned byte key and unsigned output.
pub type Record = (Vec<u8>, u64);
/// Candidate implementation.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// Sorted vector baseline.
    Flat,
    /// Full byte trie with no suffix sharing.
    Trie,
    /// Exact interning with all outputs at accepting states.
    Final,
    /// Minimum-output factoring followed by exact residual interning.
    Pushed,
}
/// Rejected construction input.
#[derive(Debug, PartialEq, Eq)]
pub enum BuildError {
    /// At least one key exceeded 64 bytes.
    KeyTooLong,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Node {
    terminal: Option<u64>,
    arcs: Vec<(u8, u64, usize)>,
}
#[derive(Default)]
struct BuildNode {
    terminal: Option<u64>,
    children: BTreeMap<u8, usize>,
}
/// Immutable generation. Keys are raw bytes of at most 64 bytes; duplicates use last write.
pub struct Index {
    flat: Option<Vec<Record>>,
    nodes: Vec<Node>,
    root: usize,
    initial: u64,
}
impl Index {
    /// Build from any arrival order; reject overlong keys before construction.
    ///
    /// # Errors
    /// Returns `KeyTooLong` if any key exceeds 64 bytes.
    pub fn build(input: &[Record], kind: Kind) -> Result<Self, BuildError> {
        if input.iter().any(|(k, _)| k.len() > 64) {
            return Err(BuildError::KeyTooLong);
        }
        let mut sorted = BTreeMap::new();
        for (k, v) in input {
            sorted.insert(k.clone(), *v);
        }
        let records: Vec<Record> = sorted.into_iter().collect();
        if matches!(kind, Kind::Flat) {
            return Ok(Self {
                flat: Some(records),
                nodes: vec![],
                root: 0,
                initial: 0,
            });
        }
        let mut trie = vec![BuildNode::default()];
        for (k, v) in records {
            let mut q = 0;
            for b in k {
                q = if let Some(&c) = trie[q].children.get(&b) {
                    c
                } else {
                    let c = trie.len();
                    trie.push(BuildNode::default());
                    trie[q].children.insert(b, c);
                    c
                };
            }
            trie[q].terminal = Some(v);
        }
        let mut result = Self {
            flat: None,
            nodes: vec![],
            root: 0,
            initial: 0,
        };
        let mut registry = HashMap::new();
        let (root, initial) = finish(0, &trie, kind, &mut result.nodes, &mut registry);
        result.root = root;
        result.initial = initial;
        Ok(result)
    }
    /// Exact full-key lookup. Consuming a nonaccepting prefix returns `None`.
    #[inline(never)]
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<u64> {
        if let Some(flat) = &self.flat {
            return flat
                .binary_search_by(|(k, _)| k.as_slice().cmp(key))
                .ok()
                .map(|i| flat[i].1);
        }
        let mut q = self.root;
        let mut out = self.initial;
        for &b in key {
            let arcs = &self.nodes[q].arcs;
            let i = arcs.binary_search_by_key(&b, |a| a.0).ok()?;
            out = out.checked_add(arcs[i].1)?;
            q = arcs[i].2;
        }
        self.nodes[q].terminal.and_then(|v| out.checked_add(v))
    }
    /// Emit sorted matching key/value pairs, including an accepting prefix itself.
    ///
    /// # Panics
    /// Panics only if an internal output invariant is broken. Constructed paths
    /// telescope to an original `u64` value, so every intermediate sum fits.
    #[must_use]
    pub fn prefix(&self, prefix: &[u8]) -> Vec<Record> {
        if let Some(flat) = &self.flat {
            let start = flat.partition_point(|(k, _)| k.as_slice() < prefix);
            return flat[start..]
                .iter()
                .take_while(|(k, _)| k.starts_with(prefix))
                .cloned()
                .collect();
        }
        let mut q = self.root;
        let mut out = self.initial;
        for &b in prefix {
            let arcs = &self.nodes[q].arcs;
            let Ok(i) = arcs.binary_search_by_key(&b, |a| a.0) else {
                return vec![];
            };
            out = out.checked_add(arcs[i].1).expect("bounded path output");
            q = arcs[i].2;
        }
        let mut result = vec![];
        self.emit(q, out, &mut prefix.to_vec(), &mut result);
        result
    }
    fn emit(&self, q: usize, out: u64, key: &mut Vec<u8>, result: &mut Vec<Record>) {
        let node = &self.nodes[q];
        if let Some(v) = node.terminal {
            result.push((
                key.clone(),
                out.checked_add(v).expect("bounded final output"),
            ));
        }
        for &(b, v, c) in &node.arcs {
            key.push(b);
            self.emit(
                c,
                out.checked_add(v).expect("bounded arc output"),
                key,
                result,
            );
            key.pop();
        }
    }
    /// State count, arc count, and retained heap capacity bytes (not RSS or peak build memory).
    #[must_use]
    pub fn stats(&self) -> (usize, usize, usize) {
        if let Some(flat) = &self.flat {
            return (
                0,
                0,
                flat.capacity() * std::mem::size_of::<Record>()
                    + flat.iter().map(|(k, _)| k.capacity()).sum::<usize>(),
            );
        }
        (
            self.nodes.len(),
            self.nodes.iter().map(|n| n.arcs.len()).sum(),
            self.nodes.capacity() * std::mem::size_of::<Node>()
                + self
                    .nodes
                    .iter()
                    .map(|n| n.arcs.capacity() * std::mem::size_of::<(u8, u64, usize)>())
                    .sum::<usize>(),
        )
    }
}
fn finish(
    q: usize,
    trie: &[BuildNode],
    kind: Kind,
    nodes: &mut Vec<Node>,
    registry: &mut HashMap<Node, usize>,
) -> (usize, u64) {
    let children: Vec<_> = trie[q]
        .children
        .iter()
        .map(|(&b, &c)| {
            let (id, p) = finish(c, trie, kind, nodes, registry);
            (b, p, id)
        })
        .collect();
    let potential = if matches!(kind, Kind::Pushed) {
        trie[q]
            .terminal
            .into_iter()
            .chain(children.iter().map(|a| a.1))
            .min()
            .unwrap_or(0)
    } else {
        0
    };
    let node = Node {
        terminal: trie[q].terminal.map(|v| v - potential),
        arcs: children
            .into_iter()
            .map(|(b, p, c)| (b, p - potential, c))
            .collect(),
    };
    if !matches!(kind, Kind::Trie)
        && let Some(&id) = registry.get(&node)
    {
        return (id, potential);
    }
    let id = nodes.len();
    nodes.push(node.clone());
    if !matches!(kind, Kind::Trie) {
        registry.insert(node, id);
    }
    (id, potential)
}
/// Independent last-write hash-map oracle; scan/filter/sort has no graph logic.
#[must_use]
pub fn oracle(input: &[Record], prefix: &[u8]) -> Vec<Record> {
    let mut map = HashMap::new();
    for (k, v) in input {
        map.insert(k.clone(), *v);
    }
    let mut rows: Vec<_> = map
        .into_iter()
        .filter(|(k, _)| k.starts_with(prefix))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}
#[cfg(test)]
mod tests {
    use super::*;
    const KINDS: [Kind; 4] = [Kind::Flat, Kind::Trie, Kind::Final, Kind::Pushed];
    #[test]
    fn exact_outputs_and_prefix_terminals() {
        let rows = vec![
            (vec![], u64::MAX),
            (b"a".to_vec(), 0),
            (b"ab".to_vec(), 12),
            (b"b".to_vec(), u64::MAX),
            (vec![0, 255], 42),
            (b"ab".to_vec(), 17),
        ];
        for kind in KINDS {
            let x = Index::build(&rows, kind).unwrap();
            assert_eq!(x.prefix(b""), oracle(&rows, b""));
            for p in [b"a".as_slice(), b"ab", b"missing", &[0], &[255]] {
                assert_eq!(x.prefix(p), oracle(&rows, p));
            }
            assert_eq!(x.get(b"ab"), Some(17));
            assert_eq!(x.get(&[0]), None);
            assert_eq!(x.get(b"aba"), None);
        }
    }
    #[test]
    fn pushing_shares_residual_functions() {
        let rows = vec![
            (b"bat".to_vec(), 10),
            (b"bats".to_vec(), 12),
            (b"cat".to_vec(), 20),
            (b"cats".to_vec(), 22),
        ];
        let t = Index::build(&rows, Kind::Trie).unwrap();
        let f = Index::build(&rows, Kind::Final).unwrap();
        let p = Index::build(&rows, Kind::Pushed).unwrap();
        assert_eq!(t.stats().0, 9);
        assert_eq!(f.stats().0, 9);
        assert_eq!(p.stats().0, 5);
        assert_eq!(p.get(b"cat"), Some(20));
        assert_eq!(p.prefix(b""), rows);
        assert_eq!(p.get(b"ca"), None);
    }
    #[test]
    fn generated_differential_and_arrival_order() {
        for seed in 0..64_u64 {
            let mut s = seed + 1;
            let mut rows = vec![];
            for _ in 0..80 {
                s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let len = usize::try_from(s % 7).unwrap();
                let key = (0..len)
                    .map(|i| u8::try_from((s >> (i * 8)) % 4).unwrap())
                    .collect();
                rows.push((key, s));
            }
            let expected = oracle(&rows, b"");
            for kind in KINDS {
                let x = Index::build(&rows, kind).unwrap();
                assert_eq!(x.prefix(b""), expected);
                for (k, v) in &expected {
                    assert_eq!(x.get(k), Some(*v));
                    for n in 0..=k.len() {
                        assert_eq!(x.prefix(&k[..n]), oracle(&rows, &k[..n]));
                    }
                }
            }
        }
    }
    #[test]
    fn empty_maximum_and_rejection() {
        for kind in KINDS {
            let e = Index::build(&[], kind).unwrap();
            assert_eq!(e.get(b""), None);
            assert!(e.prefix(b"").is_empty());
            let rows = vec![(vec![255; 64], u64::MAX), (vec![], 3)];
            let x = Index::build(&rows, kind).unwrap();
            assert_eq!(x.get(&[255; 64]), Some(u64::MAX));
            assert_eq!(x.get(&[255; 63]), None);
            assert!(matches!(
                Index::build(&[(vec![1; 65], 0)], kind),
                Err(BuildError::KeyTooLong)
            ));
        }
    }
}
