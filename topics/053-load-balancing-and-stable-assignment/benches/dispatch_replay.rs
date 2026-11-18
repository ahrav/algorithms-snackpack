//! Build the replay under Cargo's benchmark profile.

#[path = "../examples/compare.rs"]
mod compare;

fn main() {
    if std::env::args_os().len() == 6 {
        compare::main();
    }
}
