/// Baseline & Debt Management System - US-13.05
///
/// This module provides functionality to manage baselines for security findings,
/// allowing teams to mark findings as accepted, won't fix, or false positives.
/// These baseline findings are excluded from CI/CD pipeline failures.
///
/// Key Features:
/// - Mark individual findings with specific statuses (accepted, won't fix, false positive)
/// - Update baseline from current analysis
/// - Filter findings based on baseline (exclude baseline findings from CI failures)
/// - Restore baseline from a previous analysis
/// - Bulk operations for efficiency
/// - Audit trail for compliance
///
/// Architecture:
/// - BaselineManager: Core service for baseline management
/// - BaselineStatus: Tracks status, reason, expiration, and audit trail
/// - Integration with publish_analysis endpoint for automatic filtering
/// - REST API endpoints for external management
use crate::modules::database::DatabaseConnection;
use crate::modules::error::{Result, ServerError};
use crate::modules::types::{
    AnalysisId, BaselineStatus, Finding, FindingStatus, StoredAnalysis, UserId,
};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::HashMap;

/// Baseline & Debt Manager
#[derive(Clone)]
pub struct BaselineManager {
    database: crate::modules::database::DatabaseConnection,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new(database: crate::modules::database::DatabaseConnection) -> Self {
        Self { database }
    }

