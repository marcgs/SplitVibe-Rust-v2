/// Validate a group name. Returns an error message if invalid.
pub fn validate_group_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Group name cannot be empty");
    }
    if trimmed.len() > 100 {
        return Err("Group name must be 100 characters or less");
    }
    Ok(())
}

/// Validate an expense title.
pub fn validate_expense_title(title: &str) -> Result<(), &'static str> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("Expense description cannot be empty");
    }
    if trimmed.len() > 200 {
        return Err("Expense description must be 200 characters or less");
    }
    Ok(())
}

/// Validate an expense amount string. Returns the parsed Decimal or error.
pub fn validate_expense_amount(amount_str: &str) -> Result<rust_decimal::Decimal, &'static str> {
    let amount: rust_decimal::Decimal = amount_str
        .trim()
        .parse()
        .map_err(|_| "Invalid amount format")?;
    if amount <= rust_decimal::Decimal::ZERO {
        return Err("Amount must be greater than zero");
    }
    Ok(amount.round_dp(2))
}

/// Validate that at least one member is selected for splitting.
pub fn validate_split_members(members: &[String]) -> Result<(), &'static str> {
    if members.is_empty() {
        return Err("At least one member must be selected for splitting");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_name_is_invalid() {
        assert!(validate_group_name("").is_err());
        assert!(validate_group_name("   ").is_err());
    }

    #[test]
    fn test_valid_name() {
        assert!(validate_group_name("Trip to Paris").is_ok());
    }

    #[test]
    fn test_too_long_name_is_invalid() {
        let long_name = "a".repeat(101);
        assert!(validate_group_name(&long_name).is_err());
    }

    #[test]
    fn test_max_length_name_is_valid() {
        let name = "a".repeat(100);
        assert!(validate_group_name(&name).is_ok());
    }

    #[test]
    fn test_expense_title_empty() {
        assert!(validate_expense_title("").is_err());
        assert!(validate_expense_title("   ").is_err());
    }

    #[test]
    fn test_expense_title_valid() {
        assert!(validate_expense_title("Dinner").is_ok());
    }

    #[test]
    fn test_expense_amount_zero() {
        assert!(validate_expense_amount("0").is_err());
    }

    #[test]
    fn test_expense_amount_negative() {
        assert!(validate_expense_amount("-5.00").is_err());
    }

    #[test]
    fn test_expense_amount_valid() {
        let amount = validate_expense_amount("90.00").unwrap();
        assert_eq!(amount, rust_decimal::Decimal::new(9000, 2));
    }

    #[test]
    fn test_expense_amount_invalid_format() {
        assert!(validate_expense_amount("abc").is_err());
    }

    #[test]
    fn test_split_members_empty() {
        assert!(validate_split_members(&[]).is_err());
    }

    #[test]
    fn test_split_members_valid() {
        assert!(validate_split_members(&["alice".into()]).is_ok());
    }
}
