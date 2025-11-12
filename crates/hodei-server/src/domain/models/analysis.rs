use crate::domain::models::{AnalysisId, Finding, ProjectId};
/// Domain model for security analysis
/// This is pure business logic, no infrastructure dependencies
use serde::{Deserialize, Serialize};

/// Analysis metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub build_url: Option<String>,
    pub author: Option<String>,
    pub ci_run_id: Option<String>,
    pub scan_duration_ms: Option<u64>,
    pub rule_version: Option<String>,
}

/// Stored analysis entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAnalysis {
    pub id: AnalysisId,
    pub project_id: ProjectId,
    pub branch: String,
    pub commit_hash: String,
    pub findings_count: u32,
    pub metadata: Option<AnalysisMetadata>,
}

/// Publish request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishRequest {
    pub project_id: ProjectId,
    pub branch: String,
    pub commit_hash: String,
    pub findings: Vec<Finding>,
    pub metadata: Option<AnalysisMetadata>,
}

/// Publish response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishResponse {
    pub analysis_id: AnalysisId,
    pub total_findings: u32,
    pub critical_count: u32,
    pub major_count: u32,
    pub minor_count: u32,
    pub info_count: u32,
}