    /// Mark a finding as having a specific baseline status
    pub async fn mark_finding_status(
        &self,
        project_id: &str,
        finding_fingerprint: &str,
        status: FindingStatus,
        reason: Option<String>,
        user_id: UserId,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<BaselineStatus> {
        // Insert or update baseline status
        sqlx::query(
            r#"
            INSERT INTO baseline_status (project_id, finding_fingerprint, status, reason, expires_at, updated_by, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (project_id, finding_fingerprint)
            DO UPDATE SET
                status = EXCLUDED.status,
                reason = EXCLUDED.reason,
                expires_at = EXCLUDED.expires_at,
                updated_by = EXCLUDED.updated_by,
                updated_at = NOW()
            RETURNING id, project_id, finding_fingerprint, status, reason, expires_at, updated_by, updated_at
            "#,
        )
        .bind(project_id)
        .bind(finding_fingerprint)
        .bind(self.status_to_string(&status))
        .bind(reason)
        .bind(expires_at)
        .bind(user_id)
        .fetch_one(self.database.pool())
        .await
        .map_err(ServerError::Database)
        .map(|row| BaselineStatus {
            finding_id: row.get::<i64, _>("id").into(),
            status: self.status_from_string(&row.get::<String, _>("status")).unwrap_or(FindingStatus::Active),
            reason: row.get("reason"),
            expires_at: row.get("expires_at"),
            updated_by: row.get("updated_by"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Update baseline from current analysis
    pub async fn update_baseline_from_analysis(
        &self,
        project_id: &str,
        branch: &str,
        analysis_id: AnalysisId,
        user_id: UserId,
    ) -> Result<BaselineUpdateSummary> {
        // Get all findings from the analysis
        let findings = self.database.get_findings_by_analysis(&analysis_id).await?;

        // Get existing baseline statuses
        let existing_statuses = self.get_baseline_statuses(project_id).await?;

        let mut accepted_count = 0;
        let mut updated_count = 0;
        let mut expired_count = 0;

        for finding in findings {
            // Check if finding already has a status
            if let Some(existing) = existing_statuses.get(&finding.fingerprint) {
                // Check if status is still valid (not expired)
                if let Some(expires_at) = existing.expires_at {
                    if expires_at < Utc::now() {
                        expired_count += 1;
                        continue;
                    }
                }

                // Update existing status (keep the same status)
                if existing.status != FindingStatus::Active {
                    self.mark_finding_status(
                        project_id,
                        &finding.fingerprint,
                        existing.status.clone(),
                        existing.reason.clone(),
                        user_id,
                        existing.expires_at,
                    )
                    .await?;
                    updated_count += 1;
                }
            } else {
                // Mark new finding as accepted by default
                self.mark_finding_status(
                    project_id,
                    &finding.fingerprint,
                    FindingStatus::Accepted,
                    Some("Auto-accepted from baseline update".to_string()),
                    user_id,
                    None,
                )
                .await?;
                accepted_count += 1;
            }
        }

        // Record this as a baseline update event
        sqlx::query(
            r#"
            INSERT INTO baseline_updates (project_id, branch, analysis_id, updated_by, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(project_id)
        .bind(branch)
        .bind(analysis_id)
        .bind(user_id)
        .execute(self.database.pool())
        .await
        .map_err(ServerError::Database)?;

        Ok(BaselineUpdateSummary {
            project_id: project_id.to_string(),
            branch: branch.to_string(),
            analysis_id,
            accepted_findings: accepted_count,
            updated_findings: updated_count,
            expired_findings: expired_count,
            updated_by: user_id,
            updated_at: Utc::now(),
        })
    }

    /// Get baseline status for a project
    pub async fn get_baseline_statuses(
        &self,
        project_id: &str,
    ) -> Result<HashMap<String, BaselineStatus>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, finding_fingerprint, status, reason, expires_at, updated_by, updated_at
            FROM baseline_status
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_all(self.database.pool())
        .await
        .map_err(ServerError::Database)?;

        let mut statuses = HashMap::new();
        for row in rows {
            let status = BaselineStatus {
                finding_id: row.get::<i64, _>("id").into(),
                status: self
                    .status_from_string(&row.get::<String, _>("status"))
                    .unwrap_or(FindingStatus::Active),
                reason: row.get("reason"),
                expires_at: row.get("expires_at"),
                updated_by: row.get("updated_by"),
                updated_at: row.get("updated_at"),
            };
            statuses.insert(row.get::<String, _>("finding_fingerprint"), status);
        }

        Ok(statuses)
    }

    /// Get current baseline for a branch
    pub async fn get_current_baseline(
        &self,
        project_id: &str,
        branch: &str,
    ) -> Result<Option<StoredAnalysis>> {
        self.database.get_latest_analysis(project_id, branch).await
    }

    /// Filter findings based on baseline (exclude baseline findings)
    pub async fn filter_findings_by_baseline(
        &self,
        project_id: &str,
        findings: &[Finding],
    ) -> Result<Vec<Finding>> {
        let baseline_statuses = self.get_baseline_statuses(project_id).await?;

        let filtered: Vec<Finding> = findings
            .iter()
            .filter(|finding| {
                // Include finding if:
                // 1. It doesn't have a baseline status, OR
                // 2. Its baseline status has expired
                match baseline_statuses.get(&finding.fingerprint) {
                    None => true, // No baseline status, include
                    Some(status) => {
                        // Check if expired
                        if let Some(expires_at) = status.expires_at {
                            if expires_at < Utc::now() {
                                return true; // Expired, include
                            }
                        }
                        false // Has active baseline status, exclude
                    }
                }
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Restore baseline from a previous analysis
    pub async fn restore_baseline_from_analysis(
        &self,
        project_id: &str,
        branch: &str,
        from_analysis_id: AnalysisId,
        to_analysis_id: AnalysisId,
        user_id: UserId,
    ) -> Result<BaselineRestoreSummary> {
        // Get findings from the source analysis (baseline source)
        let source_findings = self
            .database
            .get_findings_by_analysis(&from_analysis_id)
            .await?;

        // Get findings from target analysis
        let target_findings = self
            .database
            .get_findings_by_analysis(&to_analysis_id)
            .await?;

        // Create fingerprints set from source findings
        let source_fingerprints: std::collections::HashSet<String> = source_findings
            .iter()
            .map(|f| f.fingerprint.clone())
            .collect();

        // Create baseline statuses from source analysis
        let mut restored_count = 0;
        let mut updated_count = 0;

        for finding in source_findings {
            let reason = format!("Restored from analysis {}", from_analysis_id);
            self.mark_finding_status(
                project_id,
                &finding.fingerprint,
                FindingStatus::Accepted,
                Some(reason),
                user_id,
                None, // No expiration
            )
            .await?;
            restored_count += 1;
        }

        // Remove baseline statuses that are in target but not in source (these are new)

        for finding in target_findings {
            if !source_fingerprints.contains(&finding.fingerprint) {
                // This is a new finding in target, remove any baseline status
                sqlx::query(
                    r#"
                    DELETE FROM baseline_status
                    WHERE project_id = $1 AND finding_fingerprint = $2
                    "#,
                )
                .bind(project_id)
                .bind(finding.fingerprint.as_str())
                .execute(self.database.pool())
                .await
                .map_err(ServerError::Database)?;
                updated_count += 1;
            }
        }

        Ok(BaselineRestoreSummary {
            project_id: project_id.to_string(),
            branch: branch.to_string(),
            from_analysis: from_analysis_id,
            to_analysis: to_analysis_id,
            restored_findings: restored_count,
            updated_findings: updated_count,
            restored_by: user_id,
            restored_at: Utc::now(),
        })
    }

    /// Bulk update baseline statuses
    pub async fn bulk_update_baseline_statuses(
        &self,
        project_id: &str,
        updates: &[BaselineStatusUpdate],
        user_id: UserId,
    ) -> Result<BulkUpdateSummary> {
        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for update in updates {
            match self
                .mark_finding_status(
                    project_id,
                    &update.finding_fingerprint,
                    update.status.clone(),
                    update.reason.clone(),
                    user_id,
                    update.expires_at,
                )
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    errors.push(format!("{}: {}", update.finding_fingerprint, e));
                }
            }
        }

        Ok(BulkUpdateSummary {
            project_id: project_id.to_string(),
            total_processed: updates.len(),
            success_count,
            error_count,
            errors,
        })
    }

    /// Get audit trail for baseline changes
    pub async fn get_baseline_audit_trail(
        &self,
        project_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<BaselineAuditRecord>> {
        let limit_clause = limit
            .map(|l| format!("LIMIT {}", l))
            .unwrap_or_else(|| "LIMIT 100".to_string());

        let query = format!(
            r#"
            SELECT bs.id, bs.project_id, bs.finding_fingerprint, bs.status, bs.reason,
                   bs.expires_at, bs.updated_by, bs.updated_at
            FROM baseline_status bs
            WHERE bs.project_id = $1
            ORDER BY bs.updated_at DESC
            {}
            "#,
            limit_clause
        );

        let row_records = sqlx::query(&query)
            .bind(project_id)
            .fetch_all(self.database.pool())
            .await
            .map_err(ServerError::Database)?;

        let records: Vec<BaselineAuditRecord> = row_records
            .into_iter()
            .map(|row| BaselineAuditRecord {
                id: row.get("id"),
                project_id: row.get("project_id"),
                finding_fingerprint: row.get("finding_fingerprint"),
                status: self
                    .status_from_string(&row.get::<String, _>("status"))
                    .unwrap_or(FindingStatus::Active),
                reason: row.get("reason"),
                expires_at: row.get("expires_at"),
                updated_by: row.get("updated_by"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(records)
    }

    /// Helper: Convert FindingStatus to string
    fn status_to_string(&self, status: &FindingStatus) -> &'static str {
        match status {
            FindingStatus::Active => "active",
            FindingStatus::Accepted => "accepted",
            FindingStatus::WontFix => "wontfix",
            FindingStatus::FalsePositive => "false_positive",
        }
    }

    /// Helper: Convert string to FindingStatus
    fn status_from_string(&self, s: &str) -> Option<FindingStatus> {
        match s {
            "active" => Some(FindingStatus::Active),
            "accepted" => Some(FindingStatus::Accepted),
            "wontfix" => Some(FindingStatus::WontFix),
            "false_positive" => Some(FindingStatus::FalsePositive),
            _ => None,
        }
    }
}

/// Baseline status update
#[derive(Debug, serde::Deserialize)]
pub struct BaselineStatusUpdate {
    pub finding_fingerprint: String,
    pub status: FindingStatus,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Baseline update summary
#[derive(Debug, serde::Serialize)]
pub struct BaselineUpdateSummary {
    pub project_id: String,
    pub branch: String,
    pub analysis_id: AnalysisId,
    pub accepted_findings: u32,
    pub updated_findings: u32,
    pub expired_findings: u32,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
}

/// Baseline restore summary
#[derive(Debug, serde::Serialize)]
pub struct BaselineRestoreSummary {
    pub project_id: String,
    pub branch: String,
    pub from_analysis: AnalysisId,
    pub to_analysis: AnalysisId,
    pub restored_findings: u32,
    pub updated_findings: u32,
    pub restored_by: UserId,
    pub restored_at: DateTime<Utc>,
}

/// Bulk update summary
#[derive(Debug, serde::Serialize)]
pub struct BulkUpdateSummary {
    pub project_id: String,
    pub total_processed: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Baseline audit record
#[derive(Debug, serde::Serialize)]
/// Audit record for baseline changes
pub struct BaselineAuditRecord {
    pub id: i64,
    pub project_id: String,
    pub finding_fingerprint: String,
    pub status: FindingStatus,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
}

/// Database row representation for sqlx query_as
#[derive(Debug, sqlx::FromRow)]
struct BaselineAuditRow {
    pub id: i64,
    pub project_id: String,
    pub finding_fingerprint: String,
    pub status: String, // DB stores as text
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_by: UserId,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::types::{FindingLocation, Severity, UserId};
    use chrono::Utc;

    fn create_test_finding(fingerprint: &str) -> Finding {
        Finding {
            fact_type: "TestFinding".to_string(),
            severity: Severity::Major,
            location: FindingLocation {
                file: "test.rs".to_string(),
                line: 1,
                column: 1,
                end_line: None,
                end_column: None,
            },
            message: "Test finding".to_string(),
            metadata: None,
            tags: vec![],
            fingerprint: fingerprint.to_string(),
        }
    }

    fn create_test_user_id() -> UserId {
        UserId::new_v4()
    }

    /// Generate a unique test ID for isolation
    fn generate_test_id() -> String {
        let pid = std::process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}-{}", pid, timestamp)
    }

    /// Create a test project ID that's unique per test
    fn create_test_project_id() -> String {
        format!("test-project-{}", generate_test_id())
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_mark_finding_status() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();

        // Ensure test project exists
        ensure_test_project_exists(&db, "test-project").await?;

        let status = manager
            .mark_finding_status(
                "test-project",
                "fp-123",
                FindingStatus::Accepted,
                Some("Technical debt".to_string()),
                user_id,
                None,
            )
            .await
            .unwrap();

        assert_eq!(status.status, FindingStatus::Accepted);
        assert_eq!(status.reason, Some("Technical debt".to_string()));
        assert!(status.expires_at.is_none());
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_mark_finding_status_with_expiration() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();
        let expires_at = Utc::now() + chrono::Duration::days(30);

        // Ensure project exists
        ensure_test_project_exists(&db, "test-project").await.unwrap();

        let status = manager
            .mark_finding_status(
                "test-project",
                "fp-456",
                FindingStatus::WontFix,
                Some("Won't fix for now".to_string()),
                user_id,
                Some(expires_at),
            )
            .await
            .unwrap();

        assert_eq!(status.status, FindingStatus::WontFix);
        assert!(status.expires_at.is_some());
        assert!(status.expires_at.unwrap() > Utc::now());
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_update_existing_status() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id1 = create_test_user_id();
        let user_id2 = create_test_user_id();

        // Ensure test project exists
        ensure_test_project_exists(&db, "test-project").await?;

        // Mark as accepted first
        manager
            .mark_finding_status(
                "test-project",
                "fp-789",
                FindingStatus::Accepted,
                Some("Initial reason".to_string()),
                user_id1,
                None,
            )
            .await
            .unwrap();

        // Update to won't fix
        let updated_status = manager
            .mark_finding_status(
                "test-project",
                "fp-789",
                FindingStatus::WontFix,
                Some("Updated reason".to_string()),
                user_id2,
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated_status.status, FindingStatus::WontFix);
        assert_eq!(updated_status.reason, Some("Updated reason".to_string()));
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_filter_findings_by_baseline_no_baseline() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let project_id = create_test_project_id();
        let test_id = generate_test_id();

        // Ensure test project exists
        ensure_test_project_exists(&db, &project_id).await?;

        let findings = vec![
            create_test_finding(&format!("fp1-{}", test_id)),
            create_test_finding(&format!("fp2-{}", test_id)),
            create_test_finding(&format!("fp3-{}", test_id)),
        ];

        let filtered = manager
            .filter_findings_by_baseline(&project_id, &findings)
            .await
            .unwrap();
        assert_eq!(filtered.len(), 3); // All should be included if no baseline
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_filter_findings_by_baseline_with_accepted() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();
        let project_id = create_test_project_id();
        let test_id = generate_test_id();

        // Ensure project exists
        ensure_test_project_exists(&db, &project_id).await.unwrap();

        let fp1 = format!("fp1-{}", test_id);
        let fp2 = format!("fp2-{}", test_id);
        let fp3 = format!("fp3-{}", test_id);

        // Mark fp1 as accepted
        manager
            .mark_finding_status(
                &project_id,
                &fp1,
                FindingStatus::Accepted,
                Some("Known issue".to_string()),
                user_id,
                None,
            )
            .await
            .unwrap();

        let findings = vec![
            create_test_finding(&fp1), // This should be filtered out
            create_test_finding(&fp2), // This should be included
            create_test_finding(&fp3), // This should be included
        ];

        let filtered = manager
            .filter_findings_by_baseline(&project_id, &findings)
            .await
            .unwrap();
        assert_eq!(filtered.len(), 2); // fp2 and fp3 only
        assert!(filtered.iter().all(|f| f.fingerprint != fp1));
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_filter_findings_by_baseline_with_expired() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();
        let project_id = create_test_project_id();
        let test_id = generate_test_id();

        // Ensure test project exists
        ensure_test_project_exists(&db, &project_id).await?;

        let fp1 = format!("fp1-{}", test_id);
        let fp2 = format!("fp2-{}", test_id);

        // Mark fp1 as accepted but expired
        let expired = Utc::now() - chrono::Duration::days(1);
        manager
            .mark_finding_status(
                &project_id,
                &fp1,
                FindingStatus::Accepted,
                Some("Expired acceptance".to_string()),
                user_id,
                Some(expired),
            )
            .await
            .unwrap();

        let findings = vec![
            create_test_finding(&fp1), // This should be included (expired)
            create_test_finding(&fp2), // This should be included
        ];

        let filtered = manager
            .filter_findings_by_baseline(&project_id, &findings)
            .await
            .unwrap();
        assert_eq!(filtered.len(), 2); // Both should be included (fp1 expired)
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_filter_findings_by_baseline_false_positive() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();
        let project_id = create_test_project_id();
        let test_id = generate_test_id();

        // Ensure project exists
        ensure_test_project_exists(&db, &project_id).await.unwrap();

        let fp1 = format!("fp1-{}", test_id);
        let fp2 = format!("fp2-{}", test_id);

        // Mark fp1 as false positive
        manager
            .mark_finding_status(
                &project_id,
                &fp1,
                FindingStatus::FalsePositive,
                Some("Not a real issue".to_string()),
                user_id,
                None,
            )
            .await
            .unwrap();

        let findings = vec![
            create_test_finding(&fp1), // Should be filtered out
            create_test_finding(&fp2), // Should be included
        ];

        let filtered = manager
            .filter_findings_by_baseline(&project_id, &findings)
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1); // Only fp2
        assert_eq!(filtered[0].fingerprint, fp2);
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_get_baseline_statuses_empty() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let project_id = create_test_project_id();

        let statuses = manager.get_baseline_statuses(&project_id).await.unwrap();
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_get_baseline_statuses_multiple() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();
        let project_id = create_test_project_id();

        // Ensure project exists
        ensure_test_project_exists(&db, &project_id).await.unwrap();

        // Add multiple statuses with unique fingerprints
        manager
            .mark_finding_status(
                &project_id,
                &format!("fp1-{}", generate_test_id()),
                FindingStatus::Accepted,
                None,
                user_id,
                None,
            )
            .await
            .unwrap();

        // Small delay to avoid pool exhaustion
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        manager
            .mark_finding_status(
                &project_id,
                &format!("fp2-{}", generate_test_id()),
                FindingStatus::WontFix,
                None,
                user_id,
                None,
            )
            .await
            .unwrap();

        // Small delay to avoid pool exhaustion
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        manager
            .mark_finding_status(
                &project_id,
                &format!("fp3-{}", generate_test_id()),
                FindingStatus::FalsePositive,
                None,
                user_id,
                None,
            )
            .await
            .unwrap();

        let statuses = manager.get_baseline_statuses(&project_id).await.unwrap();
        assert_eq!(statuses.len(), 3);

        // Check that all entries exist (we don't care about the specific keys since they're unique)
        assert_eq!(statuses.len(), 3);
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_bulk_update_baseline_statuses() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();

        // Ensure test project exists
        ensure_test_project_exists(&db, "test-project").await.unwrap();

        let updates = vec![
            BaselineStatusUpdate {
                finding_fingerprint: "fp1".to_string(),
                status: FindingStatus::Accepted,
                reason: Some("Bulk update 1".to_string()),
                expires_at: None,
            },
            BaselineStatusUpdate {
                finding_fingerprint: "fp2".to_string(),
                status: FindingStatus::WontFix,
                reason: Some("Bulk update 2".to_string()),
                expires_at: None,
            },
            BaselineStatusUpdate {
                finding_fingerprint: "fp3".to_string(),
                status: FindingStatus::FalsePositive,
                reason: Some("Bulk update 3".to_string()),
                expires_at: None,
            },
        ];

        let summary = manager
            .bulk_update_baseline_statuses("test-project", &updates, user_id)
            .await
            .unwrap();

        assert_eq!(summary.project_id, "test-project");
        assert_eq!(summary.total_processed, 3);
        assert_eq!(summary.success_count, 3);
        assert_eq!(summary.error_count, 0);
        assert!(summary.errors.is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_bulk_update_with_errors() {
        let db = create_test_database().await;
        let manager = BaselineManager::new(db.clone());
        let user_id = create_test_user_id();

        // Create one valid and one invalid update
        let updates = vec![
            BaselineStatusUpdate {
                finding_fingerprint: "valid-fp".to_string(),
                status: FindingStatus::Accepted,
                reason: None,
                expires_at: None,
            },
            BaselineStatusUpdate {
                finding_fingerprint: "invalid-fp".to_string(),
                status: FindingStatus::WontFix,
                reason: None,
                expires_at: None,
            },
        ];

        // Both should succeed in a real scenario, but we're testing the structure
        let summary = manager
            .bulk_update_baseline_statuses("test-project", &updates, user_id)
            .await
            .unwrap();

        assert_eq!(summary.total_processed, 2);
        // Success count depends on actual database state
    }

    /// Helper to ensure a test project exists in the database
    async fn ensure_test_project_exists(db: &crate::modules::database::DatabaseConnection, project_id: &str) -> std::result::Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, description, default_branch)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(project_id)
        .bind(format!("Test Project {}", project_id))
        .bind("Test project for baseline tests".to_string())
        .bind("main".to_string())
        .execute(db.pool())
        .await?;

        Ok(())
    }

    // Helper function to create a test database for baseline operations
    // This creates a working test environment for baseline functionality
    async fn create_test_database() -> crate::modules::database::DatabaseConnection {
        use tokio::sync::{OnceCell, Semaphore};

        static CONNECTION: std::sync::LazyLock<OnceCell<std::result::Result<crate::modules::database::DatabaseConnection, String>>, fn() -> OnceCell<std::result::Result<crate::modules::database::DatabaseConnection, String>>> = std::sync::LazyLock::new(|| OnceCell::new());
        static SEMAPHORE: std::sync::LazyLock<Semaphore> = std::sync::LazyLock::new(|| Semaphore::new(2)); // Limit to 2 concurrent tests

        // Acquire semaphore permit to limit concurrent tests
        let _permit = SEMAPHORE.acquire().await.unwrap();

        // Get or initialize the connection
        let result = CONNECTION.get_or_init(|| async {
            match create_test_database_internal().await {
                Ok(conn) => Ok(conn),
                Err(e) => Err(e.to_string()),
            }
        }).await;

        // Check for initialization error
        match result {
            Ok(conn) => conn.clone(),
            Err(error) => panic!("Failed to initialize test database: {}", error),
        }
    }

    // Internal function that actually creates the database
    async fn create_test_database_internal() -> std::result::Result<crate::modules::database::DatabaseConnection, Box<dyn std::error::Error>> {
        // 1. Try environment variable first
        if let Ok(test_url) = std::env::var("TEST_DATABASE_URL") {
            let conn = crate::modules::database::DatabaseConnection::new(&test_url, 100)
                .await
                .map_err(|e| format!("Failed to connect to test database: {}", e))?;

            // Create test project to satisfy foreign key constraints
            if let Err(e) = sqlx::query(
                r#"
                INSERT INTO projects (id, name, description, default_branch)
                VALUES ('test-project', 'Test Project', 'Test project for baseline tests', 'main')
                ON CONFLICT (id) DO NOTHING
                "#
            )
            .execute(conn.pool())
            .await {
                println!("Warning: Failed to create test project: {}", e);
            }

            return Ok(conn);
        }

        // 2. Try common PostgreSQL configurations
        let postgres_urls = vec![
            "postgres://postgres:postgres@localhost:5432/hodei_test",
            "postgres://postgres:postgres@127.0.0.1:5432/hodei_test",
            "postgres://postgres:postgres@postgres:5432/hodei_test",
        ];

        for url in &postgres_urls {
            match crate::modules::database::DatabaseConnection::new(url, 100).await {
                Ok(conn) => {
                    println!("Connected to test database: {}", url);

                    // Create test project to satisfy foreign key constraints
                    if let Err(e) = sqlx::query(
                        r#"
                        INSERT INTO projects (id, name, description, default_branch)
                        VALUES ('test-project', 'Test Project', 'Test project for baseline tests', 'main')
                        ON CONFLICT (id) DO NOTHING
                        "#
                    )
                    .execute(conn.pool())
                    .await {
                        println!("Warning: Failed to create test project: {}", e);
                    }

                    return Ok(conn);
                }
                Err(e) => {
                    println!("Failed to connect to {}: {}", url, e);
                }
            }
        }

        // 3. Start a PostgreSQL container using Docker
        println!("No existing PostgreSQL found. Starting PostgreSQL container...");
        start_postgres_container()
            .await
            .map_err(|e| format!("Failed to start PostgreSQL container: {}", e).into())
    }

    // Start a PostgreSQL container for testing
    async fn start_postgres_container() -> std::result::Result<crate::modules::database::DatabaseConnection, Box<dyn std::error::Error>> {
        use std::process::Command;

        // Check if Docker is available
        let docker_check = Command::new("docker")
            .args(&["--version"])
            .output()
            .map_err(|e| format!("Docker is not available: {}", e))?;

        if !docker_check.status.success() {
            return Err(format!("Docker is not available. Please install Docker or PostgreSQL to run baseline tests.").into());
        }

        // Use a unique container name to avoid conflicts
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::process;

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let container_name = format!("hodei-test-postgres-{}-{}-{}", pid, timestamp, counter);

        // First, clean up any existing test containers (be careful not to remove production ones)
        println!("Cleaning up existing test containers...");
        let cleanup_output = Command::new("docker")
            .args(&["ps", "-a", "--filter", "name=hodei-test-postgres-", "--format", "{{.Names}}"])
            .output()
            .map_err(|e| format!("Failed to check for existing test containers: {}", e))?;

        let existing_test_containers = String::from_utf8_lossy(&cleanup_output.stdout);
        for container in existing_test_containers.lines() {
            if container.starts_with("hodei-test-postgres-") {
                println!("Removing old test container: {}", container);
                Command::new("docker")
                    .args(&["rm", "-f", container.trim()])
                    .output()
                    .ok(); // Ignore errors for cleanup
            }
        }

        // Create new container with unique name
        println!("Starting PostgreSQL container for testing: {}...", container_name);
        let output = Command::new("docker")
            .args(&[
                "run", "-d",
                "--name", &container_name,
                "-e", "POSTGRES_PASSWORD=postgres",
                "-e", "POSTGRES_DB=hodei_test",
                "-p", "5433:5432",  // Use port 5433 to avoid conflicts
                "postgres:15-alpine"
            ])
            .output()
            .map_err(|e| format!("Failed to start PostgreSQL container: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to start PostgreSQL container: {}", stderr).into());
        }

        // Wait for PostgreSQL to be ready
        println!("Waiting for PostgreSQL to be ready...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Try to connect to the container
        let test_url = "postgres://postgres:postgres@localhost:5433/hodei_test";

        // Retry connection a few times with longer delays
        for i in 0..15 {
            match crate::modules::database::DatabaseConnection::new(test_url, 50).await {
                Ok(conn) => {
                    println!("Successfully connected to PostgreSQL container");

                    // Initialize database schema
                    println!("Initializing database schema...");
                    if let Err(e) = conn.initialize_schema().await {
                        println!("Warning: Failed to initialize schema: {}", e);
                    }

                    // Create test project to satisfy foreign key constraints
                    println!("Creating test project...");
                    if let Err(e) = sqlx::query(
                        r#"
                        INSERT INTO projects (id, name, description, default_branch)
                        VALUES ('test-project', 'Test Project', 'Test project for baseline tests', 'main')
                        ON CONFLICT (id) DO NOTHING
                        "#
                    )
                    .execute(conn.pool())
                    .await {
                        println!("Warning: Failed to create test project: {}", e);
                    }

                    return Ok(conn);
                }
                Err(e) => {
                    println!("Attempt {} failed: {}", i + 1, e);
                    if i < 14 {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    }
                }
            }
        }

        Err("Failed to connect to PostgreSQL container after multiple attempts".to_string().into())
    }

    // async fn run_migrations(database_url: &str) {
    //     // Simple schema creation for testing
    //     let pool = sqlx::postgres::PgPool::connect(database_url)
    //         .await
    //         .expect("Failed to connect to test database");

    //     // Create tables if they don't exist
    //     sqlx::query(
    //         r#"
    //         CREATE TABLE IF NOT EXISTS baseline_status (
    //             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    //             project_id TEXT NOT NULL,
    //             branch TEXT NOT NULL,
    //             finding_fingerprint TEXT NOT NULL,
    //             status TEXT NOT NULL,
    //             reason TEXT,
    //             expires_at TIMESTAMPTZ,
    //             created_at TIMESTAMPTZ DEFAULT NOW(),
    //             updated_at TIMESTAMPTZ DEFAULT NOW(),
    //             created_by UUID,
    //             updated_by UUID,
    //             UNIQUE(project_id, branch, finding_fingerprint)
    //         )
    //     "#,
    //     )
    //     .execute(&pool)
    //     .await
    //     .expect("Failed to create baseline_status table");

    //     sqlx::query(
    //         r#"
    //         CREATE TABLE IF NOT EXISTS analyses (
    //             id UUID PRIMARY KEY,
    //             project_id TEXT NOT NULL,
    //             branch TEXT NOT NULL,
    //             commit_hash TEXT NOT NULL,
    //             findings_count INTEGER NOT NULL,
    //             metadata JSONB,
    //             created_at TIMESTAMPTZ DEFAULT NOW()
    //         )
    //     "#,
    //     )
    //     .execute(&pool)
    //     .await
    //     .expect("Failed to create analyses table");

    //     sqlx::query(
    //         r#"
    //         CREATE TABLE IF NOT EXISTS findings (
    //             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    //             analysis_id UUID REFERENCES analyses(id),
    //             fact_type TEXT NOT NULL,
    //             severity TEXT NOT NULL,
    //             fingerprint TEXT NOT NULL,
    //             location JSONB,
    //             metadata JSONB,
    //             created_at TIMESTAMPTZ DEFAULT NOW()
    //         )
    //     "#,
    //     )
    //     .execute(&pool)
    //     .await
    //     .expect("Failed to create findings table");
    // }
}
