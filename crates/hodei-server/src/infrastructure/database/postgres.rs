use crate::domain::error::DomainResult;
/// PostgreSQL Database Adapter
/// This module is only compiled when the "postgresql" feature is enabled
use crate::domain::models::*;
use crate::domain::ports::repositories::*;
use async_trait::async_trait;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;

/// PostgreSQL-specific implementation of analysis repository
#[derive(Clone)]
pub struct PostgresAnalysisRepository {
    pool: PgPool,
}

impl PostgresAnalysisRepository {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analyses (
                id UUID PRIMARY KEY,
                project_id TEXT NOT NULL,
                branch TEXT NOT NULL,
                commit_hash TEXT NOT NULL,
                findings_count INTEGER NOT NULL,
                metadata JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS findings (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                analysis_id UUID REFERENCES analyses(id) ON DELETE CASCADE,
                fact_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                location JSONB,
                metadata JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS baseline_status (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                project_id TEXT NOT NULL,
                branch TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                status TEXT NOT NULL,
                reason TEXT,
                expires_at TIMESTAMPTZ,
                user_id UUID,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(project_id, branch, fingerprint)
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl AnalysisRepository for PostgresAnalysisRepository {
    async fn save_analysis(
        &self,
        analysis: StoredAnalysis,
        findings: &[Finding],
    ) -> DomainResult<AnalysisId> {
        // Convert analysis to JSON for storage
        let metadata_json = serde_json::to_value(&analysis.metadata).map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to serialize metadata: {}",
                e
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO analyses (id, project_id, branch, commit_hash, findings_count, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(analysis.id.0)
        .bind(analysis.project_id.as_str())
        .bind(analysis.branch)
        .bind(analysis.commit_hash)
        .bind(analysis.findings_count as i32)
        .bind(metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to save analysis: {}", e))
        })?;

        // Save findings
        for finding in findings {
            let location_json = serde_json::to_value(&finding.location).map_err(|e| {
                crate::domain::error::DomainError::internal(&format!(
                    "Failed to serialize location: {}",
                    e
                ))
            })?;
            let metadata_json =
                serde_json::to_value(&finding.metadata).unwrap_or_else(|_| serde_json::Value::Null);

            sqlx::query(r#"
                INSERT INTO findings (analysis_id, fact_type, severity, fingerprint, location, metadata)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#)
            .bind(analysis.id.0)
            .bind(&finding.fact_type)
            .bind(finding.severity.to_string())
            .bind(&finding.fingerprint)
            .bind(location_json)
            .bind(metadata_json)
            .execute(&self.pool)
            .await
            .map_err(|e| crate::domain::error::DomainError::internal(&format!("Failed to save finding: {}", e)))?;
        }

        Ok(analysis.id)
    }

    async fn get_analysis(&self, id: &AnalysisId) -> DomainResult<Option<StoredAnalysis>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, branch, commit_hash, findings_count, metadata
            FROM analyses
            WHERE id = $1
        "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to get analysis: {}", e))
        })?;

        if let Some(row) = row {
            let metadata_value: Option<serde_json::Value> = row.get("metadata");
            let metadata = metadata_value
                .as_ref()
                .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

            Ok(Some(StoredAnalysis {
                id: AnalysisId(row.get("id")),
                project_id: ProjectId(row.get("project_id")),
                branch: row.get("branch"),
                commit_hash: row.get("commit_hash"),
                findings_count: row.get::<i32, _>("findings_count") as u32,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_project_analyses(
        &self,
        project_id: &ProjectId,
        limit: u32,
    ) -> DomainResult<Vec<StoredAnalysis>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, branch, commit_hash, findings_count, metadata
            FROM analyses
            WHERE project_id = $1
            ORDER BY created_at DESC
            LIMIT $2
        "#,
        )
        .bind(project_id.as_str())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to get project analyses: {}",
                e
            ))
        })?;

        let analyses = rows
            .into_iter()
            .map(|row| {
                let metadata_value: Option<serde_json::Value> = row.get("metadata");
                let metadata = metadata_value
                    .as_ref()
                    .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

                StoredAnalysis {
                    id: AnalysisId(row.get("id")),
                    project_id: ProjectId(row.get("project_id")),
                    branch: row.get("branch"),
                    commit_hash: row.get("commit_hash"),
                    findings_count: row.get::<i32, _>("findings_count") as u32,
                    metadata,
                }
            })
            .collect();

