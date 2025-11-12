/// Database connection and operations
use crate::modules::error::{Result, ServerError};
use crate::modules::types::{
    AnalysisId, AnalysisMetadata, Finding, FindingLocation, FindingStatus, ProjectId, Severity,
    StoredAnalysis, User, UserId,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool, Row};
use std::collections::HashMap;

/// Database connection pool
#[derive(Clone)]
pub struct DatabaseConnection {
    pool: PgPool,
}

impl DatabaseConnection {
    /// Create a new database connection pool
    pub async fn new(database_url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(database_url)
            .await
            .map_err(ServerError::Database)?;

        Ok(Self { pool })
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Initialize database schema
    pub async fn initialize_schema(&self) -> Result<()> {
        // Create projects table
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS projects (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    default_branch TEXT NOT NULL DEFAULT 'main',
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        // Create analyses table
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS analyses (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                    branch TEXT NOT NULL,
                    commit_hash TEXT NOT NULL,
                    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    findings_count INTEGER NOT NULL DEFAULT 0,
                    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        // Create findings table
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS findings (
                    id BIGSERIAL PRIMARY KEY,
                    analysis_id UUID NOT NULL REFERENCES analyses(id) ON DELETE CASCADE,
                    fact_type TEXT NOT NULL,
                    severity TEXT NOT NULL CHECK (severity IN ('critical', 'major', 'minor', 'info')),
                    file_path TEXT NOT NULL,
                    line_number INTEGER NOT NULL,
                    column_number INTEGER NOT NULL,
                    end_line INTEGER,
                    end_column INTEGER,
                    message TEXT NOT NULL,
                    metadata JSONB DEFAULT '{}'::jsonb,
                    tags TEXT[] DEFAULT '{}',
                    fingerprint TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        // Create baseline_status table
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS baseline_status (
                    id BIGSERIAL PRIMARY KEY,
                    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                    finding_fingerprint TEXT NOT NULL,
                    status TEXT NOT NULL CHECK (status IN ('active', 'accepted', 'wontfix', 'false_positive')),
                    reason TEXT,
                    expires_at TIMESTAMPTZ,
                    updated_by UUID NOT NULL,
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    UNIQUE(project_id, finding_fingerprint)
                );
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        // Create users table
        self.pool
            .execute(
                r#"
                CREATE TABLE IF NOT EXISTS users (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    password_hash TEXT NOT NULL,
                    role TEXT NOT NULL CHECK (role IN ('admin', 'developer', 'viewer')),
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        // Create indexes for performance
        self.pool
            .execute("CREATE INDEX IF NOT EXISTS idx_analyses_project_branch ON analyses(project_id, branch);")
            .await
            .map_err(ServerError::Database)?;

        self.pool
            .execute("CREATE INDEX IF NOT EXISTS idx_analyses_timestamp ON analyses(timestamp);")
            .await
            .map_err(ServerError::Database)?;

        self.pool
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_findings_analysis_id ON findings(analysis_id);",
            )
            .await
            .map_err(ServerError::Database)?;

        self.pool
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_findings_fingerprint ON findings(fingerprint);",
            )
            .await
            .map_err(ServerError::Database)?;

        self.pool
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_baseline_project ON baseline_status(project_id);",
            )
            .await
            .map_err(ServerError::Database)?;

        // Create views for trend analysis
        self.pool
            .execute(
                r#"
                CREATE OR REPLACE VIEW findings_trend_daily AS
                SELECT
                    DATE_TRUNC('day', a.timestamp) as day,
                    a.project_id,
                    a.branch,
                    f.severity,
                    f.fact_type,
                    COUNT(*) as count
                FROM analyses a
                JOIN findings f ON f.analysis_id = a.id
                GROUP BY day, a.project_id, a.branch, f.severity, f.fact_type;
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        self.pool
            .execute(
                r#"
                CREATE OR REPLACE VIEW project_summary AS
                SELECT
                    p.id as project_id,
                    p.name,
                    COUNT(DISTINCT a.id) as total_analyses,
                    COUNT(DISTINCT f.id) as total_findings,
                    COUNT(DISTINCT CASE WHEN f.severity = 'critical' THEN f.id END) as critical_findings,
                    COUNT(DISTINCT CASE WHEN f.severity = 'major' THEN f.id END) as major_findings,
                    MIN(a.timestamp) as first_analysis,
                    MAX(a.timestamp) as last_analysis
                FROM projects p
                LEFT JOIN analyses a ON a.project_id = p.id
                LEFT JOIN findings f ON f.analysis_id = a.id
                GROUP BY p.id, p.name;
                "#,
            )
            .await
            .map_err(ServerError::Database)?;

        Ok(())
    }

    /// Store a new analysis with findings
    pub async fn store_analysis(
        &self,
        project_id: &str,
        branch: &str,
        commit: &str,
        findings: &[Finding],
        metadata: &AnalysisMetadata,
    ) -> Result<AnalysisId> {
        let analysis_id = AnalysisId::new_v4();

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(ServerError::Database)?;

        // Insert the analysis
        sqlx::query!(
            r#"
            INSERT INTO analyses (id, project_id, branch, commit_hash, findings_count, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            analysis_id,
            project_id,
            branch,
            commit,
            findings.len() as i32,
            serde_json::to_value(metadata).unwrap_or_default(),
        )
        .execute(&mut *tx)
        .await
        .map_err(ServerError::Database)?;

        // Insert findings in batches for performance
        for chunk in findings.chunks(1000) {
            for finding in chunk {
                sqlx::query!(
                    r#"
                    INSERT INTO findings (analysis_id, fact_type, severity, file_path, line_number,
                                         column_number, end_line, end_column, message, metadata, tags, fingerprint)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    "#,
                    analysis_id,
                    finding.fact_type,
                    finding.severity.to_string().to_lowercase(),
                    finding.location.file,
                    finding.location.line as i32,
                    finding.location.column as i32,
                    finding.location.end_line.map(|x| x as i32),
                    finding.location.end_column.map(|x| x as i32),
                    finding.message,
                    finding.metadata.as_ref().unwrap_or(&Value::Null),
                    &finding.tags,
                    finding.fingerprint,
                )
                .execute(&mut *tx)
                .await
                .map_err(ServerError::Database)?;
            }
        }

        // Commit transaction
        tx.commit().await.map_err(ServerError::Database)?;

        Ok(analysis_id)
    }

    /// Get the latest analysis for a project and branch
    pub async fn get_latest_analysis(
        &self,
        project_id: &str,
        branch: &str,
    ) -> Result<Option<StoredAnalysis>> {
        let row = sqlx::query!(
            r#"
            SELECT id, project_id, branch, commit_hash, timestamp, findings_count, metadata, created_at
            FROM analyses
            WHERE project_id = $1 AND branch = $2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            project_id,
            branch,
        )
        .fetch_optional(self.pool())
        .await
        .map_err(ServerError::Database)?;

        match row {
            Some(row) => Ok(Some(StoredAnalysis {
                id: row.id,
                project_id: row.project_id,
                branch: row.branch,
                commit: row.commit_hash,
                timestamp: row.timestamp,
                findings_count: row.findings_count as u32,
                metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Get all findings for an analysis
    pub async fn get_findings_by_analysis(&self, analysis_id: &AnalysisId) -> Result<Vec<Finding>> {
        let rows = sqlx::query!(
            r#"
            SELECT fact_type, severity, file_path, line_number, column_number,
                   end_line, end_column, message, metadata, tags, fingerprint
            FROM findings
            WHERE analysis_id = $1
            ORDER BY line_number, column_number
            "#,
            analysis_id,
        )
        .fetch_all(self.pool())
        .await
        .map_err(ServerError::Database)?;

        let findings = rows
            .into_iter()
            .map(|row| Finding {
                fact_type: row.fact_type,
                severity: match row.severity.as_str() {
                    "critical" => Severity::Critical,
                    "major" => Severity::Major,
                    "minor" => Severity::Minor,
                    _ => Severity::Info,
                },
                location: FindingLocation {
                    file: row.file_path,
                    line: row.line_number as u32,
                    column: row.column_number as u32,
                    end_line: row.end_line.map(|x| x as u32),
                    end_column: row.end_column.map(|x| x as u32),
                },
                message: row.message,
                metadata: Some(row.metadata),
                tags: row.tags.unwrap_or_default(),
                fingerprint: row.fingerprint,
            })
            .collect();

        Ok(findings)
    }

    /// Get trend metrics for a project
    pub async fn get_trend_metrics(
        &self,
        project_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<HashMap<String, u64>> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_findings,
                COUNT(CASE WHEN severity = 'critical' THEN 1 END) as critical_findings,
                COUNT(CASE WHEN severity = 'major' THEN 1 END) as major_findings,
                COUNT(CASE WHEN severity = 'minor' THEN 1 END) as minor_findings,
                COUNT(CASE WHEN severity = 'info' THEN 1 END) as info_findings
            FROM findings f
            JOIN analyses a ON a.id = f.analysis_id
            WHERE a.project_id = $1 AND a.timestamp BETWEEN $2 AND $3
            "#,
            project_id,
            start,
            end,
        )
        .fetch_one(self.pool())
        .await
        .map_err(ServerError::Database)?;

        let mut metrics = HashMap::new();
        metrics.insert("total_findings".to_string(), row.total_findings as u64);
        metrics.insert(
            "critical_findings".to_string(),
            row.critical_findings as u64,
        );
        metrics.insert("major_findings".to_string(), row.major_findings as u64);
        metrics.insert("minor_findings".to_string(), row.minor_findings as u64);
        metrics.insert("info_findings".to_string(), row.info_findings as u64);

        Ok(metrics)
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<bool> {
        sqlx::query!("SELECT 1 as health_check")
            .fetch_one(self.pool())
            .await
            .map(|_| true)
            .map_err(ServerError::Database)
    }
}
