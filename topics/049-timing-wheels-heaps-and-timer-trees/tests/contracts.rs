//! Independent map-and-sort oracle and boundary tests.
use std::collections::BTreeMap;
use topic_049_timers::{Candidate, Event, MAX_TICK, Work, replay, workload};

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
fn rejects_invalid_input_with_exact_errors() {
    use Event::{Advance as T, Arm as A, Cancel as C};
    for c in CANDIDATES {
        for (events, error) in [
            (vec![A(1, 1)], "ID outside capacity"),
            (vec![C(1)], "ID outside capacity"),
            (vec![A(0, 0)], "deadline outside future horizon"),
            (vec![T(2), A(0, 2)], "deadline outside future horizon"),
            (vec![A(0, MAX_TICK + 1)], "deadline outside future horizon"),
            (vec![T(2), T(1)], "invalid clock advance"),
            (vec![T(MAX_TICK + 1)], "invalid clock advance"),
            // A valid prefix does not produce a partial result.
            (vec![A(0, 1), T(1), C(1)], "ID outside capacity"),
        ] {
            assert_eq!(replay(c, 1, &events), Err(error), "{c:?}, {events:?}");
        }
        assert_eq!(replay(c, 65_537, &[]), Err("ID capacity exceeds 65536"));
        let largest = replay(c, 65_536, &[]).unwrap();
        assert!(largest.fired.is_empty() && largest.live.is_empty(), "{c:?}");
    }
}

#[test]
fn work_counters_match_each_documented_mechanism() {
    use Event::{Advance as T, Arm as A, Cancel as C};
    let work = |inspections, ticks, peak_records| Work {
        inspections,
        ticks,
        peak_records,
    };
    // Expected order follows CANDIDATES: scan, heap, tree, wheel.
    let cases = [
        // README trace: the stale first arm of ID 0 is popped or visited once.
        (
            2,
            vec![A(0, 5), A(1, 5), A(0, 9), T(5), T(9)],
            [work(4, 0, 2), work(3, 0, 3), work(2, 0, 2), work(3, 9, 3)],
        ),
        // Deadline 65 shares slot 1: the wheel visits it at ticks 1 and 65.
        (
            1,
            vec![A(0, 65), T(65)],
            [work(1, 0, 1), work(1, 0, 1), work(1, 0, 1), work(2, 65, 1)],
        ),
        // A cancelled record stays in the lazy heap and wheel; the tree removes it.
        (
            1,
            vec![A(0, 5), C(0), T(5)],
            [work(1, 0, 1), work(1, 0, 1), work(0, 0, 1), work(1, 5, 1)],
        ),
    ];
    for (ids, events, expected) in cases {
        for (candidate, want) in CANDIDATES.into_iter().zip(expected) {
            let result = replay(candidate, ids, &events).unwrap();
            assert_eq!(result.work, want, "{candidate:?}, {events:?}");
        }
    }
}

#[test]
fn workload_generates_declared_shapes_and_rejects_other_dimensions() {
    let plain = |events: Vec<Event>| -> Vec<(char, usize, u32)> {
        events
            .into_iter()
            .map(|event| match event {
                Event::Arm(id, d) => ('A', id, d),
                Event::Cancel(id) => ('C', id, 0),
                Event::Advance(t) => ('T', 0, t),
            })
            .collect()
    };
    // near: deadline 1+(73*ID+offset) mod 256, IDs 0 and 4 rearm with offset 17,
    // ID modulo 10 below 9 cancels, polls 1..=256.
    let near = plain(workload(5, "near", 90).unwrap());
    let mut expected = vec![
        ('A', 0, 1),
        ('A', 1, 74),
        ('A', 2, 147),
        ('A', 3, 220),
        ('A', 4, 37),
        ('A', 0, 18),
        ('A', 4, 54),
    ];
    expected.extend((0..5).map(|id| ('C', id, 0)));
    expected.extend((1..=256).map(|t| ('T', 0, t)));
    assert_eq!(near, expected);
    // far: modulus 65536, no cancellation, polls at multiples of 256.
    let far = plain(workload(2, "far", 0).unwrap());
    let mut expected = vec![('A', 0, 1), ('A', 1, 74), ('A', 0, 18)];
    expected.extend((1..=256).map(|i| ('T', 0, i * 256)));
    assert_eq!(far, expected);
    // burst: every deadline and the single poll are 65536.
    assert_eq!(
        plain(workload(2, "burst", 0).unwrap()),
        [
            ('A', 0, 65_536),
            ('A', 1, 65_536),
            ('A', 0, 65_536),
            ('T', 0, 65_536)
        ]
    );
    assert_eq!(
        workload(65_536, "burst", 0).unwrap().len(),
        65_536 + 16_384 + 1
    );
    for (ids, shape, cancel, error) in [
        (0, "near", 0, "invalid workload dimensions"),
        (65_537, "near", 0, "invalid workload dimensions"),
        (16, "near", 50, "invalid workload dimensions"),
        (16, "ring", 0, "unknown shape"),
    ] {
        assert_eq!(workload(ids, shape, cancel).map(|_| ()), Err(error));
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
