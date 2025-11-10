//! hodei-ir: Intermediate Representation core types

#![warn(missing_docs)]

pub mod types;
pub mod validator;

pub use types::*;
pub use validator::*;

/// The main FactType enum (17 atomic variants)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FactType {
    // SAST (6 variants)
    TaintSource {
        var: VariableName,
        flow_id: FlowId,
        source_type: String,
        confidence: Confidence,
    },
    TaintSink {
        func: FunctionName,
        consumes_flow: FlowId,
        category: String,
        severity: Severity,
    },
    Sanitization {
        method: String,
        sanitizes_flow: FlowId,
        effective: bool,
        confidence: Confidence,
    },
    UnsafeCall {
        function_name: FunctionName,
        reason: String,
        severity: Severity,
    },
    CryptographicOperation {
        algorithm: String,
        key_length: Option<u32>,
        secure: bool,
        recommendation: Option<String>,
    },
    Vulnerability {
        cwe_id: Option<String>,
        owasp_category: Option<String>,
        severity: Severity,
        cvss_score: Option<f32>,
        description: String,
        confidence: Confidence,
    },

    // Quality (4 variants)
    Function {
        name: FunctionName,
        complexity: u32,
        lines_of_code: u32,
    },
    Variable {
        name: VariableName,
        scope: String,
        var_type: String,
    },
    CodeSmell {
        smell_type: String,
        severity: Severity,
        message: String,
    },
    ComplexityViolation {
        metric: String,
        actual: u32,
        threshold: u32,
    },

    // SCA (3 variants)
    Dependency {
        name: String,
        version: String,
        ecosystem: Ecosystem,
    },
    DependencyVulnerability {
        dependency: String,
        cve_id: Option<String>,
        severity: Severity,
        cvss_score: Option<f32>,
        description: String,
    },
    License {
        dependency: String,
        license_type: String,
        compatible: bool,
    },

    // Coverage (3 variants)
    UncoveredLine {
        location: SourceLocation,
        coverage: f32,
    },
    LowTestCoverage {
        file: ProjectPath,
        percentage: f32,
        total_lines: u32,
        covered_lines: u32,
    },
    CoverageStats {
        scope: String,
        path: ProjectPath,
        line_coverage: f32,
        branch_coverage: f32,
    },
}

/// A single fact
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fact {
    pub id: FactId,
    pub fact_type: FactType,
    pub location: SourceLocation,
    pub provenance: Provenance,
}

impl Fact {
    pub fn new(fact_type: FactType, location: SourceLocation, provenance: Provenance) -> Self {
        Self {
            id: FactId(uuid::Uuid::new_v4()),
            fact_type,
            location,
            provenance,
        }
    }
}

/// Intermediate representation container
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IntermediateRepresentation {
    pub facts: Vec<Fact>,
    pub metadata: ProjectMetadata,
    pub schema_version: String,
}

impl IntermediateRepresentation {
    pub fn new(metadata: ProjectMetadata) -> Self {
        Self {
            facts: Vec::new(),
            metadata,
            schema_version: "3.2.0".to_string(),
        }
    }
    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.push(fact);
    }
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

/// Project metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    pub root_path: ProjectPath,
}

impl ProjectMetadata {
    pub fn new(name: String, version: String, root_path: ProjectPath) -> Self {
        Self {
            name,
            version,
            root_path,
        }
    }
}
