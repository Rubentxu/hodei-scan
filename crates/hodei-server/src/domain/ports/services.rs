use crate::domain::error::DomainResult;
/// Service ports - Interfaces for external services
///
/// These traits define the domain's needs for external services
/// like authentication, notifications, etc.
use crate::domain::models::*;

/// Authentication service port
#[async_trait::async_trait]
pub trait AuthService: Send + Sync {
    /// Validate a JWT token
    async fn validate_token(&self, token: &str) -> DomainResult<UserId>;

    /// Generate a new JWT token
    async fn generate_token(&self, user_id: UserId) -> DomainResult<String>;

    /// Refresh an existing token
    async fn refresh_token(&self, token: &str) -> DomainResult<String>;
}

/// Notification service port
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    /// Send analysis published notification
    async fn notify_analysis_published(
        &self,
        project_id: &ProjectId,
        analysis_id: &AnalysisId,
        findings_count: u32,
    ) -> DomainResult<()>;

    /// Send baseline updated notification
    async fn notify_baseline_updated(
        &self,
        project_id: &ProjectId,
        branch: &str,
        updated_count: u32,
    ) -> DomainResult<()>;
}
