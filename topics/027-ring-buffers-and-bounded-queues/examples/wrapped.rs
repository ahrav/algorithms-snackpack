//! Four slots retain FIFO order across wraparound.
use algorithms_ring::Ring;
fn main() {
    let mut queue = Ring::<u64, 1>::new(4);
    for value in 10..14 {
        queue.push(value).unwrap();
    }
    assert_eq!(queue.push(14), Err(14));
    assert_eq!(queue.pop(), Some(10));
    queue.push(14).unwrap();
    println!("{:?}", queue.as_slices());
    assert_eq!(queue.pop(), Some(11));
}
