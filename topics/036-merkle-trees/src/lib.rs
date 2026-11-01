//! Fixed-shape ordered Merkle trees with full, eager and batched updates.
//! The portable SHA-256 is a teaching implementation checked against hashlib.
//! Production callers should use a maintained cryptographic library.
#![forbid(unsafe_code)]

/// SHA-256 digest bytes.
pub type Digest = [u8; 32];

const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Portable SHA-256 over one byte slice.
///
/// # Panics
/// Requires a slice length representable as u64 and successful allocation.
#[must_use]
#[allow(
    clippy::many_single_char_names,
    reason = "SHA-256 standard state names"
)]
pub fn sha256(input: &[u8]) -> Digest {
    let mut state = [
        0x6a09_e667_u32,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(
        &(u64::try_from(input.len())
            .expect("slice length fits u64")
            .wrapping_mul(8))
        .to_be_bytes(),
    );
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, bytes) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(bytes.try_into().expect("four-byte chunk"));
        }
        for i in 16..64 {
            let x = w[i - 15];
            let y = w[i - 2];
            let s0 = x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3);
            let s1 = y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let t2 = s0.wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, x) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(x);
        }
    }
    let mut out = [0; 32];
    for (chunk, s) in out.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&s.to_be_bytes());
    }
    out
}

fn leaf(bytes: &[u8]) -> Digest {
    let mut encoded = Vec::with_capacity(bytes.len() + 1);
    encoded.push(0);
    encoded.extend_from_slice(bytes);
    sha256(&encoded)
}
fn parent(left: Digest, right: Digest) -> Digest {
    let mut encoded = [1; 65];
    encoded[1..33].copy_from_slice(&left);
    encoded[33..].copy_from_slice(&right);
    sha256(&encoded)
}

/// Update algorithm. All variants expose the final root at batch completion.
#[derive(Clone, Copy, Debug)]
pub enum Method {
    /// Recompute every digest after applying writes.
    Full,
    /// Recompute a root after each changed write.
    Eager,
    /// Sort and deduplicate dirty parents at each level.
    Batch,
}

/// Owned records and a heap-indexed digest array. Node zero is unused.
#[derive(Clone)]
pub struct Tree {
    values: Vec<Vec<u8>>,
    nodes: Vec<Digest>,
}
impl Tree {
    /// Construct a nonempty power-of-two tree.
    ///
    /// # Errors
    /// Rejects empty, non-power-of-two or overflowing node counts.
    pub fn new(values: Vec<Vec<u8>>) -> Result<Self, &'static str> {
        let n = values.len();
        if !n.is_power_of_two() {
            return Err("leaf count must be a nonempty power of two");
        }
        let len = n.checked_mul(2).ok_or("node count overflow")?;
        let mut tree = Self {
            values,
            nodes: vec![[0; 32]; len],
        };
        tree.rebuild();
        Ok(tree)
    }
    fn rebuild(&mut self) {
        let n = self.values.len();
        for (i, v) in self.values.iter().enumerate() {
            self.nodes[n + i] = leaf(v);
        }
        for i in (1..n).rev() {
            self.nodes[i] = parent(self.nodes[2 * i], self.nodes[2 * i + 1]);
        }
    }
    /// Current snapshot root.
    #[must_use]
    pub fn root(&self) -> Digest {
        self.nodes[1]
    }
    /// Apply ordered writes. Return the number of digest calls.
    /// All indices are validated before mutation. Last duplicate write wins.
    /// Equal byte writes are skipped by Eager and Batch.
    ///
    /// # Errors
    /// An out-of-range index rejects the entire batch without mutation.
    pub fn update(
        &mut self,
        writes: &[(usize, Vec<u8>)],
        method: Method,
    ) -> Result<usize, &'static str> {
        let n = self.values.len();
        if writes.iter().any(|(i, _)| *i >= n) {
            return Err("leaf index out of range");
        }
        if matches!(method, Method::Full) {
            for (i, v) in writes {
                self.values[*i].clone_from(v);
            }
            self.rebuild();
            return Ok(2 * n - 1);
        }
        let mut calls = 0;
        let mut dirty = Vec::new();
        for (i, v) in writes {
            if self.values[*i] == *v {
                continue;
            }
            self.values[*i].clone_from(v);
            let mut node = n + i;
            self.nodes[node] = leaf(v);
            calls += 1;
            if matches!(method, Method::Eager) {
                while node > 1 {
                    node /= 2;
                    self.nodes[node] = parent(self.nodes[2 * node], self.nodes[2 * node + 1]);
                    calls += 1;
                }
            } else if node > 1 {
                dirty.push(node / 2);
            }
        }
        while !dirty.is_empty() {
            dirty.sort_unstable();
            dirty.dedup();
            for &node in &dirty {
                self.nodes[node] = parent(self.nodes[2 * node], self.nodes[2 * node + 1]);
                calls += 1;
            }
            for node in &mut dirty {
                *node /= 2;
            }
            dirty.retain(|&node| node > 0);
        }
        Ok(calls)
    }
    /// Ordered sibling digests from leaf to root.
    #[must_use]
    pub fn proof(&self, index: usize) -> Option<Vec<Digest>> {
        if index >= self.values.len() {
            return None;
        }
        let mut node = self.values.len() + index;
        let mut out = Vec::new();
        while node > 1 {
            out.push(self.nodes[node ^ 1]);
            node /= 2;
        }
        Some(out)
    }
}

