//! Public API boundaries, including FIFO order after many wraps.
use algorithms_ring::Ring;
use std::collections::VecDeque;
#[test]
fn public_fifo_and_split_spans() {
    for capacity in [0, 1, 3, 4, 17] {
        let mut q = Ring::<String, 2>::new(capacity);
        for lap in 0..33 {
            for i in 0..capacity {
                q.push(format!("{lap}:{i}")).unwrap();
            }
            assert_eq!(q.push("rejected".into()), Err("rejected".into()));
            let (a, b) = q.as_slices();
            assert_eq!(a.len() + b.len(), capacity);
            let visible = a
                .iter()
                .chain(b)
                .map(|v| v.as_deref().unwrap())
                .collect::<Vec<_>>();
            let expected = (0..capacity)
                .map(|i| format!("{lap}:{i}"))
                .collect::<Vec<_>>();
            assert_eq!(visible, expected);
            for value in expected {
                assert_eq!(q.pop(), Some(value));
            }
            assert!(q.is_empty());
            assert_eq!(q.pop(), None);
        }
    }
}
/// The push/pop schedule exercises wrapped, partially filled rings at capacities 3, 4 and 17.
fn split_spans_follow_length<const MODE: u8>() {
    for capacity in [0, 1, 3, 4, 17] {
        let mut q = Ring::<u64, MODE>::new(capacity);
        assert_eq!(q.capacity(), capacity);
        assert_eq!(q.as_slices(), (&[][..], &[][..]));
        let mut oracle = VecDeque::new();
        let mut next = 0_u64;
        let mut wrapped_partial = 0;
        for round in 0..64 {
            for _ in 0..round % (capacity + 2) {
                assert_eq!(
                    q.push(next),
                    if oracle.len() < capacity {
                        Ok(())
                    } else {
                        Err(next)
                    }
                );
                if oracle.len() < capacity {
                    oracle.push_back(next);
                }
                next += 1;
            }
            for _ in 0..(round / 2) % (capacity + 1) {
                assert_eq!(q.pop(), oracle.pop_front());
            }
            let (a, b) = q.as_slices();
            assert_eq!(
                a.len() + b.len(),
                q.len(),
                "mode {MODE}, capacity {capacity}, round {round}"
            );
            assert_eq!(q.len(), oracle.len());
            let visible = a.iter().chain(b).map(|v| v.unwrap()).collect::<Vec<_>>();
            assert_eq!(visible, oracle.iter().copied().collect::<Vec<_>>());
            if !b.is_empty() && q.len() < capacity {
                wrapped_partial += 1;
            }
        }
        assert!(
            capacity < 2 || wrapped_partial > 0,
            "mode {MODE}, capacity {capacity}: no wrapped partial span observed"
        );
    }
}
#[test]
fn split_spans_follow_length_in_every_mode() {
    split_spans_follow_length::<0>();
    split_spans_follow_length::<1>();
    split_spans_follow_length::<2>();
}
