//! String interning module for memory optimization

use std::collections::HashMap;
use string_interner::StringInterner;

/// A simple string interning implementation
pub struct Interner {
    interner: StringInterner<usize>,
    cache: HashMap<String, usize>,
}

impl Interner {
    /// Create a new string interner
    pub fn new() -> Self {
        Interner {
            interner: StringInterner::new(),
            cache: HashMap::new(),
        }
    }

    /// Intern a string and return a symbol
    pub fn intern(&mut self, value: &str) -> usize {
        if let Some(&symbol) = self.cache.get(value) {
            return symbol;
        }

        let symbol = self.interner.get_or_intern(value);
        self.cache.insert(value.to_string(), symbol);
        symbol
    }

    /// Resolve a symbol back to its string
    pub fn resolve(&self, symbol: usize) -> Option<&str> {
        self.interner.resolve(symbol)
    }

    /// Get the number of interned strings
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_string_interner_basic() {
        let mut interner = Interner::new();
        let s1 = interner.intern("src/main.rs");
        let s2 = interner.intern("src/main.rs");
        let s3 = interner.intern("src/lib.rs");

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_string_interner_resolve() {
        let mut interner = Interner::new();
        let symbol = interner.intern("test");
        assert_eq!(interner.resolve(symbol), Some("test"));
    }

    #[test]
    fn test_memory_reduction() {
        let paths: Vec<String> = vec!["src/main.rs".to_string(); 10000];
        let original_size = calculate_size(&paths);

        let mut interner = Interner::new();
        let _interned_symbols: Vec<_> = paths.iter().map(|p| interner.intern(p.as_str())).collect();

        assert_eq!(interner.len(), 1);

        let interner_size = calculate_interner_size(&interner);
        let reduction = (original_size as f64 - interner_size as f64) / original_size as f64;

        assert!(
            reduction > 0.6,
            "Should reduce memory by 60%, got {:.2}%",
            reduction * 100.0
        );
    }

    #[test]
    fn test_string_comparison_performance() {
        let mut interner = Interner::new();
        let strings: Vec<String> = (0..1000)
            .map(|i| format!("src/file{}.rs", i % 100))
            .collect();

        let symbols: Vec<_> = strings
            .iter()
            .map(|s| interner.intern(s.as_str()))
            .collect();

        let start = Instant::now();
        for i in 0..symbols.len() {
            for j in (i + 1)..symbols.len() {
                let _ = symbols[i] == symbols[j];
            }
        }
        let symbol_time = start.elapsed();

        let start = Instant::now();
        for i in 0..strings.len() {
            for j in (i + 1)..strings.len() {
                let _ = strings[i] == strings[j];
            }
        }
        let string_time = start.elapsed();

        println!("Symbol comparisons: {:?}", symbol_time);
        println!("String comparisons: {:?}", string_time);

        let speedup = string_time.as_nanos() as f64 / symbol_time.as_nanos() as f64;
        assert!(
            speedup > 1.0,
            "Symbols should be faster than strings, got {:.2}x",
            speedup
        );
    }

    #[test]
    fn test_interner_len() {
        let mut interner = Interner::new();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());

        interner.intern("first");
        assert_eq!(interner.len(), 1);
        assert!(!interner.is_empty());

        interner.intern("first");
        assert_eq!(interner.len(), 1);

        interner.intern("second");
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_interning_deduplication() {
        let mut interner = Interner::new();

        let symbols: Vec<usize> = vec![
            interner.intern("src/a.rs"),
            interner.intern("src/b.rs"),
            interner.intern("src/a.rs"),
        ];

        assert_eq!(interner.len(), 2);
        assert_eq!(symbols[0], symbols[2]);
        assert_ne!(symbols[0], symbols[1]);
    }

    fn calculate_size(strings: &[String]) -> usize {
        strings
            .iter()
            .map(|s| s.len() + std::mem::size_of::<String>())
            .sum()
    }

    fn calculate_interner_size(interner: &Interner) -> usize {
        let unique_strings = interner.len();
        unique_strings * 64
    }
}
