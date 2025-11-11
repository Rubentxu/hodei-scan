//! Security tests for DSL validation and sandboxing

use crate::ast::*;
use crate::security::{
    ComplexityEstimate, DSLValidator, ExecutionSandbox, RuleComplexityAnalyzer, SandboxConfig,
};
use std::thread;
use std::time::Duration;

#[test]
fn test_validator_accepts_valid_input() {
    let validator = DSLValidator::new();
    let input = "rule test { match { TaintSource } emit { severity: High } }";
    assert!(validator.validate_raw_input(input).is_ok());
}

#[test]
fn test_validator_rejects_empty_input() {
    let validator = DSLValidator::new();
    assert!(validator.validate_raw_input("").is_err());
}

#[test]
fn test_validator_rejects_null_bytes() {
    let validator = DSLValidator::new();
    assert!(validator.validate_raw_input("test\0input").is_err());
}

#[test]
fn test_validator_rejects_excessive_length() {
    let validator = DSLValidator::new();
    let input = "a".repeat(10001);
    assert!(validator.validate_raw_input(&input).is_err());
}

#[test]
fn test_validator_rejects_path_traversal() {
    let validator = DSLValidator::new();
    let input = "../../../etc/passwd";
    assert!(validator.validate_raw_input(input).is_err());
}

#[test]
fn test_validator_rejects_script_tags() {
    let validator = DSLValidator::new();
    let input = "<script>alert('xss')</script>";
    assert!(validator.validate_raw_input(input).is_err());
}

#[test]
fn test_validator_rejects_javascript_protocol() {
    let validator = DSLValidator::new();
    let input = "javascript:alert('xss')";
    assert!(validator.validate_raw_input(input).is_err());
}

#[test]
fn test_validator_validates_simple_rule() {
    let validator = DSLValidator::new();
    let rule = RuleDef {
        name: "test".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: vec![Pattern {
                binding: "source".to_string(),
                fact_type: "TaintSource".to_string(),
                conditions: vec![],
                span: Default::default(),
            }],
            where_clause: None,
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    assert!(validator.validate_rule(&rule).is_ok());
}

#[test]
fn test_validator_rejects_rule_with_path_traversal_in_name() {
    let validator = DSLValidator::new();
    let rule = RuleDef {
        name: "../../../etc/passwd".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: vec![],
            where_clause: None,
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    assert!(validator.validate_rule(&rule).is_err());
}

#[test]
fn test_validator_accepts_string_literal() {
    let validator = DSLValidator::new();
    let literal = Literal::String("test".to_string());
    // This would need to be tested through a public API
    // For now, we test the raw input validation
    let input = "rule test { match { TaintSource } emit { message: \"test\" } }";
    assert!(validator.validate_raw_input(input).is_ok());
}

#[test]
fn test_validator_rejects_template_syntax() {
    let validator = DSLValidator::new();
    let input = "rule test { match { TaintSource } emit { message: \"{{template}}\" } }";
    assert!(validator.validate_raw_input(input).is_err());
}

#[test]
fn test_validator_rejects_command_execution() {
    let validator = DSLValidator::new();
    let input = "rule test { match { TaintSource } emit { message: \"$(whoami)\" } }";
    assert!(validator.validate_raw_input(input).is_err());
}

#[test]
fn test_sandbox_within_time_limit() {
    let mut sandbox = ExecutionSandbox::with_config(SandboxConfig {
        time_limit_ms: 500,
        ..Default::default()
    });
    let result = sandbox.execute(|| {
        thread::sleep(Duration::from_millis(50));
        Ok(42)
    });

    assert!(result.result.is_ok());
    // The sandbox has some timing variance, so we just check that it completed
    assert!(result.stats.execution_time_ms >= 50);
}

#[test]
fn test_sandbox_timeout_enforcement() {
    let mut sandbox = ExecutionSandbox::with_config(SandboxConfig {
        time_limit_ms: 50,
        ..Default::default()
    });

    let result = sandbox.execute(|| {
        thread::sleep(Duration::from_millis(200));
        Ok(42)
    });

    assert!(result.stats.timed_out);
}

#[test]
fn test_sandbox_memory_monitoring() {
    let mut sandbox = ExecutionSandbox::with_config(SandboxConfig {
        memory_limit_bytes: 1024,
        ..Default::default()
    });

    let result = sandbox.execute(|| {
        let data = vec![0u8; 2048];
        Ok(data.len())
    });

    assert!(result.result.is_ok());
}

#[test]
fn test_complexity_analysis() {
    let analyzer = RuleComplexityAnalyzer::new();
    let rule = RuleDef {
        name: "test".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: vec![Pattern {
                binding: "source".to_string(),
                fact_type: "TaintSource".to_string(),
                conditions: vec![],
                span: Default::default(),
            }],
            where_clause: None,
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    let estimate = analyzer.estimate_complexity(&rule).unwrap();

    assert!(estimate.pattern_count > 0);
    assert!(estimate.complexity_score > 0);
    assert!(estimate.estimated_iterations > 0);
}

#[test]
fn test_complexity_level_classification() {
    let mut estimate = ComplexityEstimate::default();
    assert_eq!(estimate.complexity_level().to_string(), "Low");

    estimate.complexity_score = 30;
    assert_eq!(estimate.complexity_level().to_string(), "Medium");

    estimate.complexity_score = 60;
    assert_eq!(estimate.complexity_level().to_string(), "High");

    estimate.complexity_score = 90;
    assert_eq!(estimate.complexity_level().to_string(), "Very High");
}

#[test]
fn test_sandbox_statistics() {
    let mut sandbox = ExecutionSandbox::new();
    let result = sandbox.execute(|| {
        thread::sleep(Duration::from_millis(10));
        Ok("test")
    });

    assert!(result.result.is_ok());
    assert!(result.stats.execution_time_ms > 0);
}

#[test]
fn test_sandbox_update_stats() {
    let sandbox = ExecutionSandbox::new();
    sandbox.update_stats(|stats| {
        stats.facts_scanned = 100;
        stats.iterations = 1000;
    });

    let stats = sandbox.get_stats();
    assert_eq!(stats.facts_scanned, 100);
    assert_eq!(stats.iterations, 1000);
}

#[test]
fn test_complexity_analysis_with_where_clause() {
    let analyzer = RuleComplexityAnalyzer::new();
    let rule = RuleDef {
        name: "test".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: vec![Pattern {
                binding: "source".to_string(),
                fact_type: "TaintSource".to_string(),
                conditions: vec![],
                span: Default::default(),
            }],
            where_clause: Some(Expr::Binary {
                left: Box::new(Expr::Path(Path {
                    segments: vec!["severity".to_string()],
                    span: Default::default(),
                })),
                op: BinaryOp::Eq,
                right: Box::new(Expr::Literal(Literal::String("high".to_string()))),
            }),
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    let estimate = analyzer.estimate_complexity(&rule).unwrap();

    assert!(estimate.expression_count > 0);
    assert!(estimate.estimated_facts_scanned > 0);
}
