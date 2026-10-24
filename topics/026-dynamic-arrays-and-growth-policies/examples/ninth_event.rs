//! Executable growth-policy contract checks.
use algorithms_growth::{Array, Growth, Shrink};
fn main() {
    let mut a = Array::<1>::new(Growth::Double, Shrink::Retain, 0).unwrap();
    for id in 1..=9 {
        a.try_push([id]).unwrap();
    }
    println!(
        "length={} capacity={} grows={}",
        a.as_slice().len(),
        a.capacity(),
        a.stats().grows
    );
    assert_eq!(a.as_slice().len(), 9);
    assert!(a.capacity() >= 9);
}
