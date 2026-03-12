pub mod models;
pub mod split;
pub mod validation;

#[cfg(test)]
mod tests {
    #[test]
    fn core_crate_loads() {
        assert_eq!(2 + 2, 4);
    }
}
