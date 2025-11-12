//! hodei-ir: Intermediate Representation core types

#![warn(missing_docs)]

pub mod capnp_impl;
pub mod capnp_serialization;
pub mod custom_fact_tests;
pub mod fact_type_index;
pub mod interning;
pub mod migration;
pub mod plugin_schema_registry;
pub mod types;
pub mod validator;
pub mod zero_copy;

pub use capnp_serialization::*;
pub use fact_type_index::*;
pub use migration::*;
pub use plugin_schema_registry::*;
pub use types::*;
pub use validator::*;
pub use zero_copy::*;

/// Type alias for backward compatibility with tests
/// A Finding is the same as a Fact in this codebase
pub type Finding = Fact;

/// A FindingSet is just a collection of findings (facts)
pub type FindingSet = Vec<Fact>;

/// The main FactType enum (17 atomic variants)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FactType {
    // SAST (6 variants)
    /// Taint source - identifies where untrusted input enters the system
    TaintSource {
        /// Variable that receives tainted data
        var: VariableName,
        /// Flow ID for tracking data flow
        flow_id: FlowId,
        /// Type of taint source (e.g., user_input, file_read, network)
        source_type: String,
        /// Confidence level of the taint analysis
        confidence: Confidence,
    },
    /// Taint sink - identifies where tainted data is used unsafely
    TaintSink {
        /// Function that consumes tainted data
        func: FunctionName,
        /// Flow ID that this sink consumes
        consumes_flow: FlowId,
        /// Category of vulnerability (e.g., SQL injection, XSS)
        category: String,
        /// Severity level of the sink
        severity: Severity,
    },
    /// Sanitization - identifies where data is sanitized
    Sanitization {
        /// Method used for sanitization (e.g., escape_html, base64_encode)
        method: String,
        /// Flow ID that this sanitization handles
        sanitizes_flow: FlowId,
        /// Whether the sanitization is effective
        effective: bool,
        /// Confidence in the sanitization effectiveness
        confidence: Confidence,
    },
    /// Unsafe function call
    UnsafeCall {
        /// Name of the unsafe function
        function_name: FunctionName,
        /// Reason why the call is unsafe
        reason: String,
        /// Severity of the safety issue
        severity: Severity,
    },
    /// Cryptographic operation
    CryptographicOperation {
        /// Algorithm used (e.g., AES, RSA, SHA256)
        algorithm: String,
        /// Key length in bits (None if not applicable)
        key_length: Option<u32>,
        /// Whether the operation uses secure parameters
        secure: bool,
        /// Recommendation for improvement (None if already secure)
        recommendation: Option<String>,
    },
    /// General vulnerability
    Vulnerability {
        /// CWE (Common Weakness Enumeration) ID
        cwe_id: Option<String>,
        /// OWASP category
        owasp_category: Option<String>,
        /// Severity of the vulnerability
        severity: Severity,
        /// CVSS score (Common Vulnerability Scoring System)
        cvss_score: Option<f32>,
        /// Detailed description of the vulnerability
        description: String,
        /// Confidence in the finding
        confidence: Confidence,
    },

    // Quality (4 variants)
    /// Function information
    Function {
        /// Name of the function
        name: FunctionName,
        /// Cyclomatic complexity
        complexity: u32,
        /// Number of lines of code
        lines_of_code: u32,
    },
    /// Variable information
    Variable {
        /// Name of the variable
        name: VariableName,
        /// Scope where the variable is defined
        scope: String,
        /// Type of the variable
        var_type: String,
    },
    /// Code smell
    CodeSmell {
        /// Type of code smell
        smell_type: String,
        /// Severity of the code smell
        severity: Severity,
    },
    /// Complexity violation
    ComplexityViolation {
        /// Metric that exceeded threshold (e.g., cyclomatic_complexity, lines_of_code)
        metric: String,
        /// Actual measured value
        actual: u32,
        /// Threshold that was exceeded
        threshold: u32,
    },

    // SCA (3 variants)
    /// Dependency information
    Dependency {
        /// Name of the dependency
        name: String,
        /// Version of the dependency
        version: String,
        /// Package ecosystem
        ecosystem: Ecosystem,
    },
    /// Vulnerability in a dependency
    DependencyVulnerability {
        /// Dependency that has the vulnerability
        dependency: String,
        /// CVE ID if available
        cve_id: Option<String>,
        /// Severity of the vulnerability
        severity: Severity,
        /// CVSS score
        cvss_score: Option<f32>,
        /// Description of the vulnerability
        description: String,
    },
    /// License information
    License {
        /// Dependency that this license applies to
        dependency: String,
        /// Type of license (e.g., MIT, Apache-2.0, GPL-3.0)
        license_type: String,
        /// Whether the license is compatible with the project
        compatible: bool,
    },

    // Coverage (3 variants)
    /// Uncovered line of code
    UncoveredLine {
        /// Location of the uncovered line
        location: SourceLocation,
        /// Coverage tool that identified this
        coverage: String,
    },
    /// Low test coverage for a file
    LowTestCoverage {
        /// File with low coverage
        file: ProjectPath,
        /// Percentage of lines covered
        percentage: u32,
        /// Total number of lines
        total_lines: u32,
        /// Number of covered lines
        covered_lines: u32,
    },
    /// Coverage statistics
    CoverageStats {
        /// Scope of the statistics (file, module, project)
        scope: String,
        /// Path to the file or module
        path: ProjectPath,
        /// Line coverage percentage
        line_coverage: u32,
        /// Branch coverage percentage
        branch_coverage: u32,
    },

    // Custom FactType for plugin extensibility
    /// Custom fact type defined by plugins
    Custom {
        /// Unique identifier for the custom fact type (e.g., "terraform::aws::insecure_s3_bucket")
        discriminant: String,
        /// Data fields for the custom fact
        data: std::collections::HashMap<String, FactValue>,
    },
}

