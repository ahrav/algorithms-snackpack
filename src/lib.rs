#![doc = include_str!("../README.md")]

/// Number of stable topics in the initial curriculum catalog.
pub const TOPIC_COUNT: usize = 54;

#[cfg(test)]
mod tests {
    use super::TOPIC_COUNT;

    #[test]
    fn catalog_count_matches_curriculum() {
        let topic_headers = include_str!("../curriculum.toml")
            .lines()
            .filter(|line| *line == "[[topics]]")
            .count();

        assert_eq!(topic_headers, TOPIC_COUNT);
    }
}
