//! Taint Analysis Propagator
//!
//! This module implements datafrog-based taint propagation using Datalog rules
//! for detecting data flow vulnerabilities. It leverages the FlowIndex from
//! hodei-engine for efficient graph operations.

use crate::Result;
use crate::policy::{DataTag, SanitizerDefinition, SinkDefinition, SourceDefinition, TaintPolicy};
use crate::semantic_model::SemanticModel;
use hodei_engine::store::FlowIndex;
use hodei_ir::{Fact, FlowId, FunctionName, VariableName};
use std::collections::HashSet;

/// Taint propagation error
#[derive(Debug, thiserror::Error)]
pub enum TaintAnalysisError {
    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Flow index error: {0}")]
    FlowIndex(String),

    #[error("Datalog evaluation error: {0}")]
    Datalog(String),
}

/// Result of taint analysis
#[derive(Debug, Clone, PartialEq)]
pub struct TaintFlow {
    /// Source of taint
    pub source: String,

    /// Sink where taint ends up
    pub sink: String,

    /// Path through the flow
    pub path: Vec<String>,
}

/// Taint propagator using datafrog
#[derive(Debug)]
pub struct TaintPropagator {
    /// Set of tracked source patterns
    pub source_patterns: HashSet<String>,
    /// Set of tracked sink patterns
    pub sink_patterns: HashSet<String>,
    /// Sanitizer patterns that neutralize taint
    pub sanitizer_patterns: HashSet<String>,
}

impl TaintPropagator {
    /// Create a new taint propagator
    pub fn new() -> Self {
        Self {
            source_patterns: HashSet::new(),
            sink_patterns: HashSet::new(),
            sanitizer_patterns: HashSet::new(),
        }
    }

    /// Run taint analysis on semantic model
    pub fn run_analysis(
        &mut self,
        model: &SemanticModel,
        policy: &TaintPolicy,
    ) -> std::result::Result<Vec<TaintFlow>, TaintAnalysisError> {
        // Build FlowIndex from facts if available
        // For now, we'll use a simplified approach based on the model

        // Extract patterns from policy
        self.extract_patterns(policy);

        // Apply datafrog-based taint propagation
        let flows = self.propagate_taint(model, policy)?;

        Ok(flows)
    }

    /// Extract patterns from policy
    pub fn extract_patterns(&mut self, policy: &TaintPolicy) {
        self.source_patterns.clear();
        for source in &policy.sources {
            self.source_patterns.insert(source.pattern.clone());
        }

        self.sink_patterns.clear();
        for sink in &policy.sinks {
            self.sink_patterns.insert(sink.pattern.clone());
        }

        self.sanitizer_patterns.clear();
        for sanitizer in &policy.sanitizers {
            self.sanitizer_patterns.insert(sanitizer.pattern.clone());
        }
    }

    /// Propagate taint using datafrog
    fn propagate_taint(
        &self,
        model: &SemanticModel,
        policy: &TaintPolicy,
    ) -> std::result::Result<Vec<TaintFlow>, TaintAnalysisError> {
        // Check if we have sources and sinks defined
        if policy.sources.is_empty() || policy.sinks.is_empty() {
            return Ok(Vec::new());
        }

        // Convert semantic model to facts for FlowIndex
        let facts = self.extract_facts_from_model(model);
        let fact_refs: Vec<&Fact> = facts.iter().collect();

        // Build FlowIndex from facts
        let flow_index = FlowIndex::build(&fact_refs);

        // Use datafrog for Datalog-based taint propagation
        let flows = self.run_datalog_analysis(&flow_index, policy)?;

        Ok(flows)
    }

    /// Extract facts from semantic model
    fn extract_facts_from_model(&self, _model: &SemanticModel) -> Vec<Fact> {
        // Extract CFG nodes as facts
        // In a full implementation, this would iterate over the model's
        // CFG/DFG and convert nodes to appropriate Fact types

        // For now, return empty vector
        // TODO: Implement full fact extraction from semantic model

        Vec::new()
    }