        Ok(analyses)
    }

    async fn get_analysis_findings(&self, analysis_id: &AnalysisId) -> DomainResult<Vec<Finding>> {
        let rows = sqlx::query(
            r#"
            SELECT fact_type, severity, fingerprint, location, metadata
            FROM findings
            WHERE analysis_id = $1
        "#,
        )
        .bind(analysis_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to get findings: {}", e))
        })?;

        let findings = rows
            .into_iter()
            .map(|row| {
                let location_value: serde_json::Value = row.get("location");
                let metadata_value: Option<serde_json::Value> = row.get("metadata");
                let metadata = metadata_value.as_ref().and_then(|v| {
                    serde_json::from_value::<HashMap<String, serde_json::Value>>(v.clone()).ok()
                });

                let location = serde_json::from_value::<FindingLocation>(location_value)
                    .unwrap_or_else(|_| FindingLocation {
                        file: "unknown".to_string(),
                        line: None,
                        column: None,
                    });

                Finding {
                    fact_type: row.get("fact_type"),
                    severity: match row.get::<String, _>("severity").as_str() {
                        "critical" => Severity::Critical,
                        "major" => Severity::Major,
                        "minor" => Severity::Minor,
                        _ => Severity::Info,
                    },
                    fingerprint: row.get("fingerprint"),
                    location,
                    metadata,
                }
            })
            .collect();

        Ok(findings)
    }

    async fn get_latest_analysis(
        &self,
        project_id: &ProjectId,
        branch: &str,
    ) -> DomainResult<Option<StoredAnalysis>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, branch, commit_hash, findings_count, metadata
            FROM analyses
            WHERE project_id = $1 AND branch = $2
            ORDER BY created_at DESC
            LIMIT 1
        "#,
        )
        .bind(project_id.as_str())
        .bind(branch)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to get latest analysis: {}",
                e
            ))
        })?;

        if let Some(row) = row {
            let metadata_value: Option<serde_json::Value> = row.get("metadata");
            let metadata = metadata_value
                .as_ref()
                .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

            Ok(Some(StoredAnalysis {
                id: AnalysisId(row.get("id")),
                project_id: ProjectId(row.get("project_id")),
                branch: row.get("branch"),
                commit_hash: row.get("commit_hash"),
                findings_count: row.get::<i32, _>("findings_count") as u32,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }
}

/// PostgreSQL-specific implementation of baseline repository
#[derive(Clone)]
pub struct PostgresBaselineRepository {
    pool: PgPool,
}

impl PostgresBaselineRepository {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl BaselineRepository for PostgresBaselineRepository {
    async fn save_baseline_status(
        &self,
        update: crate::domain::models::BaselineStatusUpdate,
    ) -> DomainResult<()> {
        let status_str = match update.status {
            crate::domain::models::BaselineStatus::Accepted => "accepted",
            crate::domain::models::BaselineStatus::FalsePositive => "false_positive",
            crate::domain::models::BaselineStatus::WontFix => "wont_fix",
        };

        let expires_at = update.expires_at.map(|dt| dt.naive_utc());

        sqlx::query(
            r#"
            INSERT INTO baseline_status (project_id, branch, fingerprint, status, reason, expires_at, user_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (project_id, branch, fingerprint)
            DO UPDATE SET
                status = EXCLUDED.status,
                reason = EXCLUDED.reason,
                expires_at = EXCLUDED.expires_at,
                user_id = EXCLUDED.user_id,
                updated_at = NOW()
        "#,
        )
        .bind(update.project_id.as_str())
        .bind(update.branch)
        .bind(update.finding_fingerprint)
        .bind(status_str)
        .bind(update.reason)
        .bind(expires_at)
        .bind(update.user_id.map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to save baseline status: {}", e))
        })?;

        Ok(())
    }

    async fn get_baseline_statuses(
        &self,
        project_id: &ProjectId,
        branch: &str,
    ) -> DomainResult<Vec<crate::domain::models::BaselineStatus>> {
        let rows = sqlx::query(
            r#"
            SELECT status, expires_at
            FROM baseline_status
            WHERE project_id = $1 AND branch = $2
            AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        )
        .bind(project_id.as_str())
        .bind(branch)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to get baseline statuses: {}",
                e
            ))
        })?;

        let statuses = rows
            .into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                match status_str.as_str() {
                    "accepted" => crate::domain::models::BaselineStatus::Accepted,
                    "false_positive" => crate::domain::models::BaselineStatus::FalsePositive,
                    "wont_fix" => crate::domain::models::BaselineStatus::WontFix,
                    _ => crate::domain::models::BaselineStatus::Accepted,
                }
            })
            .collect();

        Ok(statuses)
    }

    async fn is_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
    ) -> DomainResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM baseline_status
            WHERE project_id = $1 AND branch = $2 AND fingerprint = $3
            AND (expires_at IS NULL OR expires_at > NOW())
        "#,
        )
        .bind(project_id.as_str())
        .bind(branch)
        .bind(fingerprint)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to check baseline status: {}",
                e
            ))
        })?;

        Ok(row.is_some())
    }

    async fn mark_as_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
        status: crate::domain::models::BaselineStatus,
        reason: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
        user_id: Option<crate::domain::models::UserId>,
    ) -> DomainResult<()> {
        let status_str = match status {
            crate::domain::models::BaselineStatus::Accepted => "accepted",
            crate::domain::models::BaselineStatus::FalsePositive => "false_positive",
            crate::domain::models::BaselineStatus::WontFix => "wont_fix",
        };

        let expires_at_naive = expires_at.map(|dt| dt.naive_utc());

        sqlx::query(
            r#"
            INSERT INTO baseline_status (project_id, branch, fingerprint, status, reason, expires_at, user_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            ON CONFLICT (project_id, branch, fingerprint)
            DO UPDATE SET
                status = EXCLUDED.status,
                reason = EXCLUDED.reason,
                expires_at = EXCLUDED.expires_at,
                user_id = EXCLUDED.user_id,
                updated_at = NOW()
        "#,
        )
        .bind(project_id.as_str())
        .bind(branch)
        .bind(fingerprint)
        .bind(status_str)
        .bind(reason)
        .bind(expires_at_naive)
        .bind(user_id.map(|u| u.0))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to mark as baseline: {}", e))
        })?;

        Ok(())
    }

    async fn remove_from_baseline(
        &self,
        project_id: &ProjectId,
        branch: &str,
        fingerprint: &str,
    ) -> DomainResult<()> {
        sqlx::query(
            r#"
            DELETE FROM baseline_status
            WHERE project_id = $1 AND branch = $2 AND fingerprint = $3
        "#,
        )
        .bind(project_id.as_str())
        .bind(branch)
        .bind(fingerprint)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to remove from baseline: {}",
                e
            ))
        })?;

        Ok(())
    }
}
