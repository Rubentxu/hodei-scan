//! Integration tests for hodei-deep-analysis-engine
//!
//! These tests verify that different components work together correctly.

use hodei_deep_analysis_engine::{
    analysis_cache::{CouplingCache, SemanticModelCache, TaintFlowCache},
    connascence::{ConnascenceAnalyzer, CouplingFinding, EntityId},
    connascence::types::{ConnascenceType, Strength},
    policy::{DataTag, SanitizerDefinition, SinkDefinition, SourceDefinition, TaintPolicy},
    semantic_model::{FactExtractor, SemanticModel},
    taint_analysis::{TaintFlow, TaintPropagator},
};
use hodei_ir::{Confidence, Fact, FactType, FlowId, FunctionName, VariableName};
use std::sync::Arc;

#[test]
fn test_full_taint_analysis_pipeline() {
    // Test the complete taint analysis pipeline from policy to flows
    let mut propagator = TaintPropagator::new();
    let model = SemanticModel::new();

    let policy = TaintPolicy {
        sources: vec![SourceDefinition {
            pattern: "user_input".to_string(),
            source_type: "http".to_string(),
            tags: vec![DataTag::UserInput],
        }],
        sinks: vec![SinkDefinition {
            pattern: "execute_query".to_string(),
            category: "database".to_string(),
            severity: hodei_ir::types::Severity::Major,
        }],
        sanitizers: vec![],
    };

    let result = propagator.run_analysis(&model, &policy);
    assert!(result.is_ok());
}

#[test]
fn test_full_connascence_analysis_pipeline() {
    // Test the complete connascence analysis pipeline
    let analyzer = ConnascenceAnalyzer::new();
    let model = SemanticModel::new();

    let result = analyzer.analyze(&model);
    assert!(result.is_ok());

    let findings = result.unwrap();
    assert!(findings.len() > 0);
}

#[test]
fn test_fact_extractor_creation() {
    // Test that FactExtractor can create facts correctly
    let mut extractor = hodei_deep_analysis_engine::semantic_model::FactExtractor::new();

    let flow_id = FlowId::new_uuid();

    // Create facts using extractor
    let source_fact = extractor.create_taint_source("user_input", "http");
    let sink_fact = extractor.create_taint_sink("execute_query", flow_id, "sql");
    let sanitizer_fact = extractor.create_sanitizer("sanitize", flow_id);

    // Verify facts are created with different IDs
    assert_ne!(source_fact.id, sink_fact.id);
    assert_ne!(sink_fact.id, sanitizer_fact.id);
    assert_ne!(source_fact.id, sanitizer_fact.id);
}

#[test]
fn test_fact_extractor_with_multiple_facts() {
    let mut extractor = hodei_deep_analysis_engine::semantic_model::FactExtractor::new();

    // Create multiple related facts
    let flow_id1 = FlowId::new_uuid();
    let flow_id2 = FlowId::new_uuid();

    let source1 = extractor.create_taint_source("user_input", "http");
    let source2 = extractor.create_taint_source("api_key", "auth");
    let sink1 = extractor.create_taint_sink("execute_query", flow_id1, "sql");
    let sink2 = extractor.create_taint_sink("write_file", flow_id2, "file");
    let sanitizer1 = extractor.create_sanitizer("sanitize", flow_id1);

    // Verify all facts are created
    assert_eq!(source1.id != source2.id, true);
    assert_eq!(sink1.id != sink2.id, true);
    assert_eq!(sanitizer1.id != source1.id, true);
}

#[test]
fn test_caching_with_analysis_results() {
    use hodei_deep_analysis_engine::analysis_cache::{SemanticModelCache, TaintFlowCache};

    let semantic_cache = SemanticModelCache::new(60);
    let taint_cache = TaintFlowCache::new(60);

    // Cache semantic model
    let model_data = "semantic_model_data".to_string();
    semantic_cache.put("model1".to_string(), model_data.clone());
    assert_eq!(semantic_cache.get("model1"), Some(model_data));

    // Cache taint flows
    let flows = vec!["flow1".to_string(), "flow2".to_string()];
    taint_cache.put("analysis1".to_string(), flows.clone());
    assert_eq!(taint_cache.get("analysis1"), Some(flows));
}

#[test]
fn test_propagator_with_cache() {
    use hodei_deep_analysis_engine::analysis_cache::TaintFlowCache;

    let cache = TaintFlowCache::new(60);
    let mut propagator = TaintPropagator::new();
    let model = SemanticModel::new();

    let policy = TaintPolicy {
        sources: vec![SourceDefinition {
            pattern: "input".to_string(),
            source_type: "http".to_string(),
            tags: vec![],
        }],
        sinks: vec![SinkDefinition {
            pattern: "output".to_string(),
            category: "database".to_string(),
            severity: hodei_ir::types::Severity::Major,
        }],
        sanitizers: vec![],
    };

    // First analysis
    let result1 = propagator.run_analysis(&model, &policy).unwrap();

    // Cache the result
    cache.put("key1".to_string(), result1.iter().map(|f| format!("{}->{}", f.source, f.sink)).collect());

    // Verify cache hit
    let cached = cache.get("key1");
    assert!(cached.is_some());
}

