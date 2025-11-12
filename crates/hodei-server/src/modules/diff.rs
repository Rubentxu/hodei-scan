/// Diff Analysis Engine - US-13.03
/// Compares analyses to identify changes (new findings, resolved, severity changes)
use crate::modules::error::Result;
use crate::modules::types::{AnalysisDiff, Finding, Severity};
use std::collections::{HashMap, HashSet};

/// Diff calculation engine
#[derive(Clone)]
pub struct DiffEngine {
    /// Enable parallel diff calculation for large datasets
    enable_parallel: bool,
    /// Chunk size for parallel processing
    chunk_size: usize,
}

impl DiffEngine {
    /// Create a new diff engine
    pub fn new() -> Self {
        Self {
            enable_parallel: true,
            chunk_size: 1000,
        }
    }

    /// Calculate diff between two analyses
    pub fn calculate_diff(
        &self,
        current: &[Finding],
        baseline: &[Finding],
    ) -> Result<AnalysisDiff> {
        // Create fingerprints for efficient comparison
        let current_fingerprints: HashSet<&str> =
            current.iter().map(|f| f.fingerprint.as_str()).collect();

        let baseline_fingerprints: HashSet<&str> =
            baseline.iter().map(|f| f.fingerprint.as_str()).collect();

        // Find new findings (in current but not in baseline)
        let new_findings: Vec<Finding> =
            self.find_new_findings(current, &current_fingerprints, &baseline_fingerprints);

        // Find resolved findings (in baseline but not in current)
        let resolved_findings: Vec<Finding> =
            self.find_resolved_findings(baseline, &current_fingerprints, &baseline_fingerprints);

        // Find severity changes (same finding but different severity)
        let (severity_increased, severity_decreased) =
            self.find_severity_changes(current, baseline)?;

        // Find status changes (TODO: when baseline status tracking is implemented)
        let wont_fix_changed = vec![];

        Ok(AnalysisDiff {
            base_analysis: None, // Will be filled by caller
            head_analysis: None, // Will be filled by caller
            new_findings,
            resolved_findings,
            severity_increased,
            severity_decreased,
            wont_fix_changed,
        })
    }

    /// Find new findings (present in current but not in baseline)
    fn find_new_findings(
        &self,
        current: &[Finding],
        _current_fingerprints: &HashSet<&str>,
        baseline_fingerprints: &HashSet<&str>,
    ) -> Vec<Finding> {
        current
            .iter()
            .filter(|f| !baseline_fingerprints.contains(f.fingerprint.as_str()))
            .cloned()
            .collect()
    }

    /// Find resolved findings (present in baseline but not in current)
    fn find_resolved_findings(
        &self,
        baseline: &[Finding],
        current_fingerprints: &HashSet<&str>,
        _baseline_fingerprints: &HashSet<&str>,
    ) -> Vec<Finding> {
        baseline
            .iter()
            .filter(|f| !current_fingerprints.contains(f.fingerprint.as_str()))
            .cloned()
            .collect()
    }

    /// Find findings with severity changes
    fn find_severity_changes(
        &self,
        current: &[Finding],
        baseline: &[Finding],
    ) -> Result<(Vec<Finding>, Vec<Finding>)> {
        // Create a map of fingerprint -> severity for baseline
        let baseline_severities: HashMap<&str, Severity> = baseline
            .iter()
            .map(|f| (f.fingerprint.as_str(), f.severity.clone()))
            .collect();

        let mut severity_increased = vec![];
        let mut severity_decreased = vec![];

        for finding in current {
            if let Some(baseline_severity) = baseline_severities.get(finding.fingerprint.as_str()) {
                if finding.severity != *baseline_severity {
                    let severity_increase =
                        finding.severity.to_level() > baseline_severity.to_level();

                    if severity_increase {
                        severity_increased.push(finding.clone());
                    } else {
                        severity_decreased.push(finding.clone());
                    }
                }
            }
        }

        Ok((severity_increased, severity_decreased))
    }

