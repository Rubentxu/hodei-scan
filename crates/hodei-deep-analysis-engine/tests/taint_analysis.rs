//! Tests for taint analysis functionality

use hodei_deep_analysis_engine::{
    policy::{DataTag, SinkDefinition, SourceDefinition, TaintPolicy},
    semantic_model::SemanticModel,
    taint_analysis::{TaintFlow, TaintPropagator},
};

#[test]
fn test_new_propagator() {
    let propagator = TaintPropagator::new();
    // Should be able to create propagator without errors
    assert!(format!("{:?}", propagator).contains("TaintPropagator"));
}

#[test]
fn test_run_analysis_with_empty_model() {
    let mut propagator = TaintPropagator::new();
    let model = SemanticModel::default();
    let policy = TaintPolicy::default();

    let result = propagator.run_analysis(&model, &policy);
    // With current implementation, this should succeed (even if empty)
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_run_analysis_with_empty_policy() {
    let mut propagator = TaintPropagator::new();
    let model = SemanticModel::default();
    let policy = TaintPolicy::default();

    let result = propagator.run_analysis(&model, &policy);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_policy_loading() {
    let policy = TaintPolicy::default();
    assert_eq!(policy.sources.len(), 0);
    assert_eq!(policy.sinks.len(), 0);
    assert_eq!(policy.sanitizers.len(), 0);
}

#[test]
fn test_taint_flow_structure() {
    let flow = TaintFlow {
        source: "user_input".to_string(),
        sink: "sql_query".to_string(),
        path: vec!["var_a".to_string(), "var_b".to_string()],
    };

    assert_eq!(flow.source, "user_input");
    assert_eq!(flow.sink, "sql_query");
    assert_eq!(flow.path.len(), 2);
}

#[test]
fn test_propagator_with_patterns() {
    let mut propagator = TaintPropagator::new();
    let policy = TaintPolicy {
        sources: vec![SourceDefinition {
            pattern: "request".to_string(),
            source_type: "http".to_string(),
            tags: vec![DataTag::UserInput],
        }],
        sinks: vec![SinkDefinition {
            pattern: "database".to_string(),
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        }],
        sanitizers: vec![],
    };

    // Should not panic when processing patterns
    let model = SemanticModel::default();
    let result = propagator.run_analysis(&model, &policy);
    assert!(result.is_ok());
}
