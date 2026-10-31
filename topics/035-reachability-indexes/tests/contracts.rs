//! Bounded independent-oracle contract checks.
use topic035::{Closure, Dag, Intervals, Scratch, search};

fn compare(edges: Vec<Vec<usize>>) {
    let g = Dag::new(edges).unwrap();
    let c = Closure::new(&g, 64 * 1024 * 1024).unwrap();
    let f = Intervals::new(&g);
    let mut s = Scratch::new(g.len());
    for u in 0..g.len() {
        for (v, expected) in g.oracle(u).into_iter().enumerate() {
            assert_eq!(search(&g, &mut s, u, v, None), expected, "dfs {u},{v}");
            assert_eq!(
                search(&g, &mut s, u, v, Some(&f)),
                expected,
                "filter {u},{v}"
            );
            assert_eq!(c.reaches(u, v), expected, "closure {u},{v}");
            if expected {
                assert!(f.may_reach(u, v));
            }
        }
    }
}

#[test]
fn exhaustive_ordered_dags_and_reversed_ids() {
    // All 1024 edge subsets on five ordered vertices, then reverse every ID.
    for mask in 0..1024 {
        let mut edges = vec![vec![]; 5];
        let mut bit = 0;
        for (u, row) in edges.iter_mut().enumerate() {
            for v in u + 1..5 {
                if mask & (1 << bit) != 0 {
                    row.push(v);
                }
                bit += 1;
            }
        }
        let mut reversed = vec![vec![]; 5];
        for (u, row) in edges.iter().enumerate() {
            for &v in row {
                reversed[4 - u].push(4 - v);
            }
        }
        compare(edges);
        compare(reversed);
    }
}

#[test]
fn boundaries_cross_edges_duplicates_and_deep_chain() {
    compare(vec![]);
    compare(vec![vec![]]);
    compare(vec![
        vec![1, 2, 1],
        vec![3],
        vec![3, 4],
        vec![5],
        vec![5],
        vec![],
        vec![],
    ]);
    // Disconnected vertex.
    compare(vec![vec![1, 3], vec![], vec![], vec![]]);
    // Pair (2, 3) is an interval-filter false positive.
    let edges = vec![vec![4], vec![3], vec![4], vec![], vec![]];
    let g = Dag::new(edges.clone()).unwrap();
    assert!(Intervals::new(&g).may_reach(2, 3) && !g.oracle(2)[3]);
    compare(edges);
    for n in [63, 64, 65, 129] {
        compare(
            (0..n)
                .map(|u| if u + 1 < n { vec![u + 1] } else { vec![] })
                .collect(),
        );
    }
    let n = 20_000;
    let g = Dag::new(
        (0..n)
            .map(|u| if u + 1 < n { vec![u + 1] } else { vec![] })
            .collect(),
    )
    .unwrap();
    let f = Intervals::new(&g);
    let mut s = Scratch::new(n);
    assert!(search(&g, &mut s, 0, n - 1, Some(&f)));
    assert!(search(&g, &mut s, 0, n - 1, None));
    assert!(!search(&g, &mut s, n - 1, 0, Some(&f)));
}

#[test]
fn invalid_graph_and_budget_are_explicit() {
    assert_eq!(Dag::new(vec![vec![1]]).err(), Some("endpoint"));
    assert_eq!(Dag::new(vec![vec![0]]).err(), Some("cycle"));
    assert_eq!(Dag::new(vec![vec![1], vec![0]]).err(), Some("cycle"));
    let g = Dag::new(vec![vec![1], vec![]]).unwrap();
    assert_eq!(Closure::new(&g, 15).err(), Some("budget"));
    assert_eq!(Closure::new(&g, 16).unwrap().bytes(), 16);
}

#[test]
#[should_panic(expected = "assertion failed")]
fn filter_built_for_another_graph_is_rejected() {
    let g = Dag::new(vec![vec![1], vec![]]).unwrap();
    let f = Intervals::new(&Dag::new(vec![vec![], vec![], vec![]]).unwrap());
    search(&g, &mut Scratch::new(g.len()), 0, 1, Some(&f));
}

#[test]
#[ignore = "allocates a 16 GiB edge row"]
fn indegree_counter_accepts_two_to_the_thirty_one_duplicate_edges() {
    let g = Dag::new(vec![vec![1; 1 << 31], vec![]]).unwrap();
    assert_eq!(g.len(), 2);
}
