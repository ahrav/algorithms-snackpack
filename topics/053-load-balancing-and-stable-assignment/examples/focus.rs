//! The four-backend locality example from the lesson.

fn choose(t: u64, tails: [u64; 4], local: [usize; 2], remote: [usize; 2], penalty: u64) -> usize {
    let wait = |i: usize| tails[i].saturating_sub(t);
    let best = |[a, b]: [usize; 2]| if wait(a) <= wait(b) { a } else { b };
    let near = best(local);
    if wait(near) > penalty {
        let far = best(remote);
        if wait(far) < wait(near) - penalty {
            return far;
        }
    }
    near
}

fn main() {
    let mut tails = [18_u64, 13, 10, 15];
    let first = choose(10, tails, [0, 1], [2, 3], 5);
    assert_eq!(first, 1);
    tails[first] = tails[first].max(10) + 4;
    let second = choose(11, tails, [0, 1], [2, 3], 5);
    assert_eq!(second, 2);
    assert_eq!(tails, [18, 17, 10, 15]);
    println!("tick=10 backend=B response=7");
    println!("tick=11 backend=C response=9");
}
