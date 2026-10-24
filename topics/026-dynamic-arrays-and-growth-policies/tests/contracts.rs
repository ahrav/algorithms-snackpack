//! Executable growth-policy contract checks.
use algorithms_growth::{Array, Growth, Shrink};

#[test]
fn hint_is_not_an_admission_bound() {
    let mut a = Array::<1>::new(Growth::Double, Shrink::Retain, 5).unwrap();
    for value in 0..33 {
        a.try_push([value]).unwrap();
    }
    assert_eq!(a.as_slice(), &(0..33).map(|x| [x]).collect::<Vec<_>>());
    let capacity = a.capacity();
    a.clear();
    assert_eq!(a.capacity(), capacity);
}
