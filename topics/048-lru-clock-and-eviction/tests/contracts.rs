//! Independent queue models compare complete state after every request.
use std::collections::VecDeque;
use topic048_eviction::{Cache, Policy, Snapshot};

fn model(trace: &[usize], cap: usize, policy: Policy) -> (Vec<bool>, Snapshot) {
    let mut a = VecDeque::new();
    let mut b = VecDeque::new();
    let mut slots = vec![usize::MAX; cap];
    let mut bits = vec![false; cap];
    let mut hand = 0;
    let mut hits = Vec::new();
    for &k in trace {
        if cap == 0 {
            hits.push(false);
            continue;
        }
        if policy == Policy::Clock {
            if let Some(p) = slots.iter().position(|&x| x == k) {
                bits[p] = true;
                hits.push(true);
            } else {
                hits.push(false);
                while slots[hand] != usize::MAX && bits[hand] {
                    bits[hand] = false;
                    hand = (hand + 1) % cap;
                }
                slots[hand] = k;
                bits[hand] = true;
                hand = (hand + 1) % cap;
            }
        } else if let Some(p) = b.iter().position(|&x| x == k) {
            b.remove(p);
            b.push_back(k);
            hits.push(true);
        } else if let Some(p) = a.iter().position(|&x| x == k) {
            a.remove(p);
            hits.push(true);
            if policy == Policy::Segmented {
                b.push_back(k);
                if b.len() > cap / 2 {
                    a.push_back(b.pop_front().unwrap());
                }
            } else {
                a.push_back(k);
            }
        } else {
            hits.push(false);
            if a.len() + b.len() == cap {
                a.pop_front();
            }
            a.push_back(k);
        }
    }
    let queues = match policy {
        Policy::Clock => Vec::new(),
        Policy::Segmented => vec![a.into(), b.into()],
        _ => vec![a.into()],
    };
    (
        hits,
        Snapshot {
            queues,
            slots: if policy == Policy::Clock {
                slots
            } else {
                vec![]
            },
            bits: if policy == Policy::Clock {
                bits
            } else {
                vec![]
            },
            hand,
        },
    )
}

#[test]
fn exhaustive_transition_states() {
    for cap in 0..=4 {
        for policy in [
            Policy::Scan,
            Policy::Linked,
            Policy::Clock,
            Policy::Segmented,
        ] {
            for code in 0..4096usize {
                let mut x = code;
                let trace: Vec<_> = (0..6)
                    .map(|_| {
                        let k = x % 4;
                        x /= 4;
                        k
                    })
                    .collect();
                let mut cache = Cache::new(policy, cap, 4);
                let mut hits = Vec::new();
                for (i, &k) in trace.iter().enumerate() {
                    hits.push(cache.access(k).unwrap());
                    let (expected, state) = model(&trace[..=i], cap, policy);
                    assert_eq!(hits, expected, "{policy:?} cap={cap} trace={trace:?}");
                    assert_eq!(
                        cache.snapshot(),
                        state,
                        "{policy:?} cap={cap} trace={trace:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn scan_resistance_and_clock_work() {
    let trace = [0, 1, 0, 1, 2, 3, 4, 0, 1];
    for (policy, expected) in [
        (Policy::Scan, 2),
        (Policy::Linked, 2),
        (Policy::Clock, 2),
        (Policy::Segmented, 3),
    ] {
        let mut cache = Cache::new(policy, 3, 5);
        let hits = trace
            .map(|k| cache.access(k).unwrap())
            .into_iter()
            .filter(|&h| h)
            .count();
        assert_eq!(hits, expected);
        if policy == Policy::Clock {
            assert_eq!(cache.inspections, 13);
        }
        if policy == Policy::Scan {
            assert_eq!(cache.inspections, 12);
        }
    }
}

#[test]
fn invalid_key_preserves_state_and_zero_universe() {
    for policy in [
        Policy::Scan,
        Policy::Linked,
        Policy::Clock,
        Policy::Segmented,
    ] {
        let mut c = Cache::new(policy, 2, 3);
        c.access(1).unwrap();
        let before = c.snapshot();
        assert_eq!(c.access(3), Err("key outside universe"));
        assert_eq!(c.snapshot(), before);
        assert_eq!(
            Cache::new(policy, 0, 0).access(0),
            Err("key outside universe")
        );
    }
}
