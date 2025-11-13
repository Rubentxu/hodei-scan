//! Orchestrator for managing multiple extractors in parallel
//!
//! This module implements US-14.1 from EPIC-14: Infrastructure Core de Orquestación

use crate::core::{ExtractorConfig, ExtractorDefinition, ExtractorError, ExtractorRun, IRBuilder};
use futures::future::join_all;
use hodei_ir::{Fact, IntermediateRepresentation, ProjectPath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

/// Configuration for the extractor orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Whether to execute extractors in parallel
    #[serde(default = "default_true")]
    pub parallel_execution: bool,

    /// Maximum number of extractors to run concurrently
    #[serde(default = "default_max_parallel")]
    pub max_parallel_extractors: usize,

    /// Global timeout for all extractors (seconds)
    #[serde(default = "default_global_timeout")]
    pub global_timeout_seconds: u64,
}

fn default_true() -> bool {
    true
}

fn default_max_parallel() -> usize {
    4
}

fn default_global_timeout() -> u64 {
    1800 // 30 minutes
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            parallel_execution: true,
            max_parallel_extractors: 4,
            global_timeout_seconds: 1800,
        }
    }
}

/// Metadata about the aggregation of multiple extractors
#[derive(Debug, Clone, Serialize)]
pub struct AggregationMetadata {
    /// Results from individual extractor runs
    pub extractor_runs: Vec<ExtractorRun>,

    /// Total facts before deduplication
    pub total_facts_before_dedup: usize,

    /// Total facts after deduplication
    pub total_facts_after_dedup: usize,

    /// Deduplication ratio (0.0 = no duplicates, 1.0 = all duplicates)
    pub deduplication_ratio: f64,

    /// Total duration of all extractors
    pub total_duration: Duration,
}

/// Aggregated intermediate representation from multiple extractors
#[derive(Debug)]
pub struct AggregatedIR {
    /// All facts from all extractors (after deduplication)
    pub facts: Vec<Fact>,

    /// Metadata about the aggregation process
    pub metadata: AggregationMetadata,
}

/// Orchestrates the execution of multiple extractors
pub struct ExtractorOrchestrator {
    config: OrchestratorConfig,
    extractors: Vec<ExtractorDefinition>,
    semaphore: Arc<Semaphore>,
}

impl ExtractorOrchestrator {
    /// Create a new orchestrator
    pub fn new(config: OrchestratorConfig, extractors: Vec<ExtractorDefinition>) -> Self {
        let max_parallel = if config.parallel_execution {
            config.max_parallel_extractors
        } else {
            1
        };

        Self {
            config,
            extractors,
            semaphore: Arc::new(Semaphore::new(max_parallel)),
        }
    }

    /// Execute all enabled extractors and aggregate their results
    pub async fn run_all(&self, project_path: &Path) -> Result<AggregatedIR, ExtractorError> {
        let start_time = Instant::now();

        let enabled_extractors: Vec<_> = self.extractors.iter().filter(|e| e.enabled).collect();

        if enabled_extractors.is_empty() {
            warn!("No enabled extractors found");
            return Ok(AggregatedIR {
                facts: Vec::new(),
                metadata: AggregationMetadata {
                    extractor_runs: Vec::new(),
                    total_facts_before_dedup: 0,
                    total_facts_after_dedup: 0,
                    deduplication_ratio: 0.0,
                    total_duration: Duration::from_secs(0),
                },
            });
        }

        info!("Ejecutando {} extractores", enabled_extractors.len());

        // Execute extractors in parallel
        let mut handles = Vec::new();

        for extractor in enabled_extractors {
            let semaphore = Arc::clone(&self.semaphore);
            let extractor = extractor.clone();
            let project_path = project_path.to_owned();

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.unwrap();

                // Run the extractor
                Self::run_extractor(&extractor, &project_path).await
            });

