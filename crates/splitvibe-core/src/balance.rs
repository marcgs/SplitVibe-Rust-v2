use rust_decimal::Decimal;
use std::collections::HashMap;

/// A simplified debt: `from` owes `to` the given `amount`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Debt {
    pub from: String,
    pub to: String,
    pub amount: Decimal,
}

/// An expense entry for balance calculation.
#[derive(Debug, Clone)]
pub struct ExpenseEntry {
    /// User ID of who paid.
    pub payer: String,
    /// (user_id, amount) for each person's share.
    pub splits: Vec<(String, Decimal)>,
}

/// A settlement entry: payer paid payee to settle a debt.
#[derive(Debug, Clone)]
pub struct SettlementEntry {
    /// User ID of who made the payment.
    pub payer: String,
    /// User ID of who received the payment.
    pub payee: String,
    /// Amount paid.
    pub amount: Decimal,
}

/// Calculate simplified debts from expenses and settlements.
///
/// Uses a greedy min-cash-flow algorithm:
/// 1. Compute each person's net balance (paid - owed) from expenses.
/// 2. Apply settlements (payer→payee transfers reduce net balances).
/// 3. Repeatedly match the person who owes the most with the person owed the most.
///
/// Returns debts sorted by (from, to) for deterministic output.
pub fn calculate_debts(entries: &[ExpenseEntry]) -> Vec<Debt> {
    calculate_debts_with_settlements(entries, &[])
}

