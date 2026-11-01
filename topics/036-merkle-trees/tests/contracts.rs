//! Independent recursive tree-shape oracle plus proof tampering cases.
use topic036_merkle_trees::{Digest, Method, Tree, sha256, verify};
fn oracle(values: &[Vec<u8>]) -> Digest {
    if values.len() == 1 {
        let mut b = vec![0];
        b.extend(&values[0]);
        return sha256(&b);
    }
    let middle = values.len() / 2;
    let mut b = vec![1];
    b.extend(oracle(&values[..middle]));
    b.extend(oracle(&values[middle..]));
    sha256(&b)
}
#[test]
fn independent_roots_and_proof_rejections() {
    for n in [1, 2, 4, 16] {
        let values: Vec<Vec<u8>> = (0..n).map(|i| vec![u8::try_from(i).unwrap(); i]).collect();
        for method in [Method::Full, Method::Eager, Method::Batch] {
            let mut tree = Tree::new(values.clone()).unwrap();
            let old = tree.root();
            assert_eq!(old, oracle(&values));
            assert!(tree.update(&[], method).is_ok());
            assert_eq!(old, tree.root());
            let writes = vec![(0, vec![]), (n - 1, vec![4]), (n - 1, vec![5])];
            let mut expected = values.clone();
            for (i, v) in &writes {
                expected[*i].clone_from(v);
            }
            tree.update(&writes, method).unwrap();
            assert_eq!(tree.root(), oracle(&expected));
            let proof = tree.proof(n - 1).unwrap();
            assert!(verify(&expected[n - 1], n - 1, n, &proof, tree.root()));
            assert!(!verify(&expected[n - 1], n - 1, n, &proof, old));
            let mut extra = proof.clone();
            extra.push([0; 32]);
            assert!(!verify(&expected[n - 1], n - 1, n, &extra, tree.root()));
            if n > 1 {
                let mut bad = proof.clone();
                bad[0][0] ^= 1;
                assert!(!verify(&expected[n - 1], n - 1, n, &bad, tree.root()));
                assert!(!verify(&expected[n - 1], 0, n, &proof, tree.root()));
            }
        }
    }
}