            handles.push(handle);
        }

        // Wait for all extractors to complete
        let results = join_all(handles).await;

        // Process results
        let total_duration = start_time.elapsed();
        self.aggregate_results(results, total_duration).await
    }

    /// Execute a single extractor as a subprocess
    async fn run_extractor(
        extractor: &ExtractorDefinition,
        project_path: &Path,
    ) -> Result<(ExtractorRun, Option<IntermediateRepresentation>), ExtractorError> {
        let start = Instant::now();

        info!("Iniciando extractor: {}", extractor.id);

        // Prepare input configuration
        let input_config = ExtractorConfig {
            project_path: project_path.to_owned(),
            config: extractor.config.clone(),
            file_filters: Default::default(),
        };

        // Execute command with timeout
        let timeout_duration = Duration::from_secs(extractor.timeout_seconds);

        let result = tokio::time::timeout(
            timeout_duration,
            Self::execute_extractor_command(&extractor.command, &input_config),
        )
        .await;

        let duration = start.elapsed();

        match result {
            Ok(Ok((stdout, stderr))) => {
                // Log stderr if present
                if !stderr.is_empty() {
                    let stderr_str = String::from_utf8_lossy(&stderr);
                    if !stderr_str.trim().is_empty() {
                        info!("Extractor {} stderr: {}", extractor.id, stderr_str);
                    }
                }

                eprintln!(
                    "[DEBUG] Extractor {} stdout length: {}",
                    extractor.id,
                    stdout.len()
                );
                eprintln!(
                    "[DEBUG] First 500 chars: {}",
                    String::from_utf8_lossy(&stdout)
                        .chars()
                        .take(500)
                        .collect::<String>()
                );

                // Parse IR from stdout
                match Self::parse_ir_from_bytes(&stdout) {
                    Ok(ir) => {
                        let fact_count = ir.facts.len();
                        info!(
                            "Extractor {} completado: {} hechos en {:?}",
                            extractor.id, fact_count, duration
                        );

                        Ok((
                            ExtractorRun {
                                id: extractor.id.clone(),
                                success: true,
                                duration,
                                facts_extracted: fact_count,
                                error: None,
                                metadata: None,
                            },
                            Some(ir),
                        ))
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to parse IR: {}", e);
                        error!("Extractor {} error: {}", extractor.id, error_msg);

                        Ok((
                            ExtractorRun {
                                id: extractor.id.clone(),
                                success: false,
                                duration,
                                facts_extracted: 0,
                                error: Some(error_msg),
                                metadata: None,
                            },
                            None,
                        ))
                    }
                }
            }
            Ok(Err(e)) => {
                let error_msg = e.to_string();
                error!("Extractor {} failed: {}", extractor.id, error_msg);

                Ok((
                    ExtractorRun {
                        id: extractor.id.clone(),
                        success: false,
                        duration,
                        facts_extracted: 0,
                        error: Some(error_msg),
                        metadata: None,
                    },
                    None,
                ))
            }
            Err(_elapsed) => {
                let error_msg = format!("Timeout exceeded: {:?}", timeout_duration);
                warn!("Extractor {} timeout: {}", extractor.id, error_msg);

                Ok((
                    ExtractorRun {
                        id: extractor.id.clone(),
                        success: false,
                        duration,
                        facts_extracted: 0,
                        error: Some(error_msg),
                        metadata: None,
                    },
                    None,
                ))
            }
        }
    }

    /// Execute the extractor command
    async fn execute_extractor_command(
        command: &str,
        input_config: &ExtractorConfig,
    ) -> Result<(Vec<u8>, Vec<u8>), ExtractorError> {
        // Parse command string into executable and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ExtractorError::ExecutionFailed {
                id: command.to_string(),
                exit_code: None,
                stderr: "Empty command".to_string(),
            });
        }

        let executable = parts[0];
        let args = &parts[1..];

        let mut child = Command::new(executable)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExtractorError::SpawnFailed {
                id: command.to_string(),
                error: e.to_string(),
            })?;

        // Write config to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let config_json =
                serde_json::to_vec(input_config).map_err(|e| ExtractorError::Json {
                    id: command.to_string(),
                    error: e,
                })?;

            stdin
                .write_all(&config_json)
                .await
                .map_err(|e| ExtractorError::Io {
                    id: command.to_string(),
                    error: e,
                })?;

            // Close stdin to signal EOF
            drop(stdin);
        }

        // Wait for process to complete
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| ExtractorError::Io {
                id: command.to_string(),
                error: e,
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExtractorError::ExecutionFailed {
                id: command.to_string(),
                exit_code: output.status.code(),
                stderr: stderr.to_string(),
            });
        }

        Ok((output.stdout, output.stderr))
    }

    /// Parse intermediate representation from bytes
    ///
    /// This attempts to deserialize JSON IR. In the future, this could also
    /// support Cap'n Proto binary format.
    fn parse_ir_from_bytes(bytes: &[u8]) -> Result<IntermediateRepresentation, ExtractorError> {
        serde_json::from_slice(bytes).map_err(|e| ExtractorError::InvalidIR {
            id: "unknown".to_string(),
            error: e.to_string(),
        })
    }

    /// Aggregate results from all extractors
    async fn aggregate_results(
        &self,
        results: Vec<
            Result<
                Result<(ExtractorRun, Option<IntermediateRepresentation>), ExtractorError>,
                tokio::task::JoinError,
            >,
        >,
        total_duration: Duration,
    ) -> Result<AggregatedIR, ExtractorError> {
        let mut all_facts = Vec::new();
        let mut extractor_runs = Vec::new();

        for result in results {
            match result {
                Ok(Ok((run, ir_opt))) => {
                    extractor_runs.push(run);
                    if let Some(ir) = ir_opt {
                        all_facts.extend(ir.facts);
                    }
                }
                Ok(Err(e)) => {
                    error!("Extractor error: {}", e);
                    // Still record the run as failed
                    extractor_runs.push(ExtractorRun {
                        id: "unknown".to_string(),
                        success: false,
                        duration: Duration::from_secs(0),
                        facts_extracted: 0,
                        error: Some(e.to_string()),
                        metadata: None,
                    });
                }
                Err(join_err) => {
                    error!("Task join error: {}", join_err);
                }
            }
        }

        // Check that at least one extractor succeeded
        let successful_count = extractor_runs.iter().filter(|r| r.success).count();

        if successful_count == 0 {
            return Err(ExtractorError::AllExtractorsFailed);
        }

        let total_before = all_facts.len();

        info!(
            "Agregando {} hechos de {} extractores exitosos",
            total_before, successful_count
        );

        // TODO: Implement deduplication (US-14.7)
        // For now, we keep all facts
        let deduplicated_facts = all_facts;
        let total_after = deduplicated_facts.len();

        let dedup_ratio = if total_before > 0 {
            1.0 - (total_after as f64 / total_before as f64)
        } else {
            0.0
        };

        if dedup_ratio > 0.0 {
            info!(
                "Deduplicación: {} → {} hechos ({:.1}% reducción)",
                total_before,
                total_after,
                dedup_ratio * 100.0
            );
        }

        Ok(AggregatedIR {
            facts: deduplicated_facts,
            metadata: AggregationMetadata {
                extractor_runs,
                total_facts_before_dedup: total_before,
                total_facts_after_dedup: total_after,
                deduplication_ratio: dedup_ratio,
                total_duration,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_config_defaults() {
        let config = OrchestratorConfig::default();
        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_extractors, 4);
        assert_eq!(config.global_timeout_seconds, 1800);
    }

    #[test]
    fn test_orchestrator_creation() {
        let config = OrchestratorConfig::default();
        let extractors = vec![ExtractorDefinition {
            id: "test".to_string(),
            command: "test-cmd".to_string(),
            enabled: true,
            timeout_seconds: 300,
            config: serde_json::json!({}),
        }];

        let orchestrator = ExtractorOrchestrator::new(config, extractors);
        assert_eq!(orchestrator.extractors.len(), 1);
    }

    // Note: Full integration tests with actual subprocess execution
    // will be in the tests/ directory with mock extractors
}
