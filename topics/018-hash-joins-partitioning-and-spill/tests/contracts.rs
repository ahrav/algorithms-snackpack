//! File-backed progress, multiplicity and malformed-record contracts.
use hash_joins::{nested, spill, write_keys};
use std::{fs, path::PathBuf};
fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("topic018-{}-{name}", std::process::id()));
    fs::create_dir(&root).unwrap();
    root
}
#[test]
fn spill_progress_and_duplicates() {
    let root = root("spill");
    let a: Vec<_> = (0..257).map(|x| x % 23).collect();
    let b: Vec<_> = (0..131).map(|x| x % 19).collect();
    write_keys(&root.join("a"), &a).unwrap();
    write_keys(&root.join("b"), &b).unwrap();
    let (got, stats) = spill(&root.join("a"), &root.join("b"), &root, 7, 3).unwrap();
    assert_eq!(got, nested(&a, &b));
    assert!(stats.partitions > 1);
    assert!(stats.max_build_rows <= 7);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn heavy_key_terminates_without_losing_pairs() {
    let root = root("heavy");
    write_keys(&root.join("a"), &[7; 31]).unwrap();
    write_keys(&root.join("b"), &[7; 29]).unwrap();
    let (count, stats) = spill(&root.join("a"), &root.join("b"), &root, 4, 3).unwrap();
    assert_eq!(count, 899);
    assert_eq!(stats.partitions, 1);
    assert_eq!(stats.fallbacks, 1);
    assert_eq!(stats.max_build_rows, 4);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn malformed_and_empty_inputs() {
    let root = root("invalid");
    fs::write(root.join("bad"), [0; 7]).unwrap();
    write_keys(&root.join("empty"), &[]).unwrap();
    assert!(spill(&root.join("bad"), &root.join("empty"), &root, 4, 3).is_err());
    assert!(spill(&root.join("empty"), &root.join("empty"), &root, 0, 3).is_err());
    assert_eq!(
        spill(&root.join("empty"), &root.join("empty"), &root, 4, 0)
            .unwrap()
            .0,
        0
    );
    fs::remove_dir_all(root).unwrap();
}
