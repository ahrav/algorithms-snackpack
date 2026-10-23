//! Public API boundaries, including FIFO order after many wraps.
use algorithms_ring::Ring;
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
