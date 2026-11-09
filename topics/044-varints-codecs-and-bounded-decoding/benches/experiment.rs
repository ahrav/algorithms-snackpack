//! Direct sample executable; use measurements/run.py for balanced process runs.
#[path = "../examples/experiment.rs"]
mod measured;
fn main() {
    measured::main();
}