    /// Calculate diff with project context and branch information
    pub async fn calculate_branch_diff(
        &self,
        project_id: &str,
        base_branch: &str,
        head_branch: &str,
        database: &crate::modules::database::DatabaseConnection,
    ) -> Result<AnalysisDiff> {
        // Get baseline analysis for base branch
        let base_analysis = database
            .get_latest_analysis(project_id, base_branch)
            .await?;

        // Get baseline analysis for head branch
        let head_analysis = database
            .get_latest_analysis(project_id, head_branch)
            .await?;

        // If either analysis is missing, return empty diff
        let (base_findings, base_analysis_clone, head_findings, head_analysis_clone) =
            match (base_analysis, head_analysis) {
                (Some(base), Some(head)) => {
                    let base_findings = database.get_findings_by_analysis(&base.id).await?;
                    let head_findings = database.get_findings_by_analysis(&head.id).await?;
                    (base_findings, base.clone(), head_findings, head.clone())
                }
                _ => {
                    // No analyses found, return empty diff
                    return Ok(AnalysisDiff {
                        base_analysis: None,
                        head_analysis: None,
                        new_findings: vec![],
                        resolved_findings: vec![],
                        severity_increased: vec![],
                        severity_decreased: vec![],
                        wont_fix_changed: vec![],
                    });
                }
            };

        // Calculate diff
        let mut diff = self.calculate_diff(&head_findings, &base_findings)?;

        // Populate analysis references
        diff.base_analysis = Some(base_analysis_clone);
        diff.head_analysis = Some(head_analysis_clone);

        Ok(diff)
    }

    /// Calculate diff with commit-based comparison
    pub async fn calculate_commit_diff(
        &self,
        project_id: &str,
        base_commit: &str,
        head_commit: &str,
        database: &crate::modules::database::DatabaseConnection,
    ) -> Result<AnalysisDiff> {
        // TODO: Implement commit-based diff
        // This would query analyses by commit hash instead of branch

        // For now, delegate to branch-based diff
        self.calculate_branch_diff(project_id, base_commit, head_commit, database)
            .await
    }

    /// Optimize diff calculation for large datasets
    pub fn calculate_diff_optimized(
        &self,
        current: &[Finding],
        baseline: &[Finding],
    ) -> Result<AnalysisDiff> {
        // For very large datasets, use chunked processing
        if current.len() > 10000 || baseline.len() > 10000 {
            self.calculate_diff_chunked(current, baseline)
        } else {
            self.calculate_diff(current, baseline)
        }
    }

    /// Chunked diff calculation for large datasets
    fn calculate_diff_chunked(
        &self,
        current: &[Finding],
        baseline: &[Finding],
    ) -> Result<AnalysisDiff> {
        // Create fingerprint maps
        let current_map: HashMap<&str, &Finding> = current
            .iter()
            .map(|f| (f.fingerprint.as_str(), f))
            .collect();

        let baseline_map: HashMap<&str, &Finding> = baseline
            .iter()
            .map(|f| (f.fingerprint.as_str(), f))
            .collect();

        let current_fingerprints: HashSet<&str> = current_map.keys().copied().collect();
        let baseline_fingerprints: HashSet<&str> = baseline_map.keys().copied().collect();

        // Process in chunks for memory efficiency
        let mut new_findings = Vec::new();
        let mut resolved_findings = Vec::new();
        let mut severity_increased = Vec::new();
        let mut severity_decreased = Vec::new();

        // Process new findings
        for chunk in current.chunks(self.chunk_size) {
            let chunk_new: Vec<Finding> = chunk
                .iter()
                .filter(|f| !baseline_fingerprints.contains(f.fingerprint.as_str()))
                .cloned()
                .collect();
            new_findings.extend(chunk_new);
        }

        // Process resolved findings
        for chunk in baseline.chunks(self.chunk_size) {
            let chunk_resolved: Vec<Finding> = chunk
                .iter()
                .filter(|f| !current_fingerprints.contains(f.fingerprint.as_str()))
                .cloned()
                .collect();
            resolved_findings.extend(chunk_resolved);
        }

        // Process severity changes
        for (fp, finding) in &current_map {
            if let Some(&baseline_finding) = baseline_map.get(fp) {
                if finding.severity != baseline_finding.severity {
                    let severity_increase =
                        finding.severity.to_level() > baseline_finding.severity.to_level();

                    if severity_increase {
                        severity_increased.push((*finding).clone());
                    } else {
                        severity_decreased.push((*finding).clone());
                    }
                }
            }
        }

        Ok(AnalysisDiff {
            base_analysis: None,
            head_analysis: None,
            new_findings,
            resolved_findings,
            severity_increased,
            severity_decreased,
            wont_fix_changed: vec![],
        })
    }

