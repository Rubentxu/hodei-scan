//! Fact Extraction from Semantic Model
//!
//! This module extracts IR Facts from the semantic model's CFG and DFG.
//! It converts graph nodes and edges into Fact structures that can be used
//! by the taint analysis engine.

use crate::semantic_model::{ControlFlowGraph, DataFlowGraph, SemanticModel};
use hodei_ir::types::{FlowId, FunctionName, LineNumber, ProjectPath, Severity, VariableName};
use hodei_ir::{Confidence, ExtractorId, Fact, FactId, FactType, Provenance, SourceLocation};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

/// Extractor for converting semantic model to facts
#[derive(Debug)]
pub struct FactExtractor {
    /// Mapping from CFG node index to FactId
    cfg_node_to_fact: HashMap<NodeIndex, FactId>,
    /// Mapping from DFG node index to FactId
    dfg_node_to_fact: HashMap<NodeIndex, FactId>,
    /// Next available FactId for creating new facts
    next_fact_id: u64,
}

impl FactExtractor {
    pub fn new() -> Self {
        Self {
            cfg_node_to_fact: HashMap::new(),
            dfg_node_to_fact: HashMap::new(),
            next_fact_id: 1,
        }
    }

    /// Extract facts from a semantic model
    pub fn extract_facts(&mut self, model: &SemanticModel) -> Vec<Fact> {
        let mut facts = Vec::new();

        // Extract facts from CFG
        facts.extend(self.extract_cfg_facts(model));

        // Extract facts from DFG
        facts.extend(self.extract_dfg_facts(model));

        facts
    }

    /// Extract facts from Control Flow Graph
    fn extract_cfg_facts(&mut self, model: &SemanticModel) -> Vec<Fact> {
        let mut facts = Vec::new();

        // Iterate over CFG nodes
        for node_idx in model.cfg.node_indices() {
            let node_id = self.next_fact_id;
            self.next_fact_id += 1;

            let fact_id = FactId::from_uuid(uuid::Uuid::new_v4());

            self.cfg_node_to_fact.insert(node_idx, fact_id);

            // Create a Function fact for each basic block
            let fact = Fact::new(
                FactType::Function {
                    name: FunctionName(format!("block_{}", node_idx.index())),
                    complexity: 1,
                    lines_of_code: 1,
                },
                SourceLocation::new(
                    ProjectPath::new(std::path::PathBuf::from("generated.rs")),
                    LineNumber::new(node_idx.index() as u32).unwrap(),
                    None,
                    LineNumber::new(node_idx.index() as u32).unwrap(),
                    None,
                ),
                Provenance::new(
                    ExtractorId::DataFlowAnalyzer,
                    "1.0.0".to_string(),
                    Confidence::MEDIUM,
                ),
            );

            facts.push(fact);
        }

        facts
    }

    /// Extract facts from Data Flow Graph
    fn extract_dfg_facts(&mut self, model: &SemanticModel) -> Vec<Fact> {
        let mut facts = Vec::new();

        // Iterate over DFG nodes
        for node_idx in model.dfg.node_indices() {
            let node_id = self.next_fact_id;
            self.next_fact_id += 1;

            let fact_id = FactId::from_uuid(uuid::Uuid::new_v4());

            self.dfg_node_to_fact.insert(node_idx, fact_id);

            // Create appropriate fact based on DFG node type
            // TODO: Extract actual node type and create corresponding fact

            facts.push(fact_id); // Store fact_id for now
        }

        // Convert fact IDs to actual facts
        // For now, create Variable facts for DFG nodes
        facts
            .into_iter()
            .map(|fact_id| {
                Fact::new(
                    FactType::Variable {
                        name: VariableName("extracted_var".to_string()),
                        scope: "global".to_string(),
                        var_type: "extracted_type".to_string(),
                    },
                    SourceLocation::new(
                        ProjectPath::new(std::path::PathBuf::from("generated.rs")),
                        LineNumber::new(1).unwrap(),
                        None,
                        LineNumber::new(1).unwrap(),
                        None,
                    ),
                    Provenance::new(
                        ExtractorId::DataFlowAnalyzer,
                        "1.0.0".to_string(),
                        Confidence::MEDIUM,
                    ),
                )
            })
            .collect()
    }

