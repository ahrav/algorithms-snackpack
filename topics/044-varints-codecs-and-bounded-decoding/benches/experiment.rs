//! Launch the frozen native process experiment.
fn main() {
    let status = std::process::Command::new("python3")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/measurements/run.py"))
        .status()
        .expect("python3 runner");
    assert!(status.success());
}
