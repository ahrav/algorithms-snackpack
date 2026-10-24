//! Exact and prefix lookup for keys that terminate inside shared paths.
use topic024_tries_radix_trees_and_art::Index;
fn main() {
    let rows = vec![
        (b"car".to_vec(), 10),
        (b"cart".to_vec(), 20),
        (b"carbon".to_vec(), 30),
        (b"cab".to_vec(), 40),
    ];
    for name in ["flat", "tree", "trie", "radix"] {
        let index = Index::new(name, &rows);
        assert_eq!(index.get(b"ca"), None);
        assert_eq!(index.get(b"car"), Some(10));
        assert_eq!(index.prefix(b"ca"), vec![40, 10, 30, 20]);
        assert_eq!(index.prefix(b"c"), vec![40, 10, 30, 20]);
    }
    println!("four candidates agree: ca misses; prefix ca = [40, 10, 30, 20]");
}