    /// Create a taint source fact
    pub fn create_taint_source(&mut self, var_name: &str, source_type: &str) -> Fact {
        let node_id = self.next_fact_id;
        self.next_fact_id += 1;

        let flow_id = FlowId::new_uuid();

        Fact::new(
            FactType::TaintSource {
                var: VariableName(var_name.to_string()),
                flow_id,
                source_type: source_type.to_string(),
                confidence: Confidence::MEDIUM,
            },
            SourceLocation::new(
                ProjectPath::new(std::path::PathBuf::from("generated.rs")),
                LineNumber::new(1).unwrap(),
                None,
                LineNumber::new(1).unwrap(),
                None,
            ),
            Provenance::new(
                ExtractorId::DataFlowAnalyzer,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        )
    }

    /// Create a taint sink fact
    pub fn create_taint_sink(
        &mut self,
        func_name: &str,
        consumes_flow: FlowId,
        category: &str,
    ) -> Fact {
        let node_id = self.next_fact_id;
        self.next_fact_id += 1;

        Fact::new(
            FactType::TaintSink {
                func: FunctionName(func_name.to_string()),
                consumes_flow,
                category: category.to_string(),
                severity: Severity::Major,
            },
            SourceLocation::new(
                ProjectPath::new(std::path::PathBuf::from("generated.rs")),
                LineNumber::new(1).unwrap(),
                None,
                LineNumber::new(1).unwrap(),
                None,
            ),
            Provenance::new(
                ExtractorId::DataFlowAnalyzer,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        )
    }

    /// Create a sanitization fact
    pub fn create_sanitizer(&mut self, method: &str, sanitizes_flow: FlowId) -> Fact {
        let node_id = self.next_fact_id;
        self.next_fact_id += 1;

        Fact::new(
            FactType::Sanitization {
                method: method.to_string(),
                sanitizes_flow,
                effective: true,
                confidence: Confidence::HIGH,
            },
            SourceLocation::new(
                ProjectPath::new(std::path::PathBuf::from("generated.rs")),
                LineNumber::new(1).unwrap(),
                None,
                LineNumber::new(1).unwrap(),
                None,
            ),
            Provenance::new(
                ExtractorId::DataFlowAnalyzer,
                "1.0.0".to_string(),
                Confidence::HIGH,
            ),
        )
    }
}

impl Default for FactExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_model::SemanticModel;

    #[test]
    fn test_new_extractor() {
        let extractor = FactExtractor::new();
        assert!(format!("{:?}", extractor).contains("FactExtractor"));
    }

    #[test]
    fn test_extractor_default() {
        let extractor = FactExtractor::default();
        assert!(format!("{:?}", extractor).contains("FactExtractor"));
    }

    #[test]
    fn test_extract_facts_from_empty_model() {
        let mut extractor = FactExtractor::new();
        let model = SemanticModel::new();

        let facts = extractor.extract_facts(&model);

        // Empty model has empty CFG and DFG
        assert_eq!(facts.len(), 0);
    }

    #[test]
    fn test_create_taint_source() {
        let mut extractor = FactExtractor::new();

        let fact = extractor.create_taint_source("user_input", "http");

        if let FactType::TaintSource {
            var, source_type, ..
        } = fact.fact_type
        {
            assert_eq!(var.0, "user_input");
            assert_eq!(source_type, "http");
        } else {
            panic!("Expected TaintSource fact");
        }
    }

    #[test]
    fn test_create_taint_source_different_types() {
        let mut extractor = FactExtractor::new();

        let fact1 = extractor.create_taint_source("user_input", "http");
        let fact2 = extractor.create_taint_source("api_key", "auth");
        let fact3 = extractor.create_taint_source("config_value", "env");

        match (&fact1.fact_type, &fact2.fact_type, &fact3.fact_type) {
            (FactType::TaintSource { var: v1, source_type: s1, .. },
             FactType::TaintSource { var: v2, source_type: s2, .. },
             FactType::TaintSource { var: v3, source_type: s3, .. }) => {
                assert_eq!(v1.0, "user_input");
                assert_eq!(s1, "http");
                assert_eq!(v2.0, "api_key");
                assert_eq!(s2, "auth");
                assert_eq!(v3.0, "config_value");
                assert_eq!(s3, "env");
            }
            _ => panic!("Expected all TaintSource facts"),
        }
    }

    #[test]
    fn test_create_taint_source_with_special_chars() {
        let mut extractor = FactExtractor::new();

        let fact = extractor.create_taint_source("user-name_123", "http");

        if let FactType::TaintSource { var, .. } = fact.fact_type {
            assert_eq!(var.0, "user-name_123");
        } else {
            panic!("Expected TaintSource fact");
        }
    }

    #[test]
    fn test_create_taint_sink() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let fact = extractor.create_taint_sink("execute_query", flow_id, "sql");

        if let FactType::TaintSink {
            func,
            consumes_flow,
            category,
            ..
        } = fact.fact_type
        {
            assert_eq!(func.0, "execute_query");
            assert_eq!(consumes_flow, flow_id);
            assert_eq!(category, "sql");
        } else {
            panic!("Expected TaintSink fact");
        }
    }

