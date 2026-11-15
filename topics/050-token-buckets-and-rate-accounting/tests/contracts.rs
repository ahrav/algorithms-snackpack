//! Independent accepted-history oracle and boundary contracts.
use topic050_rate_accounting::{Bounded, Meter, Virtual, Wide};

// Each accepted suffix must fit the initial allowance plus earned credit.
fn history(history: &[(u64, u64)], now: u64, cost: u64, cap: u64, rate: u64) -> (bool, u64) {
    let mut available = u128::from(cap);
    let mut sum = 0_u128;
    for &(time, charge) in history.iter().rev() {
        sum += u128::from(charge);
        available =
            available.min(u128::from(cap) + u128::from(now - time) * u128::from(rate) - sum);
    }
    let accepted = u128::from(cost) <= available;
    (
        accepted,
        u64::try_from(available - if accepted { u128::from(cost) } else { 0 }).unwrap(),
    )
}
fn boundaries<M: Meter>() {
    let mut b = M::new(10, 3);
    assert_eq!(b.admit(0, 8), Ok((true, 2)));
    assert_eq!(b.admit(1, 6), Ok((false, 5)));
    assert_eq!(b.admit(2, 6), Ok((true, 2)));
    assert!(b.admit(1, 1).is_err());
    assert_eq!(b.admit(2, 2), Ok((true, 0)));
    assert_eq!(b.admit(2, 11), Ok((false, 0)));
    assert_eq!(b.admit(100, 10), Ok((true, 0)));
    let mut zero = M::new(0, 0);
    assert_eq!(zero.admit(u64::MAX, 0), Ok((true, 0)));
    assert_eq!(zero.admit(u64::MAX, 1), Ok((false, 0)));
    let mut frozen = M::new(10, 0);
    assert_eq!(frozen.admit(0, 8), Ok((true, 2)));
    assert_eq!(frozen.admit(u64::MAX, 3), Ok((false, 2)));
    let mut max = M::new(u64::MAX, u64::MAX);
    assert_eq!(max.admit(0, u64::MAX), Ok((true, 0)));
    assert_eq!(max.admit(u64::MAX, u64::MAX), Ok((true, 0)));
}
#[test]
fn exact_boundaries() {
    boundaries::<Wide>();
    boundaries::<Bounded>();
    boundaries::<Virtual>();
}
#[test]
fn accepted_suffix_oracle() {
    for cap in [0, 1, 10, u64::MAX] {
        for rate in [0, 1, 3, u64::MAX] {
            let mut meters = (
                Wide::new(cap, rate),
                Bounded::new(cap, rate),
                Virtual::new(cap, rate),
            );
            let mut accepted = Vec::new();
            let mut now = 0;
            let mut seed = 17_u64;
            for i in 0..512 {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                now += (seed >> 32) % 5;
                let cost = match i % 11 {
                    0 => 0,
                    1 => cap,
                    2 => u64::MAX,
                    _ => seed % 20,
                };
                let want = history(&accepted, now, cost, cap, rate);
                assert_eq!(meters.0.admit(now, cost), Ok(want));
                assert_eq!(meters.1.admit(now, cost), Ok(want));
                assert_eq!(meters.2.admit(now, cost), Ok(want));
                if want.0 {
                    accepted.push((now, cost));
                }
            }
        }
    }
}
