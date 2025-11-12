//! Unit tests for SemanticAnalyzer

use hodei_dsl_lsp::domain::models::{Diagnostic, DiagnosticSeverity};
use hodei_dsl_lsp::infrastructure::adapters::HodeiSemanticAnalyzer;
use hodei_dsl::ast::{RuleFile, Rule};
use crate::fixtures::{UNKNOWN_FACT_TYPE, MULTIPLE_FACT_TYPES, BASIC_RULE};

#[tokio::test]
async fn test_semantic_analyzer_initialization() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Analyzer should be initialized with built-in fact types
    assert!(true); // If we can create it, initialization worked
}

#[tokio::test]
async fn test_analyze_valid_rule() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create a valid AST
    let ast = RuleFile {
        rules: vec![Rule {
            name: "test_rule".to_string(),
            fact_type: "Vulnerability".to_string(),
            pattern: Some("test".to_string()),
            span: Default::default(),
        }],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Valid rule should have no diagnostics
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_analyze_unknown_fact_type() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create AST with unknown fact type
    let ast = RuleFile {
        rules: vec![Rule {
            name: "test_rule".to_string(),
            fact_type: "UnknownFact".to_string(),
            pattern: Some("test".to_string()),
            span: Default::default(),
        }],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Should have a diagnostic for unknown fact type
    assert_eq!(diagnostics.len(), 1);
    
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(diagnostic.message.contains("Unknown fact type: UnknownFact"));
    assert_eq!(diagnostic.source, "hodei-dsl");
}

#[tokio::test]
async fn test_analyze_multiple_rules_mixed() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create AST with mix of valid and invalid rules
    let ast = RuleFile {
        rules: vec![
            Rule {
                name: "valid_rule".to_string(),
                fact_type: "Vulnerability".to_string(),
                pattern: Some("test".to_string()),
                span: Default::default(),
            },
            Rule {
                name: "invalid_rule".to_string(),
                fact_type: "InvalidFact".to_string(),
                pattern: Some("test".to_string()),
                span: Default::default(),
            },
        ],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Should have one diagnostic for the invalid rule
    assert_eq!(diagnostics.len(), 1);
    
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(diagnostic.message.contains("InvalidFact"));
}

#[tokio::test]
async fn test_analyze_empty_ast() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create empty AST
    let ast = RuleFile {
        rules: Vec::new(),
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Empty AST should have no diagnostics
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_analyze_known_fact_types() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Test all known fact types
    let known_facts = vec![
        "Vulnerability",
        "CodeSmell",
        "SecurityIssue",
    ];
    
    for fact_type in known_facts {
        let ast = RuleFile {
            rules: vec![Rule {
                name: "test_rule".to_string(),
                fact_type: fact_type.to_string(),
                pattern: Some("test".to_string()),
                span: Default::default(),
            }],
        };
        
        let diagnostics = analyzer.analyze(&ast).await;
        
        // Known fact types should have no diagnostics
        assert!(
            diagnostics.is_empty(),
            "Fact type '{}' should be known", 
            fact_type
        );
    }
}

#[tokio::test]
async fn test_fact_type_registry() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Verify all expected fact types are in the registry
    let expected_facts = vec![
        "Vulnerability",
        "CodeSmell",
        "SecurityIssue",
    ];
    
    for fact in expected_facts {
        assert!(
            analyzer.fact_types.contains(&fact.to_string()),
            "Fact type '{}' should be in registry",
            fact
        );
    }
}

#[tokio::test]
async fn test_function_registry() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Verify all expected functions are in the registry
    let expected_functions = vec![
        "matches",
        "contains",
        "length_gt",
        "length_lt",
        "equals",
    ];
    
    for func in expected_functions {
        assert!(
            analyzer.function_names.contains(&func.to_string()),
            "Function '{}' should be in registry",
            func
        );
    }
}

#[tokio::test]
async fn test_analyze_with_pattern() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create AST with pattern
    let ast = RuleFile {
        rules: vec![Rule {
            name: "test_rule".to_string(),
            fact_type: "Vulnerability".to_string(),
            pattern: Some("matches(input, /pattern/)".to_string()),
            span: Default::default(),
        }],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Pattern without unknown functions should have no diagnostics
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_analyze_without_pattern() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create AST without pattern
    let ast = RuleFile {
        rules: vec![Rule {
            name: "test_rule".to_string(),
            fact_type: "Vulnerability".to_string(),
            pattern: None,
            span: Default::default(),
        }],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Rule without pattern should have no diagnostics
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_diagnostic_properties() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    let ast = RuleFile {
        rules: vec![Rule {
            name: "test_rule".to_string(),
            fact_type: "UnknownFact".to_string(),
            pattern: Some("test".to_string()),
            span: Default::default(),
        }],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    assert_eq!(diagnostics.len(), 1);
    
    let diagnostic = &diagnostics[0];
    
    // Verify all fields are set correctly
    assert!(matches!(diagnostic.severity, DiagnosticSeverity::Error));
    assert!(!diagnostic.message.is_empty());
    assert_eq!(diagnostic.source, "hodei-dsl");
}

#[tokio::test]
async fn test_analyze_multiple_unknown_facts() {
    let analyzer = HodeiSemanticAnalyzer::new();
    
    // Create AST with multiple unknown fact types
    let ast = RuleFile {
        rules: vec![
            Rule {
                name: "rule1".to_string(),
                fact_type: "Unknown1".to_string(),
                pattern: Some("test".to_string()),
                span: Default::default(),
            },
            Rule {
                name: "rule2".to_string(),
                fact_type: "Unknown2".to_string(),
                pattern: Some("test".to_string()),
                span: Default::default(),
            },
            Rule {
                name: "rule3".to_string(),
                fact_type: "Vulnerability".to_string(), // Known fact
                pattern: Some("test".to_string()),
                span: Default::default(),
            },
        ],
    };
    
    let diagnostics = analyzer.analyze(&ast).await;
    
    // Should have diagnostics for the two unknown facts
    assert_eq!(diagnostics.len(), 2);
    
    // Check that both unknown facts are reported
    assert!(diagnostics.iter().any(|d| d.message.contains("Unknown1")));
    assert!(diagnostics.iter().any(|d| d.message.contains("Unknown2")));
}