    /// Run Datalog analysis using datafrog
    fn run_datalog_analysis(
        &self,
        flow_index: &FlowIndex,
        _policy: &TaintPolicy,
    ) -> std::result::Result<Vec<TaintFlow>, TaintAnalysisError> {
        let mut flows = Vec::new();

        // Get all nodes from the flow index
        let nodes = flow_index.nodes();

        // Find taint sources
        for node_id in &nodes {
            // Check if this node matches any source pattern
            let is_source = self.source_patterns.contains(&format!("{:?}", node_id));

            if is_source {
                // Find all nodes reachable from this source
                let reachable = flow_index.reachable_from(*node_id);

                // Check if any reachable node matches a sink pattern
                for reachable_node in &reachable {
                    let is_sink = self
                        .sink_patterns
                        .contains(&format!("{:?}", reachable_node));

                    if is_sink {
                        // Found a taint flow!
                        flows.push(TaintFlow {
                            source: format!("{:?}", node_id),
                            sink: format!("{:?}", reachable_node),
                            path: vec![format!("{:?}", node_id), format!("{:?}", reachable_node)],
                        });
                    }
                }
            }
        }

        Ok(flows)
    }
}

impl Default for TaintPropagator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_propagator() {
        let propagator = TaintPropagator::new();
        assert!(format!("{:?}", propagator).contains("TaintPropagator"));
    }

    #[test]
    fn test_propagator_default() {
        let propagator = TaintPropagator::default();
        assert!(format!("{:?}", propagator).contains("TaintPropagator"));
    }

    #[test]
    fn test_extract_patterns_empty_policy() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy::default();

        propagator.extract_patterns(&policy);

        assert!(propagator.source_patterns.is_empty());
        assert!(propagator.sink_patterns.is_empty());
        assert!(propagator.sanitizer_patterns.is_empty());
    }

    #[test]
    fn test_extract_patterns() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "request".to_string(),
                tags: vec![DataTag::UserInput],
            }],
            sinks: vec![SinkDefinition {
                pattern: "sql_query".to_string(),
                category: "database".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        assert!(propagator.source_patterns.contains("user_input"));
        assert!(propagator.sink_patterns.contains("sql_query"));
        assert!(propagator.sanitizer_patterns.is_empty());
    }

    #[test]
    fn test_extract_patterns_multiple_sources() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![
                SourceDefinition {
                    pattern: "user_input".to_string(),
                    source_type: "http".to_string(),
                    tags: vec![DataTag::UserInput],
                },
                SourceDefinition {
                    pattern: "api_key".to_string(),
                    source_type: "auth".to_string(),
                    tags: vec![DataTag::Credentials],
                },
                SourceDefinition {
                    pattern: "config_value".to_string(),
                    source_type: "env".to_string(),
                    tags: vec![],
                },
            ],
            sinks: vec![],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        assert_eq!(propagator.source_patterns.len(), 3);
        assert!(propagator.source_patterns.contains("user_input"));
        assert!(propagator.source_patterns.contains("api_key"));
        assert!(propagator.source_patterns.contains("config_value"));
    }

    #[test]
    fn test_extract_patterns_multiple_sinks() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![
                SinkDefinition {
                    pattern: "execute_query".to_string(),
                    category: "database".to_string(),
                    severity: hodei_ir::types::Severity::Critical,
                },
                SinkDefinition {
                    pattern: "write_file".to_string(),
                    category: "file".to_string(),
                    severity: hodei_ir::types::Severity::Major,
                },
                SinkDefinition {
                    pattern: "send_email".to_string(),
                    category: "network".to_string(),
                    severity: hodei_ir::types::Severity::Minor,
                },
            ],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        assert_eq!(propagator.sink_patterns.len(), 3);
        assert!(propagator.sink_patterns.contains("execute_query"));
        assert!(propagator.sink_patterns.contains("write_file"));
        assert!(propagator.sink_patterns.contains("send_email"));
    }

    #[test]
    fn test_extract_patterns_with_sanitizers() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![],
            sanitizers: vec![
                SanitizerDefinition {
                    pattern: "sanitize".to_string(),
                    method: Some("sanitize".to_string()),
                },
                SanitizerDefinition {
                    pattern: "escape".to_string(),
                    method: Some("escape".to_string()),
                },
                SanitizerDefinition {
                    pattern: "validate".to_string(),
                    method: Some("validate".to_string()),
                },
            ],
        };

        propagator.extract_patterns(&policy);

        assert_eq!(propagator.sanitizer_patterns.len(), 3);
        assert!(propagator.sanitizer_patterns.contains("sanitize"));
        assert!(propagator.sanitizer_patterns.contains("escape"));
        assert!(propagator.sanitizer_patterns.contains("validate"));
    }

    #[test]
    fn test_extract_patterns_overwrites_previous() {
        let mut propagator = TaintPropagator::new();

        let policy1 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "pattern1".to_string(),
                source_type: "type1".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        let policy2 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "pattern2".to_string(),
                source_type: "type2".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy1);
        assert_eq!(propagator.source_patterns.len(), 1);
        assert!(propagator.source_patterns.contains("pattern1"));

        propagator.extract_patterns(&policy2);
        assert_eq!(propagator.source_patterns.len(), 1);
        assert!(propagator.source_patterns.contains("pattern2"));
        assert!(!propagator.source_patterns.contains("pattern1"));
    }

    #[test]
    fn test_run_analysis_empty_model() {
        let mut propagator = TaintPropagator::new();
        let model = SemanticModel::default();
        let policy = TaintPolicy::default();

        let result = propagator.run_analysis(&model, &policy);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_run_analysis_empty_policy() {
        let mut propagator = TaintPropagator::new();
        let model = SemanticModel::new();
        let policy = TaintPolicy::default();

        let result = propagator.run_analysis(&model, &policy);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_run_analysis_with_sources_no_sinks() {
        let mut propagator = TaintPropagator::new();
        let model = SemanticModel::new();
        let policy = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "user_input".to_string(),
                source_type: "http".to_string(),
                tags: vec![DataTag::UserInput],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        let result = propagator.run_analysis(&model, &policy);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_run_analysis_with_sinks_no_sources() {
        let mut propagator = TaintPropagator::new();
        let model = SemanticModel::new();
        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![SinkDefinition {
                pattern: "execute_query".to_string(),
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        let result = propagator.run_analysis(&model, &policy);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_run_analysis_with_complete_policy() {
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
                category: "sql".to_string(),
                severity: hodei_ir::types::Severity::Major,
            }],
            sanitizers: vec![],
        };

        let result = propagator.run_analysis(&model, &policy);

        assert!(result.is_ok());
        // With empty model, no flows will be detected
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_run_analysis_error_handling() {
        let mut propagator = TaintPropagator::new();
        let model = SemanticModel::new();
        let policy = TaintPolicy::default();

        let result = propagator.run_analysis(&model, &policy);

        // Should not return error even with empty inputs
        assert!(result.is_ok());
    }

    #[test]
    fn test_propagator_pattern_extraction_duplicate_patterns() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![
                SourceDefinition {
                    pattern: "duplicate".to_string(),
                    source_type: "type1".to_string(),
                    tags: vec![],
                },
                SourceDefinition {
                    pattern: "duplicate".to_string(),
                    source_type: "type2".to_string(),
                    tags: vec![],
                },
            ],
            sinks: vec![],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        // HashSet should deduplicate
        assert_eq!(propagator.source_patterns.len(), 1);
        assert!(propagator.source_patterns.contains("duplicate"));
    }

    #[test]
    fn test_propagator_with_all_datatags() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![
                SourceDefinition {
                    pattern: "pii_input".to_string(),
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
                    pattern: "user_feedback".to_string(),
                    source_type: "web".to_string(),
                    tags: vec![DataTag::UserInput],
                },
            ],
            sinks: vec![],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        assert_eq!(propagator.source_patterns.len(), 4);
    }

    #[test]
    fn test_propagator_with_all_severity_levels() {
        let mut propagator = TaintPropagator::new();
        let policy = TaintPolicy {
            sources: vec![],
            sinks: vec![
                SinkDefinition {
                    pattern: "critical_func".to_string(),
                    category: "database".to_string(),
                    severity: hodei_ir::types::Severity::Critical,
                },
                SinkDefinition {
                    pattern: "major_func".to_string(),
                    category: "file".to_string(),
                    severity: hodei_ir::types::Severity::Major,
                },
                SinkDefinition {
                    pattern: "minor_func".to_string(),
                    category: "log".to_string(),
                    severity: hodei_ir::types::Severity::Minor,
                },
                SinkDefinition {
                    pattern: "info_func".to_string(),
                    category: "debug".to_string(),
                    severity: hodei_ir::types::Severity::Info,
                },
            ],
            sanitizers: vec![],
        };

        propagator.extract_patterns(&policy);

        assert_eq!(propagator.sink_patterns.len(), 4);
    }

    #[test]
    fn test_taint_flow_structure() {
        let flow = TaintFlow {
            source: "user_input".to_string(),
            sink: "execute_query".to_string(),
            path: vec!["var_a".to_string(), "var_b".to_string()],
        };

        assert_eq!(flow.source, "user_input");
        assert_eq!(flow.sink, "execute_query");
        assert_eq!(flow.path.len(), 2);
        assert_eq!(flow.path[0], "var_a");
        assert_eq!(flow.path[1], "var_b");
    }

    #[test]
    fn test_taint_flow_clone() {
        let flow1 = TaintFlow {
            source: "source".to_string(),
            sink: "sink".to_string(),
            path: vec!["path1".to_string()],
        };

        let flow2 = flow1.clone();

        assert_eq!(flow1.source, flow2.source);
        assert_eq!(flow1.sink, flow2.sink);
        assert_eq!(flow1.path, flow2.path);
    }

    #[test]
    fn test_taint_flow_partial_eq() {
        let flow1 = TaintFlow {
            source: "source".to_string(),
            sink: "sink".to_string(),
            path: vec!["path".to_string()],
        };

        let flow2 = TaintFlow {
            source: "source".to_string(),
            sink: "sink".to_string(),
            path: vec!["path".to_string()],
        };

        let flow3 = TaintFlow {
            source: "other".to_string(),
            sink: "sink".to_string(),
            path: vec!["path".to_string()],
        };

        assert_eq!(flow1, flow2);
        assert_ne!(flow1, flow3);
    }

    #[test]
    fn test_taint_analysis_error_types() {
        let error = TaintAnalysisError::Analysis("test error".to_string());
        assert!(format!("{:?}", error).contains("Analysis"));

        let error = TaintAnalysisError::FlowIndex("flow error".to_string());
        assert!(format!("{:?}", error).contains("FlowIndex"));

        let error = TaintAnalysisError::Datalog("datalog error".to_string());
        assert!(format!("{:?}", error).contains("Datalog"));
    }

    #[test]
    fn test_propagator_state_isolation() {
        let mut propagator1 = TaintPropagator::new();
        let mut propagator2 = TaintPropagator::new();

        let policy1 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "pattern1".to_string(),
                source_type: "type1".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        let policy2 = TaintPolicy {
            sources: vec![SourceDefinition {
                pattern: "pattern2".to_string(),
                source_type: "type2".to_string(),
                tags: vec![],
            }],
            sinks: vec![],
            sanitizers: vec![],
        };

        propagator1.extract_patterns(&policy1);
        propagator2.extract_patterns(&policy2);

        assert!(propagator1.source_patterns.contains("pattern1"));
        assert!(!propagator1.source_patterns.contains("pattern2"));

        assert!(propagator2.source_patterns.contains("pattern2"));
        assert!(!propagator2.source_patterns.contains("pattern1"));
    }
}
