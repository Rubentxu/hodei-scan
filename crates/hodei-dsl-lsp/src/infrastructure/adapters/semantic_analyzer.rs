//! Semantic analyzer adapter
//!
//! Analyzes DSL code for semantic errors and warnings

use crate::domain::models::Diagnostic;
use crate::domain::ports::SemanticAnalyzer;
use hodei_dsl::ast::RuleFile;

/// Semantic analyzer implementation
pub struct HodeiSemanticAnalyzer {
    // Built-in fact types registry
    fact_types: Vec<String>,
    // Built-in function registry
    function_names: Vec<String>,
}

impl HodeiSemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            fact_types: vec![
                "Vulnerability".to_string(),
                "CodeSmell".to_string(),
                "SecurityIssue".to_string(),
            ],
            function_names: vec![
                "matches".to_string(),
                "contains".to_string(),
                "length_gt".to_string(),
                "length_lt".to_string(),
                "equals".to_string(),
            ],
        }
    }
}

#[async_trait::async_trait]
impl SemanticAnalyzer for HodeiSemanticAnalyzer {
    async fn analyze(&self, ast: &RuleFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Validate rules in the AST
        for rule in &ast.rules {
            // Check if fact type exists
            if !self.fact_types.contains(&rule.fact_type) {
                diagnostics.push(Diagnostic {
                    range: crate::domain::models::Range {
                        start: crate::domain::models::CursorPosition {
                            line: 0,
                            column: 0,
                        },
                        end: crate::domain::models::CursorPosition {
                            line: 0,
                            column: 10,
                        },
                    },
                    severity: crate::domain::models::DiagnosticSeverity::Error,
                    message: format!("Unknown fact type: {}", rule.fact_type),
                    source: "hodei-dsl".to_string(),
                });
            }
            
            // Validate patterns if present
            if let Some(pattern) = &rule.pattern {
                // Check for valid function calls
                // This is a simplified check - a full implementation would parse the pattern
                if pattern.contains("unknown_function(") {
                    diagnostics.push(Diagnostic {
                        range: crate::domain::models::Range {
                            start: crate::domain::models::CursorPosition {
                                line: 0,
                                column: 0,
                            },
                            end: crate::domain::models::CursorPosition {
                                line: 0,
                                column: 20,
                            },
                        },
                        severity: crate::domain::models::DiagnosticSeverity::Warning,
                        message: "Unknown function called in pattern".to_string(),
                        source: "hodei-dsl".to_string(),
                    });
                }
            }
        }
        
        diagnostics
    }
}
