//! Run a fixed trace and print policy outcomes and separate setup/request timings.
#[path = "support/mod.rs"]
mod support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    support::run()
}
