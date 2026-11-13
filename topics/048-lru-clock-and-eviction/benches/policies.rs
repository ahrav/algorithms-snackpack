//! Standalone process benchmark; the Python runner supplies the balanced schedule.
#[path = "../examples/support/mod.rs"]
mod support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run()
}
