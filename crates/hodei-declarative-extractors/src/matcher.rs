//! Pattern matcher with metavariables
//!
//! US-15.3: Matcher de Patrones con Metavariables

use crate::errors::{DeclarativeExtractorError, Result};
use crate::rules::{Pattern, Rule, WhereClause};
use crate::tree_sitter::{ASTNode, Language};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metavariable binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetavarBinding {
    pub name: String,
    pub value: String,
    pub node: ASTNode,
}

/// Match result
#[derive(Debug, Clone)]
pub struct Match {
    pub pattern: Pattern,
    pub start_position: usize,
    pub end_position: usize,
    pub matched_text: String,
    pub metavars: Vec<MetavarBinding>,
}

impl Match {
    /// Create a new match
    pub fn new(
        pattern: Pattern,
        start: usize,
        end: usize,
        text: String,
        metavars: Vec<MetavarBinding>,
    ) -> Self {
        Self {
            pattern,
            start_position: start,
            end_position: end,
            matched_text: text,
            metavars,
        }
    }

    /// Get matched text
    pub fn matched_text(&self) -> &str {
        &self.matched_text
    }

    /// Get metavariable by name
    pub fn get_metavar(&self, name: &str) -> Option<&MetavarBinding> {
        self.metavars.iter().find(|m| m.name == name)
    }

    /// Get all metavariables
    pub fn metavars(&self) -> &[MetavarBinding] {
        &self.metavars
    }
}

/// Pattern matcher
pub struct PatternMatcher {
    language: Language,
}

impl PatternMatcher {
    /// Create a new pattern matcher for a language
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// Match patterns against AST
    pub fn match_patterns(&self, ast: &ASTNode, patterns: &[Pattern]) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        for pattern in patterns {
            let pattern_matches = self.match_single_pattern(ast, pattern)?;
            matches.extend(pattern_matches);
        }

