#![warn(missing_docs)]

//! hodei-declarative-extractors
//!
//! Fase 2 EPIC-15: Extractores Declarativos
//!
//! Sistema para crear y ejecutar reglas de análisis de código usando DSL YAML
//! y motor tree-sitter multi-lenguaje.

pub mod errors;
pub mod matcher;
pub mod rules;
pub mod tree_sitter;

pub use errors::{DeclarativeExtractorError, Result};
pub use matcher::PatternMatcher;
pub use rules::{Rule, RuleLoader, RuleSet};
pub use tree_sitter::{Language, MultiLanguageParser, ParseError};

pub use hodei_ir::Fact;

/// Version del DSL declarativo
pub const DSL_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tree_sitter_basic_parsing() {
        let parser = MultiLanguageParser::new();

        // Test Python parsing
        let python_code = r#"
def test():
    return 42
"#;
        let result = parser.parse(Language::Python, python_code).await;
        assert!(result.is_ok(), "Should parse Python code");

        // Test JavaScript parsing
        let js_code = r#"
function test() {
    return 42;
}
"#;
        let result = parser.parse(Language::JavaScript, js_code).await;
        assert!(result.is_ok(), "Should parse JavaScript code");
    }

    #[test]
    fn test_language_support() {
        let languages = Language::all_languages();
        assert!(languages.len() >= 10, "Should support 10+ languages");

        // Verify key languages are supported
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::Java));
    }
}
