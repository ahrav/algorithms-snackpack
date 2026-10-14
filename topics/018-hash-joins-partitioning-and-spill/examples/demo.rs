//! Seven-pair running example, including temporary file-backed replay.
use hash_joins::{chunk, hash, merge, nested, partition, spill, write_keys};
fn main() -> std::io::Result<()> {
    let a = [2, 2, 4, 7];
    let b = [2, 2, 2, 4, 8];
    for count in [
        nested(&a, &b),
        hash(&a, &b),
        merge(&a, &b),
        partition(&a, &b),
        chunk(&a, &b, 2),
    ] {
        assert_eq!(count, 7);
    }
    let root = std::env::temp_dir().join(format!("topic018-demo-{}", std::process::id()));
    std::fs::create_dir(&root)?;
    write_keys(&root.join("a"), &a)?;
    write_keys(&root.join("b"), &b)?;
    let (count, stats) = spill(&root.join("a"), &root.join("b"), &root, 2, 3)?;
    assert_eq!(count, 7);
    println!("pairs={count} {stats:?}");
    std::fs::remove_dir_all(root)
}