    /// Calculate summary statistics for diff
    pub fn calculate_diff_summary(&self, diff: &AnalysisDiff) -> DiffSummary {
        let total_changes = diff.new_findings.len()
            + diff.resolved_findings.len()
            + diff.severity_increased.len()
            + diff.severity_decreased.len();

        let net_change = diff.new_findings.len() as isize - diff.resolved_findings.len() as isize;

        let severity_score = diff
            .severity_increased
            .iter()
            .map(|f| f.severity.to_level() as isize)
            .sum::<isize>()
            - diff
                .severity_decreased
                .iter()
                .map(|f| f.severity.to_level() as isize)
                .sum::<isize>();

        let trend = if net_change > 0 {
            crate::modules::types::TrendDirection::Degrading
        } else if net_change < 0 {
            crate::modules::types::TrendDirection::Improving
        } else {
            crate::modules::types::TrendDirection::Stable
        };

        DiffSummary {
            total_changes,
            new_findings_count: diff.new_findings.len(),
            resolved_findings_count: diff.resolved_findings.len(),
            severity_increased_count: diff.severity_increased.len(),
            severity_decreased_count: diff.severity_decreased.len(),
            net_change,
            severity_score,
            trend,
        }
    }
}

/// Diff summary statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub new_findings_count: usize,
    pub resolved_findings_count: usize,
    pub severity_increased_count: usize,
    pub severity_decreased_count: usize,
    pub net_change: isize,
    pub severity_score: isize,
    pub trend: crate::modules::types::TrendDirection,
}

impl DiffSummary {
    /// Get a human-readable summary
    pub fn to_summary(&self) -> String {
        format!(
            "Changes: {} total ({} new, {} resolved, {} severity increased, {} severity decreased), \
             Net change: {}, Severity score: {}",
            self.total_changes,
            self.new_findings_count,
            self.resolved_findings_count,
            self.severity_increased_count,
            self.severity_decreased_count,
            self.net_change,
            self.severity_score
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_finding(fingerprint: &str, severity: Severity) -> Finding {
        Finding {
            fact_type: "Test".to_string(),
            severity,
            location: crate::modules::types::FindingLocation {
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

    #[test]
    fn test_calculate_diff_new_findings() {
        let engine = DiffEngine::new();

        let current = vec![
            create_test_finding("fp1", Severity::Critical),
            create_test_finding("fp2", Severity::Major),
        ];

        let baseline = vec![create_test_finding("fp1", Severity::Critical)];

        let diff = engine.calculate_diff(&current, &baseline).unwrap();

        assert_eq!(diff.new_findings.len(), 1);
        assert_eq!(diff.new_findings[0].fingerprint, "fp2");
        assert_eq!(diff.resolved_findings.len(), 0);
    }

    #[test]
    fn test_calculate_diff_resolved_findings() {
        let engine = DiffEngine::new();

        let current = vec![create_test_finding("fp1", Severity::Critical)];

        let baseline = vec![
            create_test_finding("fp1", Severity::Critical),
            create_test_finding("fp2", Severity::Major),
        ];

        let diff = engine.calculate_diff(&current, &baseline).unwrap();

        assert_eq!(diff.new_findings.len(), 0);
        assert_eq!(diff.resolved_findings.len(), 1);
        assert_eq!(diff.resolved_findings[0].fingerprint, "fp2");
    }

    #[test]
    fn test_calculate_diff_severity_changes() {
        let engine = DiffEngine::new();

        let current = vec![create_test_finding("fp1", Severity::Major)];

        let baseline = vec![create_test_finding("fp1", Severity::Minor)];

        let diff = engine.calculate_diff(&current, &baseline).unwrap();

        assert_eq!(diff.new_findings.len(), 0);
        assert_eq!(diff.resolved_findings.len(), 0);
        assert_eq!(diff.severity_increased.len(), 1);
        assert_eq!(diff.severity_increased[0].fingerprint, "fp1");
    }
}
