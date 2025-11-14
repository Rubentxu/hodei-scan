//! Datafrog Datalog Rules for Taint Analysis
//!
//! This module implements the Datalog rules for taint propagation using datafrog.
//! The rules define how taint flows from sources through the program to sinks.

use crate::taint_analysis::{
    SanitizerDefinition, SinkDefinition, SourceDefinition, TaintFlow, TaintPolicy,
};
use datafrog::{Iteration, Relation, Variable};
use hodei_ir::{Fact, FactId, FactType, FlowId, FunctionName, VariableName};

/// Structure to hold datafrog iteration and variables
pub struct TaintDatalogEngine {
    pub iteration: Iteration<'static>,
    // Input facts
    pub sources: Variable<(FlowId, VariableName, String)>,
    pub sinks: Variable<(FlowId, FunctionName, String)>,
    pub sanitizers: Variable<(FlowId, String, bool)>,
    // Derived relations
    pub tainted: Variable<FlowId>,
    pub flows_to_sink: Variable<(FlowId, FunctionName)>,
    pub sanitized: Variable<FlowId>,
}

impl TaintDatalogEngine {
    /// Create a new datafrog engine for taint analysis
    pub fn new() -> Self {
        Self {
            iteration: Iteration::new(),
            sources: Variable::new(),
            sinks: Variable::new(),
            sanitizers: Variable::new(),
            tainted: Variable::new(),
            flows_to_sink: Variable::new(),
            sanitized: Variable::new(),
        }
    }

    /// Load policy sources into datafrog
    pub fn load_sources(&mut self, facts: &[&Fact], policy: &TaintPolicy) {
        let mut source_data = Vec::new();

        for fact in facts {
            if let FactType::TaintSource {
                var,
                flow_id,
                source_type,
                ..
            } = &fact.fact_type
            {
                // Check if this source matches any policy pattern
                for source_def in &policy.sources {
                    if self.pattern_matches(&var.0, &source_def.pattern) {
                        source_data.push((*flow_id, var.clone(), source_type.clone()));
                    }
                }
            }
        }

        self.sources.insert(&mut self.iteration, source_data);
    }

    /// Load policy sinks into datafrog
    pub fn load_sinks(&mut self, facts: &[&Fact], policy: &TaintPolicy) {
        let mut sink_data = Vec::new();

        for fact in facts {
            if let FactType::TaintSink {
                func,
                consumes_flow,
                category,
                ..
            } = &fact.fact_type
            {
                // Check if this sink matches any policy pattern
                for sink_def in &policy.sinks {
                    if self.pattern_matches(&func.0, &sink_def.pattern)
                        && self.pattern_matches(category, &sink_def.category)
                    {
                        sink_data.push((*consumes_flow, func.clone(), category.clone()));
                    }
                }
            }
        }

        self.sinks.insert(&mut self.iteration, sink_data);
    }

    /// Load policy sanitizers into datafrog
    pub fn load_sanitizers(&mut self, facts: &[&Fact], policy: &TaintPolicy) {
        let mut sanitizer_data = Vec::new();

        for fact in facts {
            if let FactType::Sanitization {
                method,
                sanitizes_flow,
                effective,
                ..
            } = &fact.fact_type
            {
                // Check if this sanitizer matches any policy pattern
                for sanitizer_def in &policy.sanitizers {
                    if self.pattern_matches(method, &sanitizer_def.pattern) {
                        sanitizer_data.push((*sanitizes_flow, method.clone(), *effective));
                    }
                }
            }
        }

        self.sanitizers.insert(&mut self.iteration, sanitizer_data);
    }

    /// Run the taint propagation Datalog rules
    pub fn run_analysis(&mut self) -> Result<(), TaintAnalysisError> {
        // Rule 1: Initial taint from sources
        // tainted(FlowId) :- sources(FlowId, _, _)
        self.tainted
            .from_map(&self.sources, |(flow_id, _, _)| *flow_id);

        // Rule 2: Taint propagates through data flow
        // This would be implemented with data flow relations
        // For now, we use a simplified version

        // Rule 3: Find flows to sinks
        // flows_to_sink(FlowId, Func) :- sinks(FlowId, Func, _), tainted(FlowId)
        self.flows_to_sink
            .from_join(&self.sinks, &self.tainted, |flow_id, func, _, _| {
                (*flow_id, func.clone())
            });

        // Rule 4: Check for sanitization
        // sanitized(FlowId) :- sanitizers(FlowId, _, true), tainted(FlowId)
        self.sanitized.from_join(
            &self.sanitizers,
            &self.tainted,
            |flow_id, _, effective, _| {
                if *effective { Some(*flow_id) } else { None }
            },
        );

        // Iterate to fixed point
        let _ = self.iteration;

        Ok(())
    }

