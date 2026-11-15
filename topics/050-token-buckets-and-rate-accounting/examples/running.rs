//! The three-request example.
use topic050_rate_accounting::{Meter, Wide};
fn main() {
    let mut b = Wide::new(10, 3);
    for (now, cost) in [(0, 8), (1, 6), (2, 6)] {
        println!("{:?}", b.admit(now, cost));
    }
}
