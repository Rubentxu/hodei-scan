use crate::domain::error::DomainResult;
/// Repository ports - Interfaces for data persistence
///
/// These traits define the data access needs of the domain layer.
/// Each repository is responsible for a specific aggregate root.
use crate::domain::models::*;

/// Repository for baseline status operations
#[async_trait::async_trait]
pub trait BaselineRepository: Send + Sync {
    /// Save or update a baseline status
    async fn save_baseline_status(&self, update: BaselineStatusUpdate) -> DomainResult<()>;

    /// Get baseline status for a project/branch
    async fn get_baseline_statuses(
        &self,
        project_id: &ProjectId,
        branch: &str,
    ) -> DomainResult<Vec<BaselineStatus>>;

    /// Check if a finding is in baseline
    async fn is_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
    ) -> DomainResult<bool>;

    /// Mark finding as baseline
    async fn mark_as_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
        status: BaselineStatus,
        reason: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<UserId>,
    ) -> DomainResult<()>;

    /// Remove finding from baseline
    async fn remove_from_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
    ) -> DomainResult<()>;
}

/// Repository for analysis operations
#[async_trait::async_trait]
pub trait AnalysisRepository: Send + Sync {
    /// Save a new analysis
    async fn save_analysis(
        &self,
        analysis: StoredAnalysis,
        findings: &[Finding],
    ) -> DomainResult<AnalysisId>;

    /// Get analysis by ID
    async fn get_analysis(&self, id: &AnalysisId) -> DomainResult<Option<StoredAnalysis>>;

    /// Get all analyses for a project
    async fn get_project_analyses(
        &self,
        project_id: &ProjectId,
        limit: u32,
    ) -> DomainResult<Vec<StoredAnalysis>>;

    /// Get findings for an analysis
    async fn get_analysis_findings(&self, analysis_id: &AnalysisId) -> DomainResult<Vec<Finding>>;

    /// Get latest analysis for a project/branch
    async fn get_latest_analysis(
        &self,
        project_id: &ProjectId,
        branch: &str,
    ) -> DomainResult<Option<StoredAnalysis>>;
}