/// Check a proof against an externally authenticated root and declared shape.
#[must_use]
pub fn verify(bytes: &[u8], index: usize, n: usize, proof: &[Digest], root: Digest) -> bool {
    if !n.is_power_of_two() || index >= n || proof.len() != n.trailing_zeros() as usize {
        return false;
    }
    let mut node = n + index;
    let mut digest = leaf(bytes);
    for &sibling in proof {
        digest = if node.is_multiple_of(2) {
            parent(digest, sibling)
        } else {
            parent(sibling, digest)
        };
        node /= 2;
    }
    digest == root
}

#[cfg(test)]
mod tests {
    use super::*;
    fn data(n: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| i.to_le_bytes().to_vec()).collect()
    }
    #[test]
    fn updates_proofs_and_transactional_rejection() {
        for n in [1, 2, 4, 8, 64] {
            let mut expected = data(n);
            let mut trees = [
                Tree::new(expected.clone()).unwrap(),
                Tree::new(expected.clone()).unwrap(),
                Tree::new(expected.clone()).unwrap(),
            ];
            for round in 0..64usize {
                let writes = vec![
                    (round % n, vec![u8::try_from(round).unwrap()]),
                    ((round * 7) % n, vec![42]),
                    (round % n, vec![99]),
                ];
                for (i, v) in &writes {
                    expected[*i].clone_from(v);
                }
                let rebuilt = Tree::new(expected.clone()).unwrap();
                for (t, m) in trees
                    .iter_mut()
                    .zip([Method::Full, Method::Eager, Method::Batch])
                {
                    t.update(&writes, m).unwrap();
                    assert_eq!(t.root(), rebuilt.root());
                    for (i, v) in expected.iter().enumerate() {
                        let p = t.proof(i).unwrap();
                        assert!(verify(v, i, n, &p, t.root()));
                        assert!(!verify(b"wrong", i, n, &p, t.root()));
                        if n > 1 {
                            assert!(!verify(v, i, n, &p[..p.len() - 1], t.root()));
                        }
                    }
                    let root = t.root();
                    assert!(t.update(&[(0, vec![1]), (n, vec![2])], m).is_err());
                    assert_eq!(t.root(), root);
                }
            }
        }
    }
    #[test]
    fn hash_work_and_noops() {
        let mut t = Tree::new(vec![
            b"A".to_vec(),
            b"B".to_vec(),
            b"C".to_vec(),
            b"D".to_vec(),
        ])
        .unwrap();
        assert_eq!(t.update(&[(2, b"C2".to_vec())], Method::Batch), Ok(3));
        assert_eq!(t.update(&[(2, b"C2".to_vec())], Method::Batch), Ok(0));
        assert_eq!(
            t.update(&[(2, b"X".to_vec()), (3, b"Y".to_vec())], Method::Batch),
            Ok(4)
        );
        assert!(Tree::new(vec![]).is_err());
        assert!(Tree::new(data(3)).is_err());
        assert!(!verify(b"A", 0, 0, &[], [0; 32]));
        assert!(t.proof(4).is_none());
        assert!(!verify(b"A", 4, 4, &[], t.root()));
    }
}
