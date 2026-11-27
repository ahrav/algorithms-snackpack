//! Show which sensor blocks are admitted and how many rows match.

use block_range_indexes_and_lossy_summaries::{Index, Layout, full_scan};
use std::num::NonZeroUsize;

fn main() {
    let rows = [10, 11, 12, 13, 18, 20, 91, 93, 49, 50, 52, 54];
    let width = NonZeroUsize::new(4).unwrap();
    let minmax = Index::build(&rows, width, Layout::MinMax);
    let two = Index::build(&rows, width, Layout::TwoRanges);
    for (low, high) in [(50, 51), (51, 51), (53, 53)] {
        println!("query [{low},{high}]");
        for (name, scan) in [
            ("full", full_scan(&rows, low, high)),
            ("minmax", minmax.scan(low, high)),
            ("two", two.scan(low, high)),
        ] {
            println!(
                "  {name:6} {:2} rows checked, {} matches",
                scan.checked_rows, scan.matches
            );
        }
    }
}
