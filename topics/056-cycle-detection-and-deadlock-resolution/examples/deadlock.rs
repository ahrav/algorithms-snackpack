//! Compare a mandatory cycle with a pool that can release another unit.

use cycle_detection_and_deadlock_resolution::{Snapshot, mandatory_waits};

fn main() {
    let graph = mandatory_waits(&[vec![1], vec![0]]).expect("valid graph");
    println!("Exclusive cycle: {:?}", graph.cycle);

    let held = [vec![1, 0], vec![0, 1], vec![0, 1]];
    let want = [vec![0, 1], vec![1, 0], vec![0, 0]];
    let pooled = Snapshot::new(&[1, 2], &held, &want).expect("valid pool");
    println!("Pooled completion: {:?}", pooled.reduce());

    let exclusive = Snapshot::new(&[1, 1], &held[..2], &want[..2]).expect("valid pool");
    let victim = exclusive
        .cheapest_single_victim(&[7, 3])
        .expect("valid policy");
    println!("Exclusive victim: {victim:?}");
}
