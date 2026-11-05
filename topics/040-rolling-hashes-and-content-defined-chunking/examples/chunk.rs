//! Print deterministic chunk boundaries.
use topic_040_chunking::{Config, Policy, boundaries, fixture};
fn main() {
    let bytes = fixture(65536, "random");
    let c = Config::new(64, 256, 1024, 32).expect("valid settings");
    for p in [
        Policy::Fixed,
        Policy::Direct,
        Policy::Rolling,
        Policy::Gear,
        Policy::Normalized,
    ] {
        let ends = boundaries(&bytes, c, p);
        println!(
            "{p:?}: {} chunks; first ends {:?}",
            ends.len(),
            &ends[..ends.len().min(8)]
        );
    }
}
