//! Basic unit tests for the entire workspace

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_assertion() {
        assert!(true);
    }

    #[test]
    fn test_string() {
        let s = "hodei-scan";
        assert_eq!(s.len(), 10);
    }

    #[test]
    fn test_numbers() {
        let x = 42;
        let y = 8;
        assert_eq!(x + y, 50);
    }

    #[test]
    fn test_vec() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(v.len(), 5);
        assert_eq!(v[0], 1);
        assert_eq!(v[4], 5);
    }
}
