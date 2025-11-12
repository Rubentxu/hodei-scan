//! Basic test to verify the test infrastructure works

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        assert_eq!(2 + 2, 4);
        assert_eq!(5 * 5, 25);
    }

    #[test]
    fn test_string_operations() {
        let s = "hello";
        assert_eq!(s.len(), 5);
        assert!(s.contains("ell"));
    }

    #[test]
    fn test_option_handling() {
        let some_value = Some(42);
        let none_value: Option<i32> = None;

        assert!(some_value.is_some());
        assert!(none_value.is_none());
        assert_eq!(some_value.unwrap(), 42);
    }

    #[test]
    fn test_result_handling() {
        let ok_result: Result<i32, &str> = Ok(100);
        let err_result: Result<i32, &str> = Err("error");

        assert!(ok_result.is_ok());
        assert!(err_result.is_err());
        assert_eq!(ok_result.unwrap(), 100);
    }
}
