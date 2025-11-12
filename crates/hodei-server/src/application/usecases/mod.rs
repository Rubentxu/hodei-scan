use crate::domain::error::{DomainError, DomainResult};
/// Use cases - Business workflows
///
/// Each use case represents a complete business action that can be performed.
/// These orchestrate domain objects and repository operations.
use crate::domain::models::*;
use crate::domain::ports::repositories::*;
use crate::domain::ports::services::*;

/// Publish analysis use case
pub struct PublishAnalysisUseCase {
    analysis_repo: Box<dyn AnalysisRepository>,
    baseline_repo: Box<dyn BaselineRepository>,
    notification_service: Box<dyn NotificationService>,
}

impl PublishAnalysisUseCase {
    pub fn new(
        analysis_repo: Box<dyn AnalysisRepository>,
        baseline_repo: Box<dyn BaselineRepository>,
        notification_service: Box<dyn NotificationService>,
    ) -> Self {
        Self {
            analysis_repo,
            baseline_repo,
            notification_service,
        }
    }

    pub async fn execute(&self, request: PublishRequest) -> DomainResult<PublishResponse> {
        // Validate request
        self.validate_request(&request)?;

        // Get baseline findings to exclude from analysis
        // Only check findings that are in baseline with Accept or WontFix status
        let mut baseline_fingerprints = std::collections::HashSet::new();

        for finding in &request.findings {
            let is_baseline = self
                .baseline_repo
                .is_baseline(&request.project_id, &request.branch, &finding.fingerprint)
                .await?;

            if is_baseline {
                baseline_fingerprints.insert(finding.fingerprint.clone());
            }
        }

        // Filter findings not in baseline
        let filtered_findings: Vec<Finding> = request
            .findings
            .iter()
            .filter(|f| !baseline_fingerprints.contains(&f.fingerprint))
            .cloned()
            .collect();

        // Create analysis entity
        let analysis_id = AnalysisId::new_v4();
        let analysis = StoredAnalysis {
            id: analysis_id.clone(),
            project_id: request.project_id,
            branch: request.branch.clone(),
            commit_hash: request.commit_hash,
            findings_count: filtered_findings.len() as u32,
            metadata: request.metadata,
        };

        // Save analysis
        self.analysis_repo
            .save_analysis(analysis.clone(), &filtered_findings)
            .await?;

        // Calculate metrics
        let (critical, major, minor, info) = self.calculate_severity_counts(&filtered_findings);

        // Send notification
        self.notification_service
            .notify_analysis_published(&analysis.project_id, &analysis_id, analysis.findings_count)
            .await
            .ok(); // Don't fail if notification fails

        Ok(PublishResponse {
            analysis_id,
            total_findings: filtered_findings.len() as u32,
            critical_count: critical,
            major_count: major,
            minor_count: minor,
            info_count: info,
        })
    }

    fn validate_request(&self, request: &PublishRequest) -> DomainResult<()> {
        if request.project_id.as_str().is_empty() {
            return Err(DomainError::invalid_input("project_id cannot be empty"));
        }

        if request.branch.is_empty() {
            return Err(DomainError::invalid_input("branch cannot be empty"));
        }

        if request.commit_hash.is_empty() {
            return Err(DomainError::invalid_input("commit_hash cannot be empty"));
        }

        Ok(())
    }

    fn calculate_severity_counts(&self, findings: &[Finding]) -> (u32, u32, u32, u32) {
        let mut critical = 0;
        let mut major = 0;
        let mut minor = 0;
        let mut info = 0;

        for finding in findings {
            match finding.severity {
                Severity::Critical => critical += 1,
                Severity::Major => major += 1,
                Severity::Minor => minor += 1,
                Severity::Info => info += 1,
            }
        }

        (critical, major, minor, info)
    }
}

/// Update baseline use case
pub struct UpdateBaselineUseCase {
    baseline_repo: Box<dyn BaselineRepository>,
    notification_service: Box<dyn NotificationService>,
}

impl UpdateBaselineUseCase {
    pub fn new(
        baseline_repo: Box<dyn BaselineRepository>,
        notification_service: Box<dyn NotificationService>,
    ) -> Self {
        Self {
            baseline_repo,
            notification_service,
        }
    }

    pub async fn execute(&self, update: BaselineStatusUpdate) -> DomainResult<()> {
        // Validate update
        if update.project_id.as_str().is_empty() {
            return Err(DomainError::invalid_input("project_id cannot be empty"));
        }

        if update.branch.is_empty() {
            return Err(DomainError::invalid_input("branch cannot be empty"));
        }

        if update.finding_fingerprint.is_empty() {
            return Err(DomainError::invalid_input(
                "finding_fingerprint cannot be empty",
            ));
        }

        // Save baseline status
        self.baseline_repo
            .save_baseline_status(update.clone())
            .await?;

        // Send notification
        self.notification_service
            .notify_baseline_updated(&update.project_id, update.branch.as_str(), 1)
            .await
            .ok();

        Ok(())
    }
}
