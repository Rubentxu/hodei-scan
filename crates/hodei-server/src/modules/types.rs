/// Core data types for hodei-server
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Project identifier
pub type ProjectId = String;

/// Analysis identifier
pub type AnalysisId = Uuid;

/// Finding identifier
pub type FindingId = Uuid;

/// User identifier
pub type UserId = Uuid;

/// Analysis publication request from hodei-scan CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub project_id: ProjectId,
    pub branch: String,
    pub commit: String,
    pub findings: Vec<Finding>,
    pub metadata: AnalysisMetadata,
}

/// Analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub build_url: Option<String>,
    pub author: Option<String>,
    pub ci_run_id: Option<String>,
    pub scan_duration_ms: Option<u64>,
    pub rule_version: Option<String>,
}

/// Finding structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub fact_type: String,
    pub severity: Severity,
    pub location: FindingLocation,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub fingerprint: String,
}

/// Finding location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// Finding severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Info,
}

impl Severity {
    /// Convert severity to numeric value for comparison
    pub fn to_level(&self) -> u8 {
        match self {
            Severity::Critical => 4,
            Severity::Major => 3,
            Severity::Minor => 2,
            Severity::Info => 1,
        }
    }
}

/// Analysis publication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub analysis_id: AnalysisId,
    pub new_findings: u32,
    pub resolved_findings: u32,
    pub total_findings: u32,
    pub trend: TrendDirection,
    pub summary_url: String,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
}

/// Stored analysis in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAnalysis {
    pub id: AnalysisId,
    pub project_id: ProjectId,
    pub branch: String,
    pub commit: String,
    pub timestamp: DateTime<Utc>,
    pub findings_count: u32,
    pub metadata: AnalysisMetadata,
    pub created_at: DateTime<Utc>,
}

/// Analysis diff between two analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDiff {
    pub base_analysis: Option<StoredAnalysis>,
    pub head_analysis: Option<StoredAnalysis>,
    pub new_findings: Vec<Finding>,
    pub resolved_findings: Vec<Finding>,
    pub severity_increased: Vec<Finding>,
    pub severity_decreased: Vec<Finding>,
    pub wont_fix_changed: Vec<Finding>,
}

/// Trend metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetrics {
    pub period: TimePeriod,
    pub total_findings: u64,
    pub critical_findings: u64,
    pub major_findings: u64,
    pub minor_findings: u64,
    pub info_findings: u64,
    pub trend_percentage: f64,
    pub by_severity: HashMap<String, u64>,
    pub by_fact_type: HashMap<String, u64>,
}

/// Time period for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: UserId,
    pub expires_at: DateTime<Utc>,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Developer,
    Viewer,
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Baseline status for a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStatus {
    pub finding_id: FindingId,
    pub status: FindingStatus,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
}

/// Finding status in baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingStatus {
    Active,
    Accepted,
    WontFix,
    FalsePositive,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthCheckStatus,
    pub version: String,
    pub database: HealthCheckStatus,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub code: String,
    pub timestamp: DateTime<Utc>,
}