impl std::fmt::Display for FactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::hash::Hash for FactType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the discriminant
        std::mem::discriminant(self).hash(state);

        match self {
            FactType::TaintSource {
                var,
                flow_id,
                source_type,
                confidence,
            } => {
                var.hash(state);
                flow_id.hash(state);
                source_type.hash(state);
                confidence.hash(state);
            }
            FactType::TaintSink {
                func,
                consumes_flow,
                category,
                severity,
            } => {
                func.hash(state);
                consumes_flow.hash(state);
                category.hash(state);
                severity.hash(state);
            }
            FactType::Sanitization {
                method,
                sanitizes_flow,
                effective,
                confidence,
            } => {
                method.hash(state);
                sanitizes_flow.hash(state);
                effective.hash(state);
                confidence.hash(state);
            }
            FactType::UnsafeCall {
                function_name,
                reason,
                severity,
            } => {
                function_name.hash(state);
                reason.hash(state);
                severity.hash(state);
            }
            FactType::CryptographicOperation {
                algorithm,
                key_length,
                secure,
                recommendation,
            } => {
                algorithm.hash(state);
                key_length.hash(state);
                secure.hash(state);
                recommendation.hash(state);
            }
            FactType::Vulnerability {
                cwe_id,
                owasp_category,
                severity,
                cvss_score,
                description,
                confidence,
            } => {
                cwe_id.hash(state);
                owasp_category.hash(state);
                severity.hash(state);
                // Hash f32 as raw bytes
                if let Some(score) = cvss_score {
                    let bits = score.to_bits();
                    bits.hash(state);
                }
                description.hash(state);
                confidence.hash(state);
            }
            FactType::Function {
                name,
                complexity,
                lines_of_code,
            } => {
                name.hash(state);
                complexity.hash(state);
                lines_of_code.hash(state);
            }
            FactType::Variable {
                name,
                scope,
                var_type,
            } => {
                name.hash(state);
                scope.hash(state);
                var_type.hash(state);
            }
            FactType::CodeSmell {
                smell_type,
                severity,
            } => {
                smell_type.hash(state);
                severity.hash(state);
            }
            FactType::ComplexityViolation {
                metric,
                actual,
                threshold,
            } => {
                metric.hash(state);
                actual.hash(state);
                threshold.hash(state);
            }
            FactType::Dependency {
                name,
                version,
                ecosystem,
            } => {
                name.hash(state);
                version.hash(state);
                ecosystem.hash(state);
            }
            FactType::DependencyVulnerability {
                dependency,
                cve_id,
                severity,
                cvss_score,
                description,
            } => {
                dependency.hash(state);
                cve_id.hash(state);
                severity.hash(state);
                // Hash f32 as raw bytes
                if let Some(score) = cvss_score {
                    score.to_bits().hash(state);
                }
                description.hash(state);
            }
            FactType::License {
                dependency,
                license_type,
                compatible,
            } => {
                dependency.hash(state);
                license_type.hash(state);
                compatible.hash(state);
            }
            FactType::UncoveredLine { location, coverage } => {
                location.hash(state);
                coverage.hash(state);
            }
            FactType::LowTestCoverage {
                file,
                percentage,
                total_lines,
                covered_lines,
            } => {
                file.hash(state);
                percentage.hash(state);
                total_lines.hash(state);
                covered_lines.hash(state);
            }
            FactType::CoverageStats {
                scope,
                path,
                line_coverage,
                branch_coverage,
            } => {
                scope.hash(state);
                path.hash(state);
                line_coverage.hash(state);
                branch_coverage.hash(state);
            }
            FactType::Custom { discriminant, data } => {
                discriminant.hash(state);
                // Hash each entry in the HashMap
                for (k, v) in data {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

impl Eq for FactType {}

impl FactType {
    /// Get the discriminant of this fact type
    pub fn discriminant(&self) -> FactTypeDiscriminant {
        match self {
            FactType::TaintSource { .. } => FactTypeDiscriminant::TaintSource,
            FactType::TaintSink { .. } => FactTypeDiscriminant::TaintSink,
            FactType::Sanitization { .. } => FactTypeDiscriminant::Sanitization,
            FactType::UnsafeCall { .. } => FactTypeDiscriminant::UnsafeCall,
            FactType::CryptographicOperation { .. } => FactTypeDiscriminant::CryptographicOperation,
            FactType::Vulnerability { .. } => FactTypeDiscriminant::Vulnerability,
            FactType::Function { .. } => FactTypeDiscriminant::Function,
            FactType::Variable { .. } => FactTypeDiscriminant::Variable,
            FactType::CodeSmell { .. } => FactTypeDiscriminant::CodeSmell,
            FactType::ComplexityViolation { .. } => FactTypeDiscriminant::ComplexityViolation,
            FactType::Dependency { .. } => FactTypeDiscriminant::Dependency,
            FactType::DependencyVulnerability { .. } => {
                FactTypeDiscriminant::DependencyVulnerability
            }
            FactType::License { .. } => FactTypeDiscriminant::License,
            FactType::UncoveredLine { .. } => FactTypeDiscriminant::UncoveredLine,
            FactType::LowTestCoverage { .. } => FactTypeDiscriminant::LowTestCoverage,
            FactType::CoverageStats { .. } => FactTypeDiscriminant::CoverageStats,
            FactType::Custom { .. } => FactTypeDiscriminant::Custom,
        }
    }

    /// Get a field value from a Custom fact type
    pub fn get_field(&self, key: &str) -> Option<&FactValue> {
        match self {
            FactType::Custom { data, .. } => data.get(key),
            _ => None,
        }
    }

    /// Get the discriminant string for Custom facts
    pub fn get_discriminant(&self) -> Option<&str> {
        match self {
            FactType::Custom { discriminant, .. } => Some(discriminant.as_str()),
            _ => None,
        }
    }
}

/// A single fact
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fact {
    /// Unique identifier for this fact
    pub id: FactId,
    /// The type of fact and its associated data
    pub fact_type: FactType,
    /// Human-readable message describing this fact
    pub message: String,
    /// Source location where this fact was found
    pub location: SourceLocation,
    /// Provenance information (extractor, version, confidence)
    pub provenance: Provenance,
}

impl Fact {
    /// Create a new fact with an auto-generated ID
    ///
    /// # Arguments
    ///
    /// * `fact_type` - The type of fact
    /// * `location` - The source location
    /// * `provenance` - The provenance information
    pub fn new(fact_type: FactType, location: SourceLocation, provenance: Provenance) -> Self {
        let message = match &fact_type {
            FactType::TaintSource { var, .. } => format!("Taint source: {}", var.0),
            FactType::TaintSink { func, .. } => format!("Taint sink in function: {}", func.0),
            FactType::Function { name, .. } => format!("Function: {}", name.0),
            FactType::Variable { name, .. } => format!("Variable: {}", name.0),
            FactType::Vulnerability { description, .. } => description.clone(),
            FactType::CodeSmell { smell_type, .. } => format!("Code smell: {}", smell_type),
            FactType::Custom { discriminant, .. } => format!("Custom fact: {}", discriminant),
            _ => format!("{:?}", fact_type),
        };

        Self {
            id: FactId::new(),
            fact_type,
            message,
            location,
            provenance,
        }
    }

    /// Create a new fact with a custom message
    ///
    /// # Arguments
    ///
    /// * `fact_type` - The type of fact
    /// * `message` - Human-readable message
    /// * `location` - The source location
    /// * `provenance` - The provenance information
    pub fn new_with_message(
        fact_type: FactType,
        message: String,
        location: SourceLocation,
        provenance: Provenance,
    ) -> Self {
        Self {
            id: FactId::new(),
            fact_type,
            message,
            location,
            provenance,
        }
    }
}

/// Intermediate representation container
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IntermediateRepresentation {
    /// List of all facts in the analysis
    pub facts: Vec<Fact>,
    /// Project metadata
    pub metadata: ProjectMetadata,
    /// Schema version for compatibility
    pub schema_version: String,
}

impl IntermediateRepresentation {
    /// Create a new intermediate representation
    ///
    /// # Arguments
    ///
    /// * `metadata` - Project metadata
    pub fn new(metadata: ProjectMetadata) -> Self {
        Self {
            facts: Vec::new(),
            metadata,
            schema_version: "3.3.0".to_string(),
        }
    }

    /// Create a new intermediate representation with facts
    ///
    /// # Arguments
    ///
    /// * `metadata` - Project metadata
    /// * `facts` - Initial facts
    pub fn with_facts(mut self, facts: Vec<Fact>) -> Self {
        self.facts = facts;
        self
    }

    /// Add a fact to the representation
    ///
    /// # Arguments
    ///
    /// * `fact` - Fact to add
    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.push(fact);
    }

    /// Get the total number of facts
    ///
    /// # Returns
    ///
    /// Number of facts in the representation
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

/// Project metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectMetadata {
    /// Name of the project
    pub name: String,
    /// Version of the project
    pub version: String,
    /// Root path of the project
    pub root_path: ProjectPath,
}

impl ProjectMetadata {
    /// Create new project metadata
    ///
    /// # Arguments
    ///
    /// * `name` - Project name
    /// * `version` - Project version
    /// * `root_path` - Project root path
    pub fn new(name: String, version: String, root_path: ProjectPath) -> Self {
        Self {
            name,
            version,
            root_path,
        }
    }
}
