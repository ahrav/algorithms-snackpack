//! Eight-item sampling example.
use sampling_lab::{Uniform, uniform, weighted_heap};
fn main() {
    for method in [Uniform::R, Uniform::X, Uniform::PriorityHeap] {
        println!("{method:?}: {:?}", uniform(8, 3, 42, method));
    }
    println!(
        "weighted: {:?}",
        weighted_heap(&[1., 1., 1., 1., 4., 1., 1., 1.], 3, 42)
            .unwrap()
            .iter()
            .map(|x| x.id)
            .collect::<Vec<_>>()
    );
}
