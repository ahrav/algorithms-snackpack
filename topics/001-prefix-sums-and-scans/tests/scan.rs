//! Contract, model-agreement, boundary, and recurrence tests for prefix scans.

use prefix_sums_and_scans::{
    ScanError, blocked_inclusive, linear_exclusive, linear_inclusive, parallel_inclusive,
    reference_exclusive, reference_inclusive,
};

fn visit_vectors<F>(len: usize, current: &mut Vec<i64>, visit: &mut F)
where
    F: FnMut(&[i64]),
{
    if current.len() == len {
        visit(current);
        return;
    }

    for value in -2_i64..=2 {
        current.push(value);
        visit_vectors(len, current, visit);
        current.pop();
    }
}

#[test]
fn exact_example_matches_inclusive_and_exclusive_contracts() {
    let input = [3, 1, 4, 1, 5];
    let inclusive = vec![3, 4, 8, 9, 14];
    let exclusive = vec![0, 3, 4, 8, 9];

    assert_eq!(reference_inclusive(&input), inclusive);
    assert_eq!(linear_inclusive(&input), inclusive);
    assert_eq!(
        blocked_inclusive(&input, 2),
        Ok(inclusive.clone()),
        "the partial final block must receive the first two block totals"
    );
    assert_eq!(parallel_inclusive(&input, 3), Ok(inclusive));
    assert_eq!(reference_exclusive(&input, 0), exclusive);
    assert_eq!(linear_exclusive(&input, 0), exclusive);
}

#[test]
fn empty_and_singleton_boundaries_match_the_models() {
    let empty = [];
    assert!(reference_inclusive(&empty).is_empty());
    assert!(reference_exclusive(&empty, 91).is_empty());
    assert!(linear_inclusive(&empty).is_empty());
    assert!(linear_exclusive(&empty, 91).is_empty());
    assert_eq!(blocked_inclusive(&empty, 4), Ok(Vec::new()));
    assert_eq!(parallel_inclusive(&empty, 4), Ok(Vec::new()));

    let singleton = [i64::MIN];
    assert_eq!(reference_inclusive(&singleton), vec![i64::MIN]);
    assert_eq!(reference_exclusive(&singleton, 0), vec![0]);
    assert_eq!(linear_inclusive(&singleton), vec![i64::MIN]);
    assert_eq!(linear_exclusive(&singleton, 0), vec![0]);
    assert_eq!(blocked_inclusive(&singleton, 8), Ok(vec![i64::MIN]));
    assert_eq!(parallel_inclusive(&singleton, 8), Ok(vec![i64::MIN]));
}

#[test]
fn every_candidate_uses_wrapping_add_at_overflow_boundaries() {
    let input = [i64::MAX, 1, -1, i64::MIN];
    let inclusive = vec![i64::MAX, i64::MIN, i64::MAX, -1];
    let exclusive = vec![0, i64::MAX, i64::MIN, i64::MAX];

    assert_eq!(reference_inclusive(&input), inclusive);
    assert_eq!(linear_inclusive(&input), inclusive);
    assert_eq!(blocked_inclusive(&input, 2), Ok(inclusive.clone()));
    assert_eq!(parallel_inclusive(&input, 3), Ok(inclusive));
    assert_eq!(reference_exclusive(&input, 0), exclusive);
    assert_eq!(linear_exclusive(&input, 0), exclusive);
}

#[test]
fn exclusive_nonzero_init_is_included_and_wraps() {
    let input = [1, -1, 2];
    let init = i64::MAX;
    let expected = vec![i64::MAX, i64::MIN, i64::MAX];

    assert_eq!(reference_exclusive(&input, init), expected);
    assert_eq!(linear_exclusive(&input, init), expected);
}

#[test]
fn zero_configuration_is_rejected_without_changing_input() {
    let input = vec![8, -3, 5];
    let before = input.clone();

    assert_eq!(blocked_inclusive(&input, 0), Err(ScanError::ZeroBlockSize));
    assert_eq!(input, before);
    assert_eq!(parallel_inclusive(&input, 0), Err(ScanError::ZeroWorkers));
    assert_eq!(input, before);

    assert_eq!(blocked_inclusive(&[], 0), Err(ScanError::ZeroBlockSize));
    assert_eq!(parallel_inclusive(&[], 0), Err(ScanError::ZeroWorkers));
}

#[test]
fn partial_final_block_receives_the_complete_preceding_offset() {
    let input = [1, 2, 3, 4, 5, 6, 7];

    assert_eq!(
        blocked_inclusive(&input, 3),
        Ok(vec![1, 3, 6, 10, 15, 21, 28])
    );
}

#[test]
fn exhaustive_model_agreement_for_lengths_zero_through_seven() {
    for len in 0..=7 {
        visit_vectors(len, &mut Vec::with_capacity(len), &mut |input| {
            let inclusive = reference_inclusive(input);
            let exclusive = reference_exclusive(input, 0);

            assert_eq!(linear_inclusive(input), inclusive, "input={input:?}");
            assert_eq!(linear_exclusive(input, 0), exclusive, "input={input:?}");

            // One effective worker covers the public candidate without making
            // nearly 100,000 operating-system thread groups. The dedicated
            // cross-chunk test below covers the scoped-thread composition path.
            assert_eq!(
                parallel_inclusive(input, 1),
                Ok(inclusive.clone()),
                "input={input:?}, workers=1"
            );

            let largest_block = input.len().saturating_add(1).max(1);
            for block_size in 1..=largest_block {
                assert_eq!(
                    blocked_inclusive(input, block_size),
                    Ok(inclusive.clone()),
                    "input={input:?}, block_size={block_size}"
                );
            }
        });
    }
}

#[test]
fn parallel_multi_chunk_output_matches_model() {
    let mut input = Vec::with_capacity(257);
    for index in 0_i64..257 {
        let value = match index % 5 {
            0 => i64::MAX,
            1 => 1,
            2 => -17,
            3 => index,
            _ => -index,
        };
        input.push(value);
    }

    assert_eq!(
        parallel_inclusive(&input, 4),
        Ok(reference_inclusive(&input))
    );
}

#[test]
fn inclusive_recurrence_holds_at_every_output_boundary() {
    let input = [i64::MAX, 1, 9, -12, i64::MIN, 7, -7];
    let scans = [
        linear_inclusive(&input),
        blocked_inclusive(&input, 3).expect("nonzero block size is valid"),
        parallel_inclusive(&input, 4).expect("nonzero worker count is valid"),
    ];

    for scan in scans {
        assert_eq!(scan.first(), input.first());
        for (output_pair, &next_input) in scan.windows(2).zip(&input[1..]) {
            assert_eq!(
                output_pair[1],
                output_pair[0].wrapping_add(next_input),
                "each output must extend the preceding prefix by one input"
            );
        }
    }
}

#[test]
fn workers_greater_than_input_length_preserve_results() {
    let input = [11, -4, 8];

    assert_eq!(
        parallel_inclusive(&input, 64),
        Ok(reference_inclusive(&input))
    );
}