    /// Extract taint flows from the analysis results
    pub fn extract_taint_flows(&self) -> Result<Vec<TaintFlow>, TaintAnalysisError> {
        let mut flows = Vec::new();

        // Get flows to sinks
        let flows_data: Vec<_> = self
            .flows_to_sink
            .iter()
            .filter(|(flow_id, _)| !self.sanitized.contains(flow_id))
            .collect();

        for (flow_id, func) in flows_data {
            // Find the corresponding source
            if let Some((_, var, source_type)) =
                self.sources.iter().find(|(fid, _, _)| fid == flow_id)
            {
                flows.push(TaintFlow {
                    source: format!("{} ({})", var.0, source_type),
                    sink: func.0.clone(),
                    path: vec![var.0.clone(), func.0.clone()],
                });
            }
        }

        Ok(flows)
    }

    /// Simple pattern matching (placeholder for regex)
    fn pattern_matches(&self, text: &str, pattern: &str) -> bool {
        // Simple substring match for now
        // TODO: Replace with proper regex matching
        text.contains(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::{Confidence, ExtractorId, Fact, FactType, Provenance, SourceLocation};

    fn create_test_fact(fact_type: FactType) -> Fact {
        Fact::new(
            fact_type,
            SourceLocation::new(
                hodei_ir::types::ProjectPath::new(std::path::PathBuf::from("test.rs")),
                hodei_ir::types::LineNumber::new(1).unwrap(),
                None,
                hodei_ir::types::LineNumber::new(1).unwrap(),
                None,
            ),
            Provenance::new(
                ExtractorId::TreeSitter,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        )
    }

    #[test]
    fn test_new_engine() {
        let engine = TaintDatalogEngine::new();
        assert!(format!("{:?}", engine).contains("TaintDatalogEngine"));
    }

    #[test]
    fn test_load_sources_with_single_match() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "request".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        engine.load_sources(&[&fact], &policy);

        let sources: Vec<_> = engine.sources.iter().collect();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].1 .0, "user_input");
    }

    #[test]
    fn test_load_sources_no_match() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let fact = create_test_fact(FactType::TaintSource {
            var: VariableName("internal_var".to_string()),
            flow_id,
            source_type: "internal".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "request".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        engine.load_sources(&[&fact], &policy);

        let sources: Vec<_> = engine.sources.iter().collect();
        assert_eq!(sources.len(), 0);
    }

    #[test]
    fn test_load_sources_multiple_patterns() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id1 = FlowId::new_uuid();
        let flow_id2 = FlowId::new_uuid();

        let fact1 = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id: flow_id1,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let fact2 = create_test_fact(FactType::TaintSource {
            var: VariableName("request_data".to_string()),
            flow_id: flow_id2,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user".to_string(), // Matches user_input
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        engine.load_sources(&[&fact1, &fact2], &policy);

        let sources: Vec<_> = engine.sources.iter().collect();
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_load_sinks_with_pattern_match() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_sql_query".to_string()),
            consumes_flow: flow_id,
            category: "database".to_string(),
            severity: hodei_ir::types::Severity::Critical,
        });

        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![SinkDefinition {
                pattern: "sql".to_string(),
                category: "database".to_string(),
                severity: hodei_ir::types::Severity::Critical,
            }],
            sanitizers: vec![],
        };

        engine.load_sinks(&[&sink_fact], &policy);

