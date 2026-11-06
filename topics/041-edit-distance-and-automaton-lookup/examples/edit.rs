//! Complete dictionary lookup for a one-edit query.
use topic_041_edit::Trie;
fn main() {
    let words: Vec<_> = ["cat", "cart", "cut", "dog"]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
    let (ids, edges) = Trie::new(&words).search(b"cat", 1);
    println!("matching IDs={ids:?}, evaluated edges={edges}");
    assert_eq!(ids, vec![0, 1, 2]);
}
