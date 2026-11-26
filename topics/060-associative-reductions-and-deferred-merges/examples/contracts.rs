//! Focused checks for grouping, snapshots, operand order, and arithmetic.

fn main() {
    let base = (20_i64, 2_u64);
    let first = [10_i64, 30];
    let later = [80_i64];
    let state = |xs: &[i64]| (xs.iter().sum::<i64>(), xs.len() as u64);
    let merge = |a: (i64, u64), b: (i64, u64)| (a.0 + b.0, a.1 + b.1);
    let (earlier, newer) = (state(&first), state(&later));
    assert_eq!(merge(base, earlier), (60, 4));
    assert_eq!(merge(merge(base, earlier), newer), (140, 5));
    assert_eq!(
        merge(merge(base, earlier), newer),
        merge(base, merge(earlier, newer))
    );
    let apply = |(m, d): (u64, u64), x: u64| m.wrapping_mul(x).wrapping_add(d);
    assert_eq!(apply((1, 3), apply((2, 0), 10)), 23);
    assert_eq!(apply((2, 0), apply((1, 3), 10)), 26);
    let (x, y, z) = std::hint::black_box((1e16_f64, -1e16_f64, 1.0_f64));
    assert_eq!(((x + y) + z, x + (y + z)), (1.0, 0.0));
    assert_eq!(i64::MAX.checked_add(1), None);
    assert_eq!(i64::MAX.checked_add(1 + -1), Some(i64::MAX));
    println!("retained=60/4 latest=140/5 ordered=23 reversed=26 floats=1,0");
}