/// Calculate debts accounting for both expenses and settlements.
pub fn calculate_debts_with_settlements(
    entries: &[ExpenseEntry],
    settlements: &[SettlementEntry],
) -> Vec<Debt> {
    if entries.is_empty() && settlements.is_empty() {
        return Vec::new();
    }

    // Step 1: compute net balances from expenses
    let mut balances: HashMap<String, Decimal> = HashMap::new();

    for entry in entries {
        let total_paid: Decimal = entry.splits.iter().map(|(_, a)| a).sum();
        *balances.entry(entry.payer.clone()).or_default() += total_paid;

        for (user_id, amount) in &entry.splits {
            *balances.entry(user_id.clone()).or_default() -= amount;
        }
    }

    // Step 2: apply settlements (payer pays payee → payer's balance goes up, payee's goes down)
    for s in settlements {
        *balances.entry(s.payer.clone()).or_default() += s.amount;
        *balances.entry(s.payee.clone()).or_default() -= s.amount;
    }

    // Step 2: greedy min-cash-flow
    let mut creditors: Vec<(String, Decimal)> = Vec::new();
    let mut debtors: Vec<(String, Decimal)> = Vec::new();

    for (user, balance) in &balances {
        if *balance > Decimal::ZERO {
            creditors.push((user.clone(), *balance));
        } else if *balance < Decimal::ZERO {
            debtors.push((user.clone(), balance.abs()));
        }
    }

    // Sort for deterministic output
    creditors.sort_by(|a, b| a.0.cmp(&b.0));
    debtors.sort_by(|a, b| a.0.cmp(&b.0));

    let mut debts = Vec::new();
    let mut ci = 0;
    let mut di = 0;

    while ci < creditors.len() && di < debtors.len() {
        let transfer = creditors[ci].1.min(debtors[di].1);
        if transfer > Decimal::ZERO {
            debts.push(Debt {
                from: debtors[di].0.clone(),
                to: creditors[ci].0.clone(),
                amount: transfer.round_dp(2),
            });
        }
        creditors[ci].1 -= transfer;
        debtors[di].1 -= transfer;

        if creditors[ci].1 == Decimal::ZERO {
            ci += 1;
        }
        if debtors[di].1 == Decimal::ZERO {
            di += 1;
        }
    }

    debts.sort_by(|a, b| (&a.from, &a.to).cmp(&(&b.from, &b.to)));
    debts
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_no_expenses_returns_no_debts() {
        let debts = calculate_debts(&[]);
        assert!(debts.is_empty());
    }

    #[test]
    fn test_single_expense_equal_split_three_ways() {
        // Alice pays $90, split among Alice/Bob/Charlie
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![
                ("Alice".into(), dec!(30.00)),
                ("Bob".into(), dec!(30.00)),
                ("Charlie".into(), dec!(30.00)),
            ],
        }];

        let debts = calculate_debts(&entries);
        assert_eq!(debts.len(), 2);
        assert_eq!(
            debts[0],
            Debt {
                from: "Bob".into(),
                to: "Alice".into(),
                amount: dec!(30.00),
            }
        );
        assert_eq!(
            debts[1],
            Debt {
                from: "Charlie".into(),
                to: "Alice".into(),
                amount: dec!(30.00),
            }
        );
    }

    #[test]
    fn test_alice_owed_total() {
        // Alice pays $90, split among Alice/Bob/Charlie
        // Alice's net: paid 90, owes 30 → +60
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![
                ("Alice".into(), dec!(30.00)),
                ("Bob".into(), dec!(30.00)),
                ("Charlie".into(), dec!(30.00)),
            ],
        }];

        let debts = calculate_debts(&entries);
        let total_owed_to_alice: Decimal = debts
            .iter()
            .filter(|d| d.to == "Alice")
            .map(|d| d.amount)
            .sum();
        assert_eq!(total_owed_to_alice, dec!(60.00));
    }

    #[test]
    fn test_multiple_expenses_simplified() {
        // Alice paid $90 split 3 ways → each owes $30
        // Bob paid $60 split 3 ways → each owes $20
        // Net: Alice: +90 - 30 - 20 = +40, Bob: +60 - 30 - 20 = +10, Charlie: -30 - 20 = -50
        // Simplified: Charlie owes Alice $40, Charlie owes Bob $10
        let entries = vec![
            ExpenseEntry {
                payer: "Alice".into(),
                splits: vec![
                    ("Alice".into(), dec!(30.00)),
                    ("Bob".into(), dec!(30.00)),
                    ("Charlie".into(), dec!(30.00)),
                ],
            },
            ExpenseEntry {
                payer: "Bob".into(),
                splits: vec![
                    ("Alice".into(), dec!(20.00)),
                    ("Bob".into(), dec!(20.00)),
                    ("Charlie".into(), dec!(20.00)),
                ],
            },
        ];

        let debts = calculate_debts(&entries);
        assert_eq!(debts.len(), 2);

        let charlie_to_alice = debts
            .iter()
            .find(|d| d.from == "Charlie" && d.to == "Alice");
        let charlie_to_bob = debts.iter().find(|d| d.from == "Charlie" && d.to == "Bob");

        assert_eq!(charlie_to_alice.unwrap().amount, dec!(40.00));
        assert_eq!(charlie_to_bob.unwrap().amount, dec!(10.00));
    }

    #[test]
    fn test_single_payer_single_member_no_debts() {
        // Alice pays $50, only Alice in split → no debts
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![("Alice".into(), dec!(50.00))],
        }];

        let debts = calculate_debts(&entries);
        assert!(debts.is_empty());
    }

    #[test]
    fn test_all_settled_up() {
        // Alice pays $30 split 2 ways ($15 each), Bob pays $30 split 2 ways ($15 each)
        // Net: Alice: +30 - 15 - 15 = 0, Bob: +30 - 15 - 15 = 0
        let entries = vec![
            ExpenseEntry {
                payer: "Alice".into(),
                splits: vec![("Alice".into(), dec!(15.00)), ("Bob".into(), dec!(15.00))],
            },
            ExpenseEntry {
                payer: "Bob".into(),
                splits: vec![("Alice".into(), dec!(15.00)), ("Bob".into(), dec!(15.00))],
            },
        ];

        let debts = calculate_debts(&entries);
        assert!(debts.is_empty());
    }

    #[test]
    fn test_settlement_clears_debt() {
        // Alice pays $90 split 3 ways → Bob owes $30, Charlie owes $30
        // Bob settles $30 with Alice → Bob no longer owes
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![
                ("Alice".into(), dec!(30.00)),
                ("Bob".into(), dec!(30.00)),
                ("Charlie".into(), dec!(30.00)),
            ],
        }];

        let settlements = vec![SettlementEntry {
            payer: "Bob".into(),
            payee: "Alice".into(),
            amount: dec!(30.00),
        }];

        let debts = calculate_debts_with_settlements(&entries, &settlements);
        assert_eq!(debts.len(), 1);
        assert_eq!(
            debts[0],
            Debt {
                from: "Charlie".into(),
                to: "Alice".into(),
                amount: dec!(30.00),
            }
        );
    }

    #[test]
    fn test_settlement_partial() {
        // Alice pays $90 split 3 ways → Bob owes $30
        // Bob settles $10 → Bob still owes $20
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![
                ("Alice".into(), dec!(30.00)),
                ("Bob".into(), dec!(30.00)),
                ("Charlie".into(), dec!(30.00)),
            ],
        }];

        let settlements = vec![SettlementEntry {
            payer: "Bob".into(),
            payee: "Alice".into(),
            amount: dec!(10.00),
        }];

        let debts = calculate_debts_with_settlements(&entries, &settlements);
        let bob_to_alice = debts.iter().find(|d| d.from == "Bob" && d.to == "Alice");
        assert_eq!(bob_to_alice.unwrap().amount, dec!(20.00));
    }

    #[test]
    fn test_full_settlement_clears_all() {
        // Alice pays $90 split 3 ways
        // Both Bob and Charlie settle $30 each → all settled
        let entries = vec![ExpenseEntry {
            payer: "Alice".into(),
            splits: vec![
                ("Alice".into(), dec!(30.00)),
                ("Bob".into(), dec!(30.00)),
                ("Charlie".into(), dec!(30.00)),
            ],
        }];

        let settlements = vec![
            SettlementEntry {
                payer: "Bob".into(),
                payee: "Alice".into(),
                amount: dec!(30.00),
            },
            SettlementEntry {
                payer: "Charlie".into(),
                payee: "Alice".into(),
                amount: dec!(30.00),
            },
        ];

        let debts = calculate_debts_with_settlements(&entries, &settlements);
        assert!(debts.is_empty());
    }
}