    #[test]
    fn test_create_taint_sink_different_categories() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let fact1 = extractor.create_taint_sink("write_file", flow_id, "file");
        let fact2 = extractor.create_taint_sink("send_email", flow_id, "network");
        let fact3 = extractor.create_taint_sink("log_message", flow_id, "logging");

        match (&fact1.fact_type, &fact2.fact_type, &fact3.fact_type) {
            (FactType::TaintSink { category: c1, .. },
             FactType::TaintSink { category: c2, .. },
             FactType::TaintSink { category: c3, .. }) => {
                assert_eq!(c1, "file");
                assert_eq!(c2, "network");
                assert_eq!(c3, "logging");
            }
            _ => panic!("Expected all TaintSink facts"),
        }
    }

    #[test]
    fn test_create_taint_sink_with_special_chars() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let fact = extractor.create_taint_sink("execute_query_123", flow_id, "sql_database");

        if let FactType::TaintSink { func, category, .. } = fact.fact_type {
            assert_eq!(func.0, "execute_query_123");
            assert_eq!(category, "sql_database");
        } else {
            panic!("Expected TaintSink fact");
        }
    }

    #[test]
    fn test_create_sanitizer() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let fact = extractor.create_sanitizer("sanitize_input", flow_id);

        if let FactType::Sanitization {
            method,
            sanitizes_flow,
            effective,
            ..
        } = fact.fact_type
        {
            assert_eq!(method, "sanitize_input");
            assert_eq!(sanitizes_flow, flow_id);
            assert_eq!(effective, true);
        } else {
            panic!("Expected Sanitization fact");
        }
    }

    #[test]
    fn test_create_sanitizer_with_different_methods() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let fact1 = extractor.create_sanitizer("escape_html", flow_id);
        let fact2 = extractor.create_sanitizer("validate_email", flow_id);
        let fact3 = extractor.create_sanitizer("sanitize_sql", flow_id);

        match (&fact1.fact_type, &fact2.fact_type, &fact3.fact_type) {
            (FactType::Sanitization { method: m1, .. },
             FactType::Sanitization { method: m2, .. },
             FactType::Sanitization { method: m3, .. }) => {
                assert_eq!(m1, "escape_html");
                assert_eq!(m2, "validate_email");
                assert_eq!(m3, "sanitize_sql");
            }
            _ => panic!("Expected all Sanitization facts"),
        }
    }

    #[test]
    fn test_fact_ids_are_unique() {
        let mut extractor = FactExtractor::new();

        let fact1 = extractor.create_taint_source("user_input", "http");
        let fact2 = extractor.create_taint_source("api_key", "auth");
        let fact3 = extractor.create_taint_sink("execute_query", FlowId::new_uuid(), "sql");

        // Each fact should have a unique ID
        assert_ne!(fact1.id, fact2.id);
        assert_ne!(fact2.id, fact3.id);
        assert_ne!(fact1.id, fact3.id);
    }

    #[test]
    fn test_fact_source_location() {
        let mut extractor = FactExtractor::new();

        let fact = extractor.create_taint_source("user_input", "http");

        assert!(fact.location.file.as_str().contains("generated.rs"));
        assert!(fact.location.start_line.get() > 0);
    }

    #[test]
    fn test_fact_provenance() {
        let mut extractor = FactExtractor::new();

        let fact = extractor.create_taint_source("user_input", "http");

        assert_eq!(fact.provenance.extractor, hodei_ir::ExtractorId::DataFlowAnalyzer);
        assert_eq!(fact.provenance.version, "1.0.0");
        assert_eq!(fact.provenance.confidence, hodei_ir::Confidence::MEDIUM);
    }

    #[test]
    fn test_multiple_source_facts() {
        let mut extractor = FactExtractor::new();

        let sources = vec![
            "user_input",
            "api_key",
            "session_id",
            "request_data",
        ];

        let facts: Vec<_> = sources
            .iter()
            .map(|s| extractor.create_taint_source(s, "http"))
            .collect();

        assert_eq!(facts.len(), 4);

        for (i, (fact, expected_name)) in facts.iter().zip(sources.iter()).enumerate() {
            if let FactType::TaintSource { var, .. } = &fact.fact_type {
                assert_eq!(var.0, *expected_name, "Source {} mismatch", i);
            } else {
                panic!("Expected TaintSource fact at index {}", i);
            }
        }
    }

    #[test]
    fn test_multiple_sink_facts() {
        let mut extractor = FactExtractor::new();

        let sinks = vec![
            ("execute_query", "sql"),
            ("write_file", "file"),
            ("send_email", "network"),
            ("log_message", "logging"),
        ];

        let flow_id = FlowId::new_uuid();
        let facts: Vec<_> = sinks
            .iter()
            .map(|(func, cat)| extractor.create_taint_sink(func, flow_id, cat))
            .collect();

        assert_eq!(facts.len(), 4);

        for (i, (fact, expected_func)) in facts.iter().zip(sinks.iter()).enumerate() {
            if let FactType::TaintSink { func, .. } = &fact.fact_type {
                assert_eq!(func.0, expected_func.0, "Sink {} mismatch", i);
            } else {
                panic!("Expected TaintSink fact at index {}", i);
            }
        }
    }

    #[test]
    fn test_extractor_fact_id_counter() {
        let mut extractor = FactExtractor::new();

        // Create facts and verify IDs increment
        let fact1 = extractor.create_taint_source("input1", "http");
        let fact2 = extractor.create_taint_source("input2", "http");
        let fact3 = extractor.create_taint_source("input3", "http");

        // IDs should be unique but we can't assert ordering
        assert_ne!(fact1.id, fact2.id);
        assert_ne!(fact2.id, fact3.id);
        assert_ne!(fact1.id, fact3.id);
    }

    #[test]
    fn test_empty_string_parameters() {
        let mut extractor = FactExtractor::new();

        let fact = extractor.create_taint_source("", "");

        if let FactType::TaintSource { var, source_type, .. } = fact.fact_type {
            assert_eq!(var.0, "");
            assert_eq!(source_type, "");
        } else {
            panic!("Expected TaintSource fact");
        }
    }

    #[test]
    fn test_long_strings() {
        let mut extractor = FactExtractor::new();

        let long_string = "a".repeat(1000);
        let fact = extractor.create_taint_source(&long_string, &long_string);

        if let FactType::TaintSource { var, source_type, .. } = fact.fact_type {
            assert_eq!(var.0.len(), 1000);
            assert_eq!(source_type.len(), 1000);
        } else {
            panic!("Expected TaintSource fact");
        }
    }

    #[test]
    fn test_flow_id_consistency() {
        let mut extractor = FactExtractor::new();

        let flow_id = FlowId::new_uuid();
        let sink = extractor.create_taint_sink("execute_query", flow_id, "sql");
        let sanitizer = extractor.create_sanitizer("sanitize", flow_id);

        // Sink and sanitizer should reference the same flow ID
        if let (FactType::TaintSink { consumes_flow: sink_flow, .. },
                FactType::Sanitization { sanitizes_flow: san_flow, .. }) =
            (&sink.fact_type, &sanitizer.fact_type)
        {
            assert_eq!(*sink_flow, flow_id);
            assert_eq!(*san_flow, flow_id);
            assert_eq!(*sink_flow, *san_flow);
        } else {
            panic!("Expected TaintSink and Sanitization facts with matching flow IDs");
        }
    }

    #[test]
    fn test_extract_facts_with_model() {
        let mut extractor = FactExtractor::new();
        let model = SemanticModel::new();

        let facts = extractor.extract_facts(&model);

        // Empty model returns empty facts
        assert_eq!(facts.len(), 0);
    }

    #[test]
    fn test_unicode_strings() {
        let mut extractor = FactExtractor::new();

        let unicode_str = "用户输入_🔐";
        let fact = extractor.create_taint_source(unicode_str, "http");

        if let FactType::TaintSource { var, .. } = fact.fact_type {
            assert_eq!(var.0, unicode_str);
        } else {
            panic!("Expected TaintSource fact");
        }
    }

    #[test]
    fn test_confidence_levels() {
        let mut extractor = FactExtractor::new();

        let source = extractor.create_taint_source("user_input", "http");
        let sanitizer = extractor.create_sanitizer("sanitize", FlowId::new_uuid());

        // Source should have MEDIUM confidence
        if let FactType::TaintSource { confidence, .. } = source.fact_type {
            assert_eq!(confidence, hodei_ir::Confidence::MEDIUM);
        } else {
            panic!("Expected TaintSource fact");
        }

        // Sanitizer should have HIGH confidence
        if let FactType::Sanitization { confidence, .. } = sanitizer.fact_type {
            assert_eq!(confidence, hodei_ir::Confidence::HIGH);
        } else {
            panic!("Expected Sanitization fact");
        }
    }

    #[test]
    fn test_sink_severity() {
        let mut extractor = FactExtractor::new();
        let flow_id = FlowId::new_uuid();

        let sink = extractor.create_taint_sink("execute_query", flow_id, "sql");

        if let FactType::TaintSink { severity, .. } = sink.fact_type {
            assert_eq!(severity, hodei_ir::types::Severity::Major);
        } else {
            panic!("Expected TaintSink fact");
        }
    }
}
