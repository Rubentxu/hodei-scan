use crate::domain::error::DomainResult;
/// SQLite Database Adapter
/// This module is only compiled when the "sqlite" feature is enabled
use crate::domain::models::*;
use crate::domain::ports::repositories::*;
use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

/// SQLite-specific implementation of analysis repository
#[derive(Clone)]
pub struct SqliteAnalysisRepository {
    pool: SqlitePool,
}

impl SqliteAnalysisRepository {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analyses (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                branch TEXT NOT NULL,
                commit_hash TEXT NOT NULL,
                findings_count INTEGER NOT NULL,
                metadata TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                analysis_id TEXT NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
                fact_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                location TEXT,
                metadata TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl AnalysisRepository for SqliteAnalysisRepository {
    async fn save_analysis(
        &self,
        analysis: StoredAnalysis,
        findings: &[Finding],
    ) -> DomainResult<AnalysisId> {
        // Convert analysis to JSON for storage
        let metadata_json = serde_json::to_string(&analysis.metadata).map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to serialize metadata: {}",
                e
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO analyses (id, project_id, branch, commit_hash, findings_count, metadata)
            VALUES (?, ?, ?, ?, ?, ?)
        "#,
        )
        .bind(analysis.id.0.to_string())
        .bind(analysis.project_id.as_str())
        .bind(analysis.branch)
        .bind(analysis.commit_hash)
        .bind(analysis.findings_count as i64)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to save analysis: {}", e))
        })?;

        // Save findings
        for finding in findings {
            let location_json = serde_json::to_string(&finding.location).map_err(|e| {
                crate::domain::error::DomainError::internal(&format!(
                    "Failed to serialize location: {}",
                    e
                ))
            })?;
            let metadata_json =
                serde_json::to_string(&finding.metadata).unwrap_or_else(|_| "null".to_string());

            sqlx::query(r#"
                INSERT INTO findings (analysis_id, fact_type, severity, fingerprint, location, metadata)
                VALUES (?, ?, ?, ?, ?, ?)
            "#)
            .bind(analysis.id.0.to_string())
            .bind(&finding.fact_type)
            .bind(finding.severity.to_string())
            .bind(&finding.fingerprint)
            .bind(&location_json)
            .bind(&metadata_json)
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
            WHERE id = ?
        "#,
        )
        .bind(id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to get analysis: {}", e))
        })?;

        if let Some(row) = row {
            let metadata: Option<serde_json::Value> = row.try_get("metadata")?;
            let parsed_metadata = metadata
                .as_ref()
                .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

            Ok(Some(StoredAnalysis {
                id: AnalysisId(uuid::Uuid::parse_str(row.get("id")).unwrap()),
                project_id: ProjectId(row.get("project_id")),
                branch: row.get("branch"),
                commit_hash: row.get("commit_hash"),
                findings_count: row.get("findings_count"),
                metadata: parsed_metadata,
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
            WHERE project_id = ?
            ORDER BY created_at DESC
            LIMIT ?
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

        let mut analyses = Vec::new();
        for row in rows {
            let metadata: Option<serde_json::Value> = row.try_get("metadata")?;
            let parsed_metadata = metadata
                .as_ref()
                .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

            analyses.push(StoredAnalysis {
                id: AnalysisId(uuid::Uuid::parse_str(row.get("id")).unwrap()),
                project_id: ProjectId(row.get("project_id")),
                branch: row.get("branch"),
                commit_hash: row.get("commit_hash"),
                findings_count: row.get("findings_count"),
                metadata: parsed_metadata,
            });
        }

        Ok(analyses)
    }

    async fn get_analysis_findings(&self, analysis_id: &AnalysisId) -> DomainResult<Vec<Finding>> {
        let rows = sqlx::query(
            r#"
            SELECT fact_type, severity, fingerprint, location, metadata
            FROM findings
            WHERE analysis_id = ?
        "#,
        )
        .bind(analysis_id.0.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!("Failed to get findings: {}", e))
        })?;

        let mut findings = Vec::new();
        for row in rows {
            let location: serde_json::Value = row.try_get("location")?;
            let metadata: Option<serde_json::Value> = row.try_get("metadata")?;

            let finding = Finding {
                fact_type: row.get("fact_type"),
                severity: match row.get::<String, _>("severity").as_str() {
                    "critical" => Severity::Critical,
                    "major" => Severity::Major,
                    "minor" => Severity::Minor,
                    _ => Severity::Info,
                },
                fingerprint: row.get("fingerprint"),
                location: serde_json::from_value(location).map_err(|e| {
                    crate::domain::error::DomainError::internal(&format!(
                        "Failed to deserialize location: {}",
                        e
                    ))
                })?,
                metadata: metadata.and_then(|v| serde_json::from_value(v).ok()),
            };

            findings.push(finding);
        }

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
            WHERE project_id = ? AND branch = ?
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
            let metadata: Option<serde_json::Value> = row.try_get("metadata")?;
            let parsed_metadata = metadata
                .as_ref()
                .and_then(|v| serde_json::from_value::<AnalysisMetadata>(v.clone()).ok());

            Ok(Some(StoredAnalysis {
                id: AnalysisId(uuid::Uuid::parse_str(row.get("id")).unwrap()),
                project_id: ProjectId(row.get("project_id")),
                branch: row.get("branch"),
                commit_hash: row.get("commit_hash"),
                findings_count: row.get("findings_count"),
                metadata: parsed_metadata,
            }))
        } else {
            Ok(None)
        }
    }
}
