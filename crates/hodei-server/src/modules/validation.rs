/// Data validation module for API requests
use crate::modules::error::{Result, ServerError};
use crate::modules::types::{Finding, PublishRequest};

/// Validation rules for publish analysis request
#[derive(Clone)]
pub struct ValidationConfig {
    pub max_findings_per_request: usize,
    pub max_metadata_size: usize,
    pub required_fields: Vec<String>,
    pub valid_severities: Vec<String>,
    pub max_project_id_length: usize,
    pub max_branch_length: usize,
    pub max_commit_length: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_findings_per_request: 100_000,
            max_metadata_size: 1024 * 1024, // 1MB
            required_fields: vec![
                "project_id".to_string(),
                "branch".to_string(),
                "commit".to_string(),
            ],
            valid_severities: vec![
                "critical".to_string(),
                "major".to_string(),
                "minor".to_string(),
                "info".to_string(),
            ],
            max_project_id_length: 100,
            max_branch_length: 100,
            max_commit_length: 100,
        }
    }
}

/// Validate publish analysis request
pub fn validate_publish_request(
    project_id: &str,
    request: &PublishRequest,
    config: &ValidationConfig,
) -> Result<()> {
    // Validate project_id
    if project_id.trim().is_empty() {
        return Err(ServerError::Validation(
            "Project ID cannot be empty".to_string(),
        ));
    }

    if project_id.len() > config.max_project_id_length {
        return Err(ServerError::Validation(format!(
            "Project ID exceeds maximum length of {}",
            config.max_project_id_length
        )));
    }

    // Validate branch
    if request.branch.trim().is_empty() {
        return Err(ServerError::Validation(
            "Branch cannot be empty".to_string(),
        ));
    }

    if request.branch.len() > config.max_branch_length {
        return Err(ServerError::Validation(format!(
            "Branch exceeds maximum length of {}",
            config.max_branch_length
        )));
    }

    // Validate commit
    if request.commit.trim().is_empty() {
        return Err(ServerError::Validation(
            "Commit hash cannot be empty".to_string(),
        ));
    }

    if request.commit.len() > config.max_commit_length {
        return Err(ServerError::Validation(format!(
            "Commit hash exceeds maximum length of {}",
            config.max_commit_length
        )));
    }

    // Validate findings
    if request.findings.len() > config.max_findings_per_request {
        return Err(ServerError::Validation(format!(
            "Number of findings ({}) exceeds maximum allowed ({})",
            request.findings.len(),
            config.max_findings_per_request
        )));
    }

    // Validate each finding
    for finding in &request.findings {
        validate_finding(finding, config)?;
    }

    // Validate metadata size
    let metadata_str = serde_json::to_string(&request.metadata)
        .map_err(|e| ServerError::Validation(format!("Invalid metadata: {}", e)))?;

    if metadata_str.len() > config.max_metadata_size {
        return Err(ServerError::Validation(format!(
            "Metadata exceeds maximum size of {} bytes",
            config.max_metadata_size
        )));
    }

    Ok(())
}

/// Validate individual finding
fn validate_finding(finding: &Finding, config: &ValidationConfig) -> Result<()> {
    // Validate fact_type
    if finding.fact_type.trim().is_empty() {
        return Err(ServerError::Validation(
            "Finding fact_type cannot be empty".to_string(),
        ));
    }

    // Validate severity
    let severity_str = finding.severity.to_string().to_lowercase();
    if !config.valid_severities.contains(&severity_str) {
        return Err(ServerError::Validation(format!(
            "Invalid severity level: {}",
            severity_str
        )));
    }

    // Validate location
    if finding.location.file.trim().is_empty() {
        return Err(ServerError::Validation(
            "Finding file path cannot be empty".to_string(),
        ));
    }

    if finding.location.line == 0 {
        return Err(ServerError::Validation(
            "Finding line number must be greater than 0".to_string(),
        ));
    }

    if finding.location.column == 0 {
        return Err(ServerError::Validation(
            "Finding column number must be greater than 0".to_string(),
        ));
    }

    // Validate message
    if finding.message.trim().is_empty() {
        return Err(ServerError::Validation(
            "Finding message cannot be empty".to_string(),
        ));
    }

    // Validate fingerprint
    if finding.fingerprint.trim().is_empty() {
        return Err(ServerError::Validation(
            "Finding fingerprint cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Project validation
pub async fn validate_project_exists(
    _project_id: &str,
    _database: &crate::modules::database::DatabaseConnection,
) -> Result<bool> {
    // TODO: Implement actual database check
    // For now, allow any project_id
    Ok(true)
}

/// Summary calculation from findings
pub fn calculate_summary(findings: &[Finding]) -> crate::modules::types::PublishResponse {
    use crate::modules::types::{PublishResponse, Severity, TrendDirection};

    let total_findings = findings.len() as u32;
    let critical_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical))
        .count() as u32;
    let _major_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Major))
        .count() as u32;
    let _minor_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Minor))
        .count() as u32;
    let _info_count = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Info))
        .count() as u32;

    // Simple trend calculation (TODO: implement real trend analysis)
    let trend = if critical_count > 0 {
        TrendDirection::Degrading
    } else if total_findings == 0 {
        TrendDirection::Improving
    } else {
        TrendDirection::Stable
    };

    PublishResponse {
        analysis_id: uuid::Uuid::new_v4(),
        new_findings: total_findings,
        resolved_findings: 0, // TODO: Calculate vs baseline
        total_findings,
        trend,
        summary_url: "/api/v1/analyses".to_string(), // TODO: Include actual ID
    }
}
