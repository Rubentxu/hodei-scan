/// Domain model for baseline management
/// This is pure business logic, no infrastructure dependencies
use crate::domain::models::{ProjectId, UserId};
use serde::{Deserialize, Serialize};

/// Baseline status for a finding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaselineStatus {
    Accepted,
    FalsePositive,
    WontFix,
}

/// Update request for baseline status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaselineStatusUpdate {
    pub project_id: ProjectId,
    pub branch: String,
    pub finding_fingerprint: String,
    pub status: BaselineStatus,
    pub reason: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub user_id: Option<UserId>,
}