        Ok(matches)
    }

    /// Match a single pattern against AST
    fn match_single_pattern(&self, ast: &ASTNode, pattern: &Pattern) -> Result<Vec<Match>> {
        // Simple pattern matching implementation
        // For now, does basic string matching
        // TODO: Implement actual tree-sitter pattern matching

        let mut matches = Vec::new();
        let pattern_text = &pattern.pattern;

        // Search for pattern in AST nodes
        self.search_in_node(ast, pattern, pattern_text, &mut matches)?;

        Ok(matches)
    }

    /// Search for pattern in a node
    fn search_in_node(
        &self,
        node: &ASTNode,
        pattern: &Pattern,
        pattern_text: &str,
        matches: &mut Vec<Match>,
    ) -> Result<()> {
        // Check if node text contains pattern
        let node_text = node.text();

        if node_text.contains(pattern_text.trim()) {
            // Simple match found
            let metavars = self.extract_metavariables(pattern, node_text);
            matches.push(Match::new(
                pattern.clone(),
                node.start_position,
                node.end_position,
                node_text.to_string(),
                metavars,
            ));
        }

        // Recursively search children
        for child in node.children() {
            self.search_in_node(child, pattern, pattern_text, matches)?;
        }

        Ok(())
    }

    /// Extract metavariables from matched text
    fn extract_metavariables(&self, pattern: &Pattern, matched_text: &str) -> Vec<MetavarBinding> {
        let mut metavars = Vec::new();
        let pattern_text = &pattern.pattern;

        // Simple metavariable extraction
        // Looking for patterns like $VAR, $FUNC, $OBJ, etc.
        let metavar_regex = regex::Regex::new(r"\$(\w+)").unwrap();

        for capture in metavar_regex.captures_iter(pattern_text) {
            if let Some(var_name) = capture.get(1) {
                // Create a placeholder binding
                // In real implementation, would extract actual value from AST
                metavars.push(MetavarBinding {
                    name: var_name.as_str().to_string(),
                    value: matched_text.to_string(),
                    node: ASTNode::new_leaf(
                        "metavar".to_string(),
                        matched_text.to_string(),
                        0,
                        matched_text.len(),
                    ),
                });
            }
        }

        metavars
    }

    /// Apply where clause filters
    pub fn apply_where_clause(&self, matches: &[Match], where_clause: &WhereClause) -> Vec<Match> {
        matches
            .iter()
            .filter(|m| self.matches_where_clause(m, where_clause))
            .cloned()
            .collect()
    }

    /// Check if match satisfies where clause
    fn matches_where_clause(&self, match_: &Match, where_clause: &WhereClause) -> bool {
        // Simple implementation
        // TODO: Implement actual where clause evaluation

        // Check negation
        if let Some(not_clause) = &where_clause.not {
            // If there's a negation, we should NOT match the pattern
            return !self.matches_where_clause(match_, not_clause);
        }

        // For now, accept all matches
        // TODO: Implement actual validation
        true
    }

    /// Convert matches to facts
    pub fn matches_to_facts(&self, matches: &[Match], rule: &Rule) -> Vec<hodei_ir::Fact> {
        let mut facts = Vec::new();

        for match_ in matches {
            let fact = self.match_to_fact(match_, rule);
            facts.push(fact);
        }

        facts
    }

    /// Convert a match to a Fact
    fn match_to_fact(&self, match_: &Match, rule: &Rule) -> hodei_ir::Fact {
        use hodei_ir::{
            ColumnNumber, Confidence, ExtractorId, Fact, FactId, FactType, LineNumber, ProjectPath,
            Provenance, Severity, SourceLocation,
        };

        // Create source location
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from("")),
            LineNumber::new(1).unwrap(),
            None,
            LineNumber::new(1).unwrap(),
            None,
        );

        // Create provenance
        let provenance = Provenance::new(
            ExtractorId::Custom,
            rule.id.clone(),
            Confidence::new(0.8).unwrap(),
        );

        // Create fact type based on rule metadata
        let fact_type = FactType::CodeSmell {
            smell_type: rule.metadata.as_ref().map_or("custom".to_string(), |m| {
                m.category.as_ref().unwrap_or(&"custom".to_string()).clone()
            }),
            severity: Severity::Minor,
        };

        // Create fact
        Fact::new(fact_type, location, provenance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_sitter::{ASTNode, Language};

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = PatternMatcher::new(Language::Python);
        assert_eq!(matcher.language, Language::Python);
    }

    #[test]
    fn test_simple_pattern_match() {
        let matcher = PatternMatcher::new(Language::Python);

        // Create a simple AST
        let ast = ASTNode::new_internal(
            "module".to_string(),
            vec![ASTNode::new_leaf(
                "assignment".to_string(),
                "x = 42".to_string(),
                0,
                5,
            )],
        );

        let patterns = vec![Pattern {
            pattern: "x = 42".to_string(),
            message: "Variable assignment".to_string(),
        }];

        let matches = matcher.match_patterns(&ast, &patterns).unwrap();
        assert!(!matches.is_empty(), "Should find at least one match");
    }

    #[test]
    fn test_metavar_extraction() {
        let matcher = PatternMatcher::new(Language::Python);

        let pattern = Pattern {
            pattern: "$VAR = $VALUE".to_string(),
            message: "Assignment".to_string(),
        };

        let metavars = matcher.extract_metavariables(&pattern, "x = 42");
        assert!(!metavars.is_empty(), "Should extract metavariables");
        assert!(metavars.iter().any(|m| m.name == "VAR"));
        assert!(metavars.iter().any(|m| m.name == "VALUE"));
    }

    #[test]
    fn test_match_creation() {
        let pattern = Pattern {
            pattern: "$VAR".to_string(),
            message: "Test".to_string(),
        };

        let metavar = MetavarBinding {
            name: "VAR".to_string(),
            value: "x".to_string(),
            node: ASTNode::new_leaf("identifier".to_string(), "x".to_string(), 0, 1),
        };

        let match_ = Match::new(pattern.clone(), 0, 1, "x".to_string(), vec![metavar]);

        assert_eq!(match_.matched_text(), "x");
        assert_eq!(match_.metavars().len(), 1);
        assert_eq!(match_.get_metavar("VAR").unwrap().name, "VAR");
    }
}