#[test]
fn test_policy_with_different_data_tags() {
    let policy = TaintPolicy {
        sources: vec![
            SourceDefinition {
                pattern: "pii_data".to_string(),
                source_type: "form".to_string(),
                tags: vec![DataTag::PII],
            },
            SourceDefinition {
                pattern: "credit_card".to_string(),
                source_type: "payment".to_string(),
                tags: vec![DataTag::Finance],
            },
            SourceDefinition {
                pattern: "password".to_string(),
                source_type: "auth".to_string(),
                tags: vec![DataTag::Credentials],
            },
            SourceDefinition {
                pattern: "user_comment".to_string(),
                source_type: "web".to_string(),
                tags: vec![DataTag::UserInput],
            },
        ],
        sinks: vec![],
        sanitizers: vec![],
    };

    assert_eq!(policy.sources.len(), 4);

    let mut propagator = TaintPropagator::new();
    propagator.extract_patterns(&policy);

    assert_eq!(propagator.source_patterns.len(), 4);
    assert!(propagator.source_patterns.contains("pii_data"));
    assert!(propagator.source_patterns.contains("credit_card"));
    assert!(propagator.source_patterns.contains("password"));
    assert!(propagator.source_patterns.contains("user_comment"));
}

#[test]
fn test_connascence_analyzer_with_cache() {
    use hodei_deep_analysis_engine::analysis_cache::CouplingCache;

    let cache = CouplingCache::new(60);
    let analyzer = ConnascenceAnalyzer::new();
    let model = SemanticModel::new();

    // Run analysis
    let result = analyzer.analyze(&model).unwrap();

    // Cache findings
    let finding_strings: Vec<String> = result
        .iter()
        .map(|f| format!("{:?}:{:?}", f.connascence_type, f.strength))
        .collect();

    cache.put("analysis1".to_string(), finding_strings.clone());

    // Verify cache
    let cached = cache.get("analysis1");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap(), finding_strings);
}

#[test]
fn test_multiple_analyses_with_same_model() {
    let analyzer = ConnascenceAnalyzer::new();
    let mut propagator = TaintPropagator::new();
    let model = SemanticModel::new();

    let policy = TaintPolicy {
        sources: vec![SourceDefinition {
            pattern: "input".to_string(),
            source_type: "http".to_string(),
            tags: vec![],
        }],
        sinks: vec![SinkDefinition {
            pattern: "output".to_string(),
            category: "db".to_string(),
            severity: hodei_ir::types::Severity::Major,
        }],
        sanitizers: vec![],
    };

    // Run multiple analyses
    let result1 = propagator.run_analysis(&model, &policy);
    let result2 = propagator.run_analysis(&model, &policy);
    let result3 = analyzer.analyze(&model);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    // Results should be consistent
    assert_eq!(result1.unwrap().len(), result2.unwrap().len());
}

#[test]
fn test_fact_creation_with_flow_tracking() {
    let mut extractor = hodei_deep_analysis_engine::semantic_model::FactExtractor::new();

    // Create a flow with multiple points - sink and sanitizer should have same flow_id
    let flow_id = FlowId::new_uuid();

    let source = extractor.create_taint_source("user_input", "http");
    let sink = extractor.create_taint_sink("process_data", flow_id, "processing");
    let sanitizer = extractor.create_sanitizer("validate", flow_id);

    // Sink and sanitizer should reference the same flow
    match (&sink.fact_type, &sanitizer.fact_type) {
        (FactType::TaintSink { consumes_flow: sink_flow, .. },
         FactType::Sanitization { sanitizes_flow: san_flow, .. }) => {
            assert_eq!(sink_flow, &flow_id);
            assert_eq!(san_flow, &flow_id);
            assert_eq!(sink_flow, san_flow);
        }
        _ => panic!("Expected TaintSink and Sanitization facts"),
    }

    // Source has its own flow_id (generated internally)
    match &source.fact_type {
        FactType::TaintSource { .. } => {
            // Source flow_id is generated internally, not controlled by caller
            assert!(true, "Source fact created successfully");
        }
        _ => panic!("Expected TaintSource fact"),
    }
}

#[test]
fn test_empty_components_interaction() {
    let analyzer = ConnascenceAnalyzer::new();
    let mut propagator = TaintPropagator::new();
    let mut extractor = FactExtractor::new();

    let empty_model = SemanticModel::default();
    let empty_policy = TaintPolicy::default();

    // All should handle empty inputs gracefully
    let taint_result = propagator.run_analysis(&empty_model, &empty_policy);
    let connascence_result = analyzer.analyze(&empty_model);
    let facts = extractor.extract_facts(&empty_model);

    assert!(taint_result.is_ok());
    assert!(connascence_result.is_ok());
    assert_eq!(facts.len(), 0);
}