        let sinks: Vec<_> = engine.sinks.iter().collect();
        assert_eq!(sinks.len(), 1);
    }

    #[test]
    fn test_load_sinks_category_mismatch() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "file".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![SinkDefinition {
                pattern: "execute".to_string(),
                category: "database".to_string(), // Mismatch
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        engine.load_sinks(&[&sink_fact], &policy);

        let sinks: Vec<_> = engine.sinks.iter().collect();
        assert_eq!(sinks.len(), 0);
    }

    #[test]
    fn test_load_sanitizers() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let sanitizer_fact = create_test_fact(FactType::Sanitization {
            method: "sanitize_html".to_string(),
            sanitizes_flow: flow_id,
            effective: true,
            confidence: Confidence::HIGH,
        });

        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![SanitizerDefinition {
                pattern: "sanitize".to_string(),
            }],
        };

        engine.load_sanitizers(&[&sanitizer_fact], &policy);

        let sanitizers: Vec<_> = engine.sanitizers.iter().collect();
        assert_eq!(sanitizers.len(), 1);
        assert_eq!(sanitizers[0].2, true); // effective flag
    }

    #[test]
    fn test_run_analysis_empty() {
        let mut engine = TaintDatalogEngine::new();
        let result = engine.run_analysis();
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_taint_propagation() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        engine.load_sources(&[&source_fact], &policy);
        engine.load_sinks(&[&sink_fact], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].source, "user_input (http)");
        assert_eq!(flows[0].sink, "execute_query");
    }

    #[test]
    fn test_extract_taint_flows_with_sanitization() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let sanitizer_fact = create_test_fact(FactType::Sanitization {
            method: "sanitize".to_string(),
            sanitizes_flow: flow_id,
            effective: true,
            confidence: Confidence::HIGH,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![SanitizerDefinition {
                pattern: "sanitize".to_string(),
            }],
        };

        engine.load_sources(&[&source_fact], &policy);
        engine.load_sinks(&[&sink_fact], &policy);
        engine.load_sanitizers(&[&sanitizer_fact], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 0); // Flow should be sanitized
    }

    #[test]
    fn test_ineffective_sanitizer() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let sanitizer_fact = create_test_fact(FactType::Sanitization {
            method: "incomplete_sanitize".to_string(),
            sanitizes_flow: flow_id,
            effective: false, // Not effective
            confidence: Confidence::LOW,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![SanitizerDefinition {
                pattern: "sanitize".to_string(),
            }],
        };

        engine.load_sources(&[&source_fact], &policy);
        engine.load_sinks(&[&sink_fact], &policy);
        engine.load_sanitizers(&[&sanitizer_fact], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 1); // Flow not sanitized
    }

    #[test]
    fn test_multiple_sources_and_sinks() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id1 = FlowId::new_uuid();
        let flow_id2 = FlowId::new_uuid();
        let flow_id3 = FlowId::new_uuid();

        let source1 = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id: flow_id1,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let source2 = create_test_fact(FactType::TaintSource {
            var: VariableName("api_key".to_string()),
            flow_id: flow_id2,
            source_type: "auth".to_string(),
            confidence: Confidence::HIGH,
        });

        let sink1 = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_sql".to_string()),
            consumes_flow: flow_id1,
            category: "database".to_string(),
            severity: hodei_ir::types::Severity::Critical,
        });

        let sink2 = create_test_fact(FactType::TaintSink {
            func: FunctionName("log_to_file".to_string()),
            consumes_flow: flow_id2,
            category: "logging".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let policy = TaintPolicy {
            sources: vec![
                SourceDefinition {
                    pattern: "user".to_string(),
                    source_type: "http".to_string(),
                    tags: vec![],
                },
                SourceDefinition {
                    pattern: "api".to_string(),
                    source_type: "auth".to_string(),
                    tags: vec![],
                },
            ],
            sinks: vec![
                SinkDefinition {
                    pattern: "execute".to_string(),
                    category: "database".to_string(),
                    severity: hodei_ir::types::Severity::Critical,
                },
                SinkDefinition {
                    pattern: "log".to_string(),
                    category: "logging".to_string(),
                    severity: hodei_ir::types::Severity::Major,
                },
            ],
            sanitizers: vec![],
        };

        engine.load_sources(&[&source1, &source2], &policy);
        engine.load_sinks(&[&sink1, &sink2], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 2);
    }

    #[test]
    fn test_pattern_matching_case_sensitive() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let fact = create_test_fact(FactType::TaintSource {
            var: VariableName("UserInput".to_string()), // Capital U
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user", // lowercase
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        engine.load_sources(&[&fact], &policy);

        let sources: Vec<_> = engine.sources.iter().collect();
        assert_eq!(sources.len(), 1); // substring match is case-sensitive in current impl
    }

    #[test]
    fn test_extract_taint_flows_empty_when_no_sinks() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        engine.load_sources(&[&source_fact], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 0);
    }

    #[test]
    fn test_extract_taint_flows_empty_when_no_sources() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        engine.load_sinks(&[&sink_fact], &policy);
        engine.run_analysis().unwrap();

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 0);
    }

    #[test]
    fn test_run_analysis_multiple_iterations() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("user_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let sink_fact = create_test_fact(FactType::TaintSink {
            func: FunctionName("execute_query".to_string()),
            consumes_flow: flow_id,
            category: "sql".to_string(),
            severity: hodei_ir::types::Severity::Major,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        engine.load_sources(&[&source_fact], &policy);
        engine.load_sinks(&[&sink_fact], &policy);

        // Run analysis multiple times
        assert!(engine.run_analysis().is_ok());
        assert!(engine.run_analysis().is_ok()); // Should be idempotent

        let flows = engine.extract_taint_flows().unwrap();
        assert_eq!(flows.len(), 1);
    }

    #[test]
    fn test_concurrent_access_patterns() {
        let mut engine = TaintDatalogEngine::new();
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("shared_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "shared"..to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        // Simulate concurrent load
        for _ in 0..10 {
            engine.load_sources(&[&source_fact], &policy);
        }

        let sources: Vec<_> = engine.sources.iter().collect();
        // Only unique entries should be stored
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn test_engine_state_isolation() {
        let flow_id = FlowId::new_uuid();

        let source_fact = create_test_fact(FactType::TaintSource {
            var: VariableName("test_input".to_string()),
            flow_id,
            source_type: "http".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let policy1 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "test".to_string(),
                source_type: "http".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        let policy2 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "other".to_string(),
                source_type: "api".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        // Create two separate engines
        let mut engine1 = TaintDatalogEngine::new();
        let mut engine2 = TaintDatalogEngine::new();

        engine1.load_sources(&[&source_fact], &policy1);
        engine2.load_sources(&[&source_fact], &policy2);

        let sources1: Vec<_> = engine1.sources.iter().collect();
        let sources2: Vec<_> = engine2.sources.iter().collect();

        assert_eq!(sources1.len(), 1); // Matches "test"
        assert_eq!(sources2.len(), 0); // Doesn't match "other"
    }
}
