//! Default Cargo benchmark smoke; full process schedule is in measurements/runner.py.
use std::hint::black_box;
use topic_042_lcs_diff::{Candidate, diff, workload};
fn main() {
    let (a, b) = workload("sparse", 64);
    for c in [Candidate::Matrix, Candidate::Hirschberg, Candidate::Myers] {
        black_box(diff(&a, &b, c));
    }
}
