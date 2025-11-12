//! Basic test to verify hodei-dsl compiles

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_types() {
        // Test that basic types compile
        let _string = String::from("test");
        let _vec = Vec::new();
        let _hashmap = std::collections::HashMap::new();
    }

    #[test]
    fn test_option_and_result() {
        let value = Some(42);
        assert_eq!(value, Some(42));

        let result: Result<i32, &str> = Ok(10);
        assert!(result.is_ok());
    }
}
