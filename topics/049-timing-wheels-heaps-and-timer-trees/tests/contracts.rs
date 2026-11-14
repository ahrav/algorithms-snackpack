//! Independent map-and-sort oracle and boundary tests.
use std::collections::BTreeMap;
use topic_049_timers::{Candidate, Event, MAX_TICK, replay};

const CANDIDATES: [Candidate; 4] = [
    Candidate::Scan,
    Candidate::Heap,
    Candidate::Tree,
    Candidate::Wheel,
];

type Oracle = (Vec<(u32, u32, usize)>, Vec<(usize, u32)>);

fn oracle(events: &[Event]) -> Oracle {
    let mut timers = BTreeMap::new();
    let mut output = Vec::new();
    for event in events {
        match *event {
            Event::Arm(id, d) => {
                timers.insert(id, d);
            }
            Event::Cancel(id) => {
                timers.remove(&id);
            }
            Event::Advance(t) => {
                let mut due: Vec<_> = timers
                    .iter()
                    .filter(|&(_, &d)| d <= t)
                    .map(|(&id, &d)| (d, id))
                    .collect();
                due.sort_unstable();
                for (d, id) in due {
                    output.push((t, d, id));
                    timers.remove(&id);
                }
            }
        }
    }
    (output, timers.into_iter().collect())
}

fn check(ids: usize, events: &[Event]) {
    let expected = oracle(events);
    for candidate in CANDIDATES {
        let result = replay(candidate, ids, events).unwrap();
        assert_eq!(
            (&result.fired, &result.live),
            (&expected.0, &expected.1),
            "{candidate:?}, {events:?}"
        );
    }
}

#[test]
fn exact_boundaries_stale_rearm_and_bursts() {
    use Event::{Advance as T, Arm as A, Cancel as C};
    check(0, &[]);
    check(0, &[T(0), T(MAX_TICK)]);
    let events = [
        A(0, 1),
        A(1, 65),
        A(2, 64),
        T(1),
        C(0),
        A(0, 65),
        A(0, 65),
        C(2),
        T(64),
        T(64),
        T(65),
        T(66),
        A(0, MAX_TICK),
        C(0),
        A(0, MAX_TICK),
        T(MAX_TICK),
    ];
    for length in 0..=events.len() {
        check(3, &events[..length]);
    }
    check(2, &[A(0, 5), A(1, 5), A(0, 9), T(5), T(9)]);
    check(3, &[A(2, 5), A(0, 5), A(1, 5), T(9)]);
    check(1, &[C(0), C(0), A(0, 1), C(0), T(1)]);
}

#[test]
fn rejects_invalid_input_before_replay() {
    for c in CANDIDATES {
        for events in [
            vec![Event::Arm(1, 1)],
            vec![Event::Cancel(1)],
            vec![Event::Arm(0, 0)],
            vec![Event::Advance(2), Event::Arm(0, 2)],
            vec![Event::Advance(2), Event::Advance(1)],
            vec![Event::Advance(MAX_TICK + 1)],
            vec![Event::Arm(0, MAX_TICK + 1)],
        ] {
            assert!(replay(c, 1, &events).is_err());
        }
        assert!(replay(c, 65_537, &[]).is_err());
    }
}

#[test]
fn generated_prefixes_against_independent_model() {
    for seed in 0_u64..96 {
        let mut value = seed + 1;
        let mut now = 0;
        let mut events = Vec::new();
        for _ in 0..80 {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let id = usize::try_from((value >> 32) % 5).unwrap();
            events.push(match value % 5 {
                0 => Event::Cancel(id),
                1 => {
                    now += u32::try_from((value >> 24) % 100).unwrap();
                    Event::Advance(now)
                }
                _ => Event::Arm(id, now + 1 + u32::try_from((value >> 16) % 129).unwrap()),
            });
            check(5, &events);
        }
    }
}
