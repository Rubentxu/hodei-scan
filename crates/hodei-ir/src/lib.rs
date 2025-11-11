//! hodei-ir: Intermediate Representation core types

#![warn(missing_docs)]

pub mod fact_type_index;
pub mod interning;
pub mod types;
pub mod validator;
pub mod zero_copy;

pub use fact_type_index::*;
pub use types::*;
pub use validator::*;
pub use zero_copy::*;

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
        /// Descriptive message
        message: String,
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
}

/// A single fact
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fact {
    /// Unique identifier for this fact
    pub id: FactId,
    /// The type of fact and its associated data
    pub fact_type: FactType,
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
        Self {
            id: FactId::new(),
            fact_type,
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
            schema_version: "3.2.0".to_string(),
        }
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
