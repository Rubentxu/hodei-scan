/// Rate limiting and data retention policies
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    /// Request count per minute per key
    requests_per_minute: u64,
    /// Current token bucket state
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: u64,
    last_refill: SystemTime,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u64) -> Self {
        Self {
            requests_per_minute,
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if request is allowed for the given key
    pub async fn check_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let mut buckets = self.buckets.write().await;
        let bucket = buckets
            .entry(key.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: self.requests_per_minute,
                last_refill: SystemTime::now(),
            });

        // Refill tokens based on time passed
        let now = SystemTime::now();
        let elapsed = now.duration_since(bucket.last_refill).unwrap_or_default();
        let tokens_to_add = elapsed.as_secs() / 60 * self.requests_per_minute;

        if tokens_to_add > 0 {
            bucket.tokens = (bucket.tokens + tokens_to_add).min(self.requests_per_minute);
            bucket.last_refill = now;
        }

        // Check if we have tokens
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            Err(RateLimitError::TooManyRequests {
                limit: self.requests_per_minute,
                window: Duration::from_secs(60),
            })
        }
    }

    /// Clean up old buckets to prevent memory leaks
    pub async fn cleanup_old_buckets(&self) {
        let mut buckets = self.buckets.write().await;
        let now = SystemTime::now();

        // Remove buckets inactive for more than 1 hour
        let cutoff = now - Duration::from_secs(3600);

        buckets.retain(|_, bucket| bucket.last_refill > cutoff);

        info!(
            "Rate limiter: cleaned up buckets, active: {}",
            buckets.len()
        );
    }
}

/// Rate limit error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {limit} requests per {window:?}")]
    TooManyRequests { limit: u64, window: Duration },
}

impl RateLimitError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        axum::http::StatusCode::TOO_MANY_REQUESTS
    }
}

/// Data retention policy manager
pub struct RetentionManager {
    /// Default retention period in days
    default_retention_days: u64,
    /// Project-specific retention policies
    project_policies: Arc<RwLock<HashMap<String, RetentionPolicy>>>,
}

#[derive(Clone)]
struct RetentionPolicy {
    retention_days: u64,
    archive_after_days: u64, // Move to archive table after this period
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 365,    // Keep for 1 year
            archive_after_days: 90, // Archive after 90 days
        }
    }
}

impl RetentionManager {
    pub fn new(default_retention_days: u64) -> Self {
        Self {
            default_retention_days,
            project_policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get retention policy for a project
    pub async fn get_policy(&self, project_id: &str) -> RetentionPolicy {
        let policies = self.project_policies.read().await;
        policies
            .get(project_id)
            .cloned()
            .unwrap_or_else(|| RetentionPolicy {
                retention_days: self.default_retention_days,
                archive_after_days: self.default_retention_days / 4, // Quarter of retention period
            })
    }

    /// Set custom retention policy for a project
    pub async fn set_policy(&self, project_id: &str, policy: RetentionPolicy) {
        let mut policies = self.project_policies.write().await;
        policies.insert(project_id.to_string(), policy);
        info!(
            "Set retention policy for project {}: {} days",
            project_id, policy.retention_days
        );
    }

    /// Get analyses older than retention period
    pub async fn get_expired_analyses(
        &self,
        database: &crate::modules::database::DatabaseConnection,
    ) -> Result<Vec<String>, crate::modules::error::ServerError> {
        // TODO: Implement actual database query
        // This would query analyses older than retention period

        let policy = self.get_policy("default").await;
        let cutoff = SystemTime::now() - Duration::from_secs(policy.retention_days * 24 * 60 * 60);
        let cutoff_unix = cutoff.duration_since(UNIX_EPOCH).unwrap().as_secs();

        warn!(
            "Retention: Would delete analyses older than {} days",
            policy.retention_days
        );

        // Mock implementation - return empty list
        Ok(vec![])
    }

    /// Clean up expired analyses
    pub async fn cleanup_expired_analyses(
        &self,
        database: &crate::modules::database::DatabaseConnection,
    ) -> Result<CleanupSummary, crate::modules::error::ServerError> {
        info!("Starting data retention cleanup");

        let expired_analysis_ids = self.get_expired_analyses(database).await?;

        let mut deleted_analyses = 0;
        let mut deleted_findings = 0;

        // TODO: Implement actual deletion from database
        for analysis_id in &expired_analysis_ids {
            // Find all findings for this analysis
            // Delete findings
            // Delete analysis record
            deleted_findings += 100; // Mock
            deleted_analyses += 1;
        }

        let summary = CleanupSummary {
            analyses_deleted: deleted_analyses,
            findings_deleted: deleted_findings,
            cutoff_date: SystemTime::now()
                - Duration::from_secs(self.default_retention_days * 24 * 60 * 60),
        };

        info!(
            "Cleanup completed: {} analyses, {} findings deleted",
            summary.analyses_deleted, summary.findings_deleted
        );

        Ok(summary)
    }
}

/// Cleanup operation summary
pub struct CleanupSummary {
    pub analyses_deleted: u64,
    pub findings_deleted: u64,
    pub cutoff_date: SystemTime,
}

/// Background cleanup task
pub struct CleanupTask {
    retention_manager: RetentionManager,
    database: crate::modules::database::DatabaseConnection,
    interval: Duration,
}

impl CleanupTask {
    pub fn new(
        retention_manager: RetentionManager,
        database: crate::modules::database::DatabaseConnection,
        interval_hours: u64,
    ) -> Self {
        Self {
            retention_manager,
            database,
            interval: Duration::from_secs(interval_hours * 60 * 60),
        }
    }

    /// Start the background cleanup task
    pub async fn run(self) {
        let mut interval = tokio::time::interval(self.interval);

        info!(
            "Started data retention cleanup task, interval: {} hours",
            self.interval.as_secs() / 3600
        );

        loop {
            interval.tick().await;

            match self
                .retention_manager
                .cleanup_expired_analyses(&self.database)
                .await
            {
                Ok(summary) => {
                    info!(
                        "Retention cleanup completed: {} analyses deleted",
                        summary.analyses_deleted
                    );
                }
                Err(e) => {
                    warn!("Retention cleanup failed: {}", e);
                }
            }
        }
    }
}

/// Analysis summary for response
pub fn create_analysis_summary(
    analysis_id: uuid::Uuid,
    findings_count: u32,
    new_findings: u32,
    resolved_findings: u32,
) -> crate::modules::types::PublishResponse {
    use crate::modules::types::{PublishResponse, Severity, TrendDirection};

    let total_findings = findings_count;
    let trend = if new_findings > resolved_findings {
        TrendDirection::Degrading
    } else if resolved_findings > new_findings {
        TrendDirection::Improving
    } else {
        TrendDirection::Stable
    };

    PublishResponse {
        analysis_id,
        new_findings,
        resolved_findings,
        total_findings,
        trend,
        summary_url: format!("/api/v1/analyses/{}", analysis_id),
    }
}
