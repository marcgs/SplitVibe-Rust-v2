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
}
