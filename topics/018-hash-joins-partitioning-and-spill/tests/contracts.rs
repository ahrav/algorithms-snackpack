//! File-backed progress, multiplicity and malformed-record contracts.
use hash_joins::{nested, spill, write_keys};
use std::{fs, io, path::PathBuf};
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
    // One split reads 31+29 rows and writes them once. The fallback leaf reads the
    // 31 build rows and rescans 29 probe rows for each of the 8 build chunks.
    assert_eq!(stats.writes, 60);
    assert_eq!(stats.reads, 60 + 31 + 8 * 29);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn leaf_counts_reads_without_writes() {
    let root = root("leaf");
    write_keys(&root.join("a"), &[1, 2, 3]).unwrap();
    write_keys(&root.join("b"), &[1, 1, 4]).unwrap();
    let (count, stats) = spill(&root.join("a"), &root.join("b"), &root, 4, 3).unwrap();
    assert_eq!(count, 2);
    assert_eq!((stats.reads, stats.writes), (6, 0));
    assert_eq!((stats.partitions, stats.fallbacks), (0, 0));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn heavy_key_with_empty_probe_bucket_skips_fallback() {
    let root = root("disjoint");
    write_keys(&root.join("a"), &[0; 31]).unwrap();
    write_keys(&root.join("b"), &[1; 29]).unwrap();
    let (count, stats) = spill(&root.join("a"), &root.join("b"), &root, 4, 3).unwrap();
    assert_eq!(count, 0);
    assert_eq!(stats.partitions, 1);
    // Build and probe route to different children, so all traffic comes from
    // splitting each input once.
    assert_eq!(stats.fallbacks, 0);
    assert_eq!((stats.reads, stats.writes), (60, 60));
    assert_eq!(stats.max_build_rows, 0);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn malformed_and_empty_inputs() {
    let root = root("invalid");
    fs::write(root.join("bad"), [0; 7]).unwrap();
    write_keys(&root.join("empty"), &[]).unwrap();
    let truncated = spill(&root.join("bad"), &root.join("empty"), &root, 4, 3).unwrap_err();
    assert_eq!(truncated.kind(), io::ErrorKind::InvalidData);
    let zero_budget = spill(&root.join("empty"), &root.join("empty"), &root, 0, 3).unwrap_err();
    assert_eq!(zero_budget.kind(), io::ErrorKind::InvalidInput);
    let deep = spill(&root.join("empty"), &root.join("empty"), &root, 4, 32).unwrap_err();
    assert_eq!(deep.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(
        spill(&root.join("empty"), &root.join("empty"), &root, 4, 0)
            .unwrap()
            .0,
        0
    );
    // Rejections and empty joins leave exactly the two input files in scratch.
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    fs::remove_dir_all(root).unwrap();
}
