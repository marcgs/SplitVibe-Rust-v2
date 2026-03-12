use rust_decimal::Decimal;

/// Result of splitting an amount equally among participants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitResult {
    /// (participant name/id, split amount) — sorted alphabetically by name.
    pub shares: Vec<(String, Decimal)>,
}

/// Split `total` equally among `participants` (sorted alphabetically).
/// Remainder cents are distributed to the first participants alphabetically.
///
/// All amounts are rounded to 2 decimal places.
pub fn split_equal(total: Decimal, mut participants: Vec<String>) -> SplitResult {
    assert!(
        !participants.is_empty(),
        "Cannot split among zero participants"
    );
    assert!(total > Decimal::ZERO, "Total must be positive");

    participants.sort();

    let count = Decimal::from(participants.len() as u64);
    let base = (total / count).round_dp(2);

    // Calculate remainder: total - (base * count)
    let remainder = total - base * count;
    // remainder in cents
    let remainder_cents = (remainder * Decimal::from(100)).round_dp(0);
    let extra_count = remainder_cents
        .abs()
        .to_string()
        .parse::<usize>()
        .unwrap_or(0);

    let shares: Vec<(String, Decimal)> = participants
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let amount = if remainder_cents > Decimal::ZERO && i < extra_count {
                base + Decimal::new(1, 2) // +0.01
            } else if remainder_cents < Decimal::ZERO && i < extra_count {
                base - Decimal::new(1, 2) // -0.01
            } else {
                base
            };
            (name, amount)
        })
        .collect();

    SplitResult { shares }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_even_split() {
        let result = split_equal(
            dec!(90.00),
            vec!["Alice".into(), "Bob".into(), "Charlie".into()],
        );
        assert_eq!(result.shares.len(), 3);
        assert_eq!(result.shares[0], ("Alice".into(), dec!(30.00)));
        assert_eq!(result.shares[1], ("Bob".into(), dec!(30.00)));
        assert_eq!(result.shares[2], ("Charlie".into(), dec!(30.00)));
    }

    #[test]
    fn test_remainder_distribution() {
        // $100 / 3 = $33.333... → base = $33.33, remainder = $0.01
        // Extra cent goes to first alphabetically (Alice)
        let result = split_equal(
            dec!(100.00),
            vec!["Charlie".into(), "Alice".into(), "Bob".into()],
        );
        assert_eq!(result.shares[0], ("Alice".into(), dec!(33.34)));
        assert_eq!(result.shares[1], ("Bob".into(), dec!(33.33)));
        assert_eq!(result.shares[2], ("Charlie".into(), dec!(33.33)));

        // Verify they sum to total
        let sum: Decimal = result.shares.iter().map(|(_, a)| a).sum();
        assert_eq!(sum, dec!(100.00));
    }

    #[test]
    fn test_two_way_split() {
        let result = split_equal(dec!(10.00), vec!["Alice".into(), "Bob".into()]);
        assert_eq!(result.shares[0], ("Alice".into(), dec!(5.00)));
        assert_eq!(result.shares[1], ("Bob".into(), dec!(5.00)));
    }

    #[test]
    fn test_single_participant() {
        let result = split_equal(dec!(50.00), vec!["Alice".into()]);
        assert_eq!(result.shares[0], ("Alice".into(), dec!(50.00)));
    }

    #[test]
    fn test_sum_always_equals_total() {
        // Test various amounts that produce remainders
        for total in [dec!(10.00), dec!(7.00), dec!(1.00), dec!(99.99), dec!(0.01)] {
            let result = split_equal(total, vec!["A".into(), "B".into(), "C".into()]);
            let sum: Decimal = result.shares.iter().map(|(_, a)| a).sum();
            assert_eq!(sum, total, "Sum mismatch for total {}", total);
        }
    }

    #[test]
    fn test_alphabetical_ordering() {
        let result = split_equal(
            dec!(90.00),
            vec!["Zara".into(), "Alice".into(), "Mike".into()],
        );
        assert_eq!(result.shares[0].0, "Alice");
        assert_eq!(result.shares[1].0, "Mike");
        assert_eq!(result.shares[2].0, "Zara");
    }
}
