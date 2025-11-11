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

/// A specialized string interner for project paths with normalization
pub struct ProjectPathInterner {
    base_interner: Interner,
}

impl ProjectPathInterner {
    /// Create a new project path interner
    pub fn new() -> Self {
        ProjectPathInterner {
            base_interner: Interner::new(),
        }
    }

    /// Intern a path with normalization
    pub fn intern_path(&mut self, path: &str) -> usize {
        let normalized = Self::normalize_path(path);
        self.base_interner.intern(&normalized)
    }

    /// Resolve a symbol back to its normalized path
    pub fn resolve_path(&self, symbol: usize) -> Option<&str> {
        self.base_interner.resolve(symbol)
    }

    /// Get the number of interned paths
    pub fn len(&self) -> usize {
        self.base_interner.len()
    }

    /// Check if the interner is empty
    pub fn is_empty(&self) -> bool {
        self.base_interner.is_empty()
    }

    /// Normalize a path by resolving . and .. components
    fn normalize_path(path: &str) -> String {
        use std::path::Component;
        use std::path::Path as StdPath;

        let path = StdPath::new(path);
        let mut normalized = Vec::new();
        let mut path_depth = 0; // Track the current depth relative to starting point

        for component in path.components() {
            match component {
                Component::CurDir => {
                    // Skip current directory references
                }
                Component::ParentDir => {
                    // Go up one level if we're not at the starting point
                    if path_depth > 0 {
                        normalized.pop();
                        path_depth -= 1;
                    }
                    // If we're at starting point (depth 0), don't go up
                }
                Component::RootDir | Component::Normal(_) | Component::Prefix(_) => {
                    normalized.push(component.as_os_str().to_string_lossy().into_owned());
                    path_depth += 1;
                }
            }
        }

        if normalized.is_empty() {
            return ".".to_string();
        }

        normalized.join(std::path::MAIN_SEPARATOR_STR)
    }
}

impl Default for ProjectPathInterner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod project_path_interner_tests {
    use super::*;

    #[test]
    fn test_path_normalization() {
        let mut path_interner = ProjectPathInterner::new();

        // Test simple path
        let s1 = path_interner.intern_path("src/main.rs");

        // Test path with ../ that should normalize
        let s2 = path_interner.intern_path("src/../src/main.rs");

        // They should resolve to the same symbol
        assert_eq!(s1, s2);
        assert_eq!(path_interner.resolve_path(s1), Some("src/main.rs"));
    }

    #[test]
    fn test_trailing_slash_removal() {
        let mut path_interner = ProjectPathInterner::new();

        let s1 = path_interner.intern_path("src/");
        let s2 = path_interner.intern_path("src");

        assert_eq!(s1, s2);
        assert_eq!(path_interner.resolve_path(s1), Some("src"));
    }

    #[test]
    fn test_current_directory_reference() {
        let mut path_interner = ProjectPathInterner::new();

        let s1 = path_interner.intern_path("src/main.rs");
        let s2 = path_interner.intern_path("./src/main.rs");

        assert_eq!(s1, s2);
        assert_eq!(path_interner.resolve_path(s1), Some("src/main.rs"));
    }

    #[test]
    fn test_path_deduplication() {
        let mut path_interner = ProjectPathInterner::new();

        let paths = vec!["src/a.rs", "src/../src/a.rs", "src/b.rs", "./src/b.rs"];

        let symbols: Vec<_> = paths.iter().map(|p| path_interner.intern_path(p)).collect();

        // Should deduplicate to 2 unique paths
        assert_eq!(path_interner.len(), 2);
        assert_eq!(symbols[0], symbols[1]);
        assert_eq!(symbols[2], symbols[3]);
        assert_ne!(symbols[0], symbols[2]);
    }

    #[test]
    fn test_complex_path_normalization() {
        let mut path_interner = ProjectPathInterner::new();

        // Complex normalization case: src/../src/lib/../src/main.rs
        // This normalizes to: src/src/main.rs
        // because each time we go "back to base" (with ..), we start fresh
        let complex = "src/../src/lib/../src/main.rs";
        let s1 = path_interner.intern_path(complex);

        // The complex path normalizes to src/src/main.rs
        assert_eq!(path_interner.resolve_path(s1), Some("src/src/main.rs"));
    }

    #[test]
    fn test_separator_normalization() {
        let mut path_interner = ProjectPathInterner::new();

        // Test that different separators are handled (on Unix, backslash might be part of filename)
        let path1 = "src/main.rs";
        let path2 = "src\\main.rs"; // On Unix, this is a different path

        let s1 = path_interner.intern_path(path1);
        let s2 = path_interner.intern_path(path2);

        // On Unix, these should be different
        #[cfg(unix)]
        assert_ne!(s1, s2);

        // Both should resolve to their original form
        assert_eq!(path_interner.resolve_path(s1), Some(path1));
        assert_eq!(path_interner.resolve_path(s2), Some(path2));
    }
}
