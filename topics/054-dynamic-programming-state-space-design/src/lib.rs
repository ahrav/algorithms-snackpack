//! Exact score-only 0/1 knapsack over a bounded teaching domain.

/// One independently selectable item, with integer weight and value.
#[derive(Clone, Copy, Debug)]
pub struct Item {
    /// Budget consumed when selected.
    pub weight: u16,
    /// Score contributed when selected.
    pub value: u16,
}

/// Storage or enumeration strategy for the same optimization problem.
#[derive(Clone, Copy, Debug)]
pub enum Strategy {
    /// Enumerate include and exclude choices.
    Exhaustive,
    /// Retain a capacity row for every item prefix.
    Table,
    /// Update one capacity row in descending order.
    Rolling,
    /// Track the minimum weight for each attainable total value.
    Value,
    /// Track the nondominated exact weight-value pairs.
    Frontier,
}

/// Return the maximum value attainable within the capacity.
/// Each item can be selected once, including items of weight zero.
/// The empty subset is legal. The return value is a score.
///
/// # Panics
///
/// Panics when the input contains more than 20 items. The lab bounds this
/// domain so every strategy, including exhaustive enumeration, can run.
#[must_use]
pub fn solve(strategy: Strategy, items: &[Item], capacity: u16) -> u32 {
    assert!(items.len() <= 20, "this lab accepts at most 20 items");
    match strategy {
        Strategy::Exhaustive => exhaustive(items, capacity),
        Strategy::Table => table(items, usize::from(capacity)),
        Strategy::Rolling => rolling(items, usize::from(capacity)),
        Strategy::Value => value_indexed(items, u32::from(capacity)),
        Strategy::Frontier => frontier(items, u32::from(capacity)),
    }
}

fn exhaustive(items: &[Item], remaining: u16) -> u32 {
    let Some((item, rest)) = items.split_first() else {
        return 0;
    };
    let skip = exhaustive(rest, remaining);
    match remaining.checked_sub(item.weight) {
        Some(next) => skip.max(u32::from(item.value) + exhaustive(rest, next)),
        None => skip,
    }
}

fn table(items: &[Item], capacity: usize) -> u32 {
    let width = capacity + 1;
    let mut cells = vec![0; (items.len() + 1) * width];
    for (i, item) in items.iter().enumerate() {
        let (prior_rows, later_rows) = cells.split_at_mut((i + 1) * width);
        let prior = &prior_rows[i * width..];
        let next = &mut later_rows[..width];
        let weight = usize::from(item.weight);
        next.copy_from_slice(prior);
        for (dest, src) in next.iter_mut().skip(weight).zip(prior) {
            *dest = (*dest).max(*src + u32::from(item.value));
        }
    }
    cells[items.len() * width + capacity]
}

fn rolling(items: &[Item], capacity: usize) -> u32 {
    let mut row = vec![0; capacity + 1];
    for item in items {
        let weight = usize::from(item.weight);
        for c in (weight..=capacity).rev() {
            row[c] = row[c].max(row[c - weight] + u32::from(item.value));
        }
    }
    row[capacity]
}

fn value_indexed(items: &[Item], capacity: u32) -> u32 {
    let total: usize = items.iter().map(|item| usize::from(item.value)).sum();
    let unreachable = capacity + 1;
    let mut min_weight = vec![unreachable; total + 1];
    min_weight[0] = 0;
    let mut prefix_value = 0;
    for item in items {
        let value = usize::from(item.value);
        prefix_value += value;
        for score in (value..=prefix_value).rev() {
            let take = min_weight[score - value] + u32::from(item.weight);
            min_weight[score] = min_weight[score].min(take);
        }
    }
    u32::try_from(
        min_weight
            .iter()
            .rposition(|&weight| weight <= capacity)
            .unwrap_or(0),
    )
    .expect("sum of at most 20 u16 values fits u32")
}

fn frontier(items: &[Item], capacity: u32) -> u32 {
    let mut pairs = vec![(0, 0)];
    let mut next = Vec::new();
    for item in items {
        let mut skip = pairs.iter().copied().peekable();
        let mut take = pairs
            .iter()
            .map_while(|&(weight, value)| {
                let shifted = weight + u32::from(item.weight);
                (shifted <= capacity).then_some((shifted, value + u32::from(item.value)))
            })
            .peekable();
        while let Some(pair) = match (skip.peek(), take.peek()) {
            (Some(a), Some(b)) if a.0 < b.0 || (a.0 == b.0 && a.1 >= b.1) => skip.next(),
            (_, Some(_)) => take.next(),
            (_, None) => skip.next(),
        } {
            // For equal weights the merge emits the larger value first.
            if next.last().is_none_or(|&(_, best)| pair.1 > best) {
                next.push(pair);
            }
        }
        std::mem::swap(&mut pairs, &mut next);
        next.clear();
    }
    pairs.last().expect("the empty solution fits").1
}