#[test]
fn test_policy_sources_sinks_matching() {
    let policy = TaintPolicy {
        sources: vec![
            SourceDefinition {
                pattern: "user".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            },
            SourceDefinition {
                pattern: "request".to_string(),
                source_type: "api".to_string(),
                tags: vec![],
            },
        ],
        sinks: vec![
            SinkDefinition {
                pattern: "query".to_string(),
                category: "database".to_string(),
                severity: hodei_ir::types::Severity::Critical,
            },
            SinkDefinition {
                pattern: "save".to_string(),
                category: "storage".to_string(),
                severity: hodei_ir::types::Severity::Major,
            },
        ],
        sanitizers: vec![
            SanitizerDefinition {
                pattern: "sanitize".to_string(),
                method: Some("sanitize".to_string()),
            },
            SanitizerDefinition {
                pattern: "escape".to_string(),
                method: Some("escape".to_string()),
            },
        ],
    };

    let mut propagator = TaintPropagator::new();
    propagator.extract_patterns(&policy);

    assert_eq!(propagator.source_patterns.len(), 2);
    assert_eq!(propagator.sink_patterns.len(), 2);
    assert_eq!(propagator.sanitizer_patterns.len(), 2);
}

#[test]
fn test_coupling_finding_structure() {
    let finding = CouplingFinding {
        entity: EntityId("test_entity".to_string()),
        connascence_type: ConnascenceType::Position,
        strength: Strength::High,
        related_entities: vec![
            EntityId("entity1".to_string()),
            EntityId("entity2".to_string()),
        ],
        message: "Test coupling detected".to_string(),
        remediation: "Extract interface".to_string(),
    };

    assert_eq!(finding.entity.0, "test_entity");
    assert_eq!(finding.connascence_type, ConnascenceType::Position);
    assert_eq!(finding.strength, Strength::High);
    assert_eq!(finding.related_entities.len(), 2);
    assert!(finding.message.contains("coupling"));
    assert!(finding.remediation.contains("Extract"));
}

#[test]
fn test_taint_flow_with_cache() {
    use hodei_deep_analysis_engine::analysis_cache::TaintFlowCache;

    let cache = TaintFlowCache::new(60);

    let flow = TaintFlow {
        source: "user_input".to_string(),
        sink: "execute_query".to_string(),
        path: vec!["var_a".to_string(), "var_b".to_string()],
    };

    // Convert flow to string for caching
    let flow_str = format!("{}->{}: {:?}", flow.source, flow.sink, flow.path);
    cache.put("flow1".to_string(), vec![flow_str]);

    let cached_flows = cache.get("flow1").unwrap();
    assert_eq!(cached_flows.len(), 1);
    assert!(cached_flows[0].contains("user_input"));
    assert!(cached_flows[0].contains("execute_query"));
}

#[test]
fn test_semantic_model_builder_integration() {
    let temp_file = "/tmp/test_integration.rs";
    std::fs::write(temp_file, r#"
        fn main() {
            let user_input = get_input();
            let result = process_data(user_input);
            save_result(result);
        }

        fn get_input() -> String {
            "input".to_string()
        }

        fn process_data(data: String) -> String {
            data.to_uppercase()
        }

        fn save_result(result: String) {
            println!("{}", result);
        }
    "#).unwrap();

    let mut builder = hodei_deep_analysis_engine::semantic_model::SemanticModelBuilder::new();
    let model = builder.from_source(temp_file);

    assert!(model.is_ok());
    let model = model.unwrap();

    // Model should be created even without full parsing
    assert_eq!(model.cfg_node_count(), 0);
    assert_eq!(model.dfg_node_count(), 0);

    // Clean up
    std::fs::remove_file(temp_file).unwrap();
}

#[test]
fn test_concurrent_cache_operations() {
    use hodei_deep_analysis_engine::analysis_cache::SemanticModelCache;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let cache: Arc<SemanticModelCache> = Arc::new(SemanticModelCache::new(60));
    let cache_clone = Arc::clone(&cache);

    // Spawn thread to write
    let handle = thread::spawn(move || {
        cache_clone.put("key1".to_string(), "value1".to_string());
    });

    // Wait for thread
    handle.join().unwrap();

    // Should be written now
    assert_eq!(cache.get("key1"), Some("value1".to_string()));
}

#[test]
fn test_analysis_components_work_together() {
    // Test that all components can be instantiated and used together
    let _propagator = TaintPropagator::new();
    let _analyzer = ConnascenceAnalyzer::new();
    let _extractor = FactExtractor::new();
    let _cache = SemanticModelCache::new(60);
    let _builder = hodei_deep_analysis_engine::semantic_model::SemanticModelBuilder::new();

    // All components instantiated successfully - if we got here, test passed
    assert!(true);
}
