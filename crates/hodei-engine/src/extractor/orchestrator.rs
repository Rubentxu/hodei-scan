//! Extractor Orchestrator - High-performance async implementation
//!
//! Manages the lifecycle of external extractor processes with Cap'n Proto messaging,
//! using Tokio for async I/O and proper resource management.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command as TokioCommand};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::{Duration, Instant, timeout};
use tracing::{debug, error, instrument, warn};

use super::{
    Result,
    error::OrchestratorError,
    protocol::{
        AggregatedIR, ErrorResponse, ExtractorConfig, ExtractorDef, ExtractorMessage,
        ExtractorRequest, ExtractorResponse, PROTOCOL_VERSION,
    },
};

/// Main orchestrator for managing extractor processes with high performance
#[derive(Debug)]
pub struct ExtractorOrchestrator {
    config: ExtractorConfig,
    running_extractors: Mutex<HashMap<String, Child>>,
    concurrency_semaphore: Arc<Semaphore>,
    resource_tracker: Mutex<ResourceTracker>,
}

/// Tracks resource usage across all extractors
#[derive(Debug, Default)]
struct ResourceTracker {
    active_count: usize,
    total_processed: u64,
    total_errors: u64,
}

impl ResourceTracker {
    fn record_success(&mut self) {
        self.active_count = self.active_count.saturating_sub(1);
        self.total_processed = self.total_processed.wrapping_add(1);
    }

    fn record_error(&mut self) {
        self.active_count = self.active_count.saturating_sub(1);
        self.total_errors = self.total_errors.wrapping_add(1);
    }

    fn record_start(&mut self) {
        self.active_count = self.active_count.wrapping_add(1);
    }
}

impl ExtractorOrchestrator {
    /// Create a new orchestrator with the given configuration
    pub fn new(config: ExtractorConfig) -> Self {
        let max_concurrent = config.max_concurrent.unwrap_or(4);
        Self {
            config,
            running_extractors: Mutex::new(HashMap::new()),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            resource_tracker: Mutex::new(ResourceTracker::default()),
        }
    }

    /// Execute all extractors with controlled concurrency and timeouts
    #[instrument(skip_all, fields(extractor_count = self.config.extractors.len()))]
    pub async fn execute_all(&self, project_path: &str, language: &str) -> Result<AggregatedIR> {
        debug!(
            "Starting execution of {} extractors",
            self.config.extractors.len()
        );

        let mut aggregated = AggregatedIR::new();

        // Get concurrency limit from semaphore
        let max_concurrent = self.concurrency_semaphore.available_permits();

        // Use adaptive batching for better resource utilization
        let batch_size = std::cmp::min(max_concurrent, self.config.extractors.len());

        for chunk in self.config.extractors.chunks(batch_size) {
            let mut handles = Vec::new();

            for extractor in chunk {
                let extractor_clone = extractor.clone();
                let project_path = project_path.to_string();
                let language = language.to_string();

                // Acquire semaphore permit before spawning
                let semaphore = Arc::clone(&self.concurrency_semaphore);
                let permit = semaphore.acquire_owned().await.unwrap();

                let handle = tokio::spawn(async move {
                    let result = ExtractorOrchestrator::execute_single_extractor(
                        &extractor_clone,
                        &project_path,
                        &language,
                    )
                    .await;
                    // Drop permit when task completes
                    drop(permit);
                    result
                });

                handles.push(handle);
            }

            // Wait for all extractors in this batch to complete
            for handle in handles {
                match handle.await {
                    Ok(Ok((extractor_name, response))) => {
                        if let Err(e) = aggregated.add_extractor_results(&extractor_name, &response)
                        {
                            warn!("Failed to add extractor results: {}", e);
                        }
                        debug!("Extractor '{}' completed successfully", extractor_name);
                    }
                    Ok(Err(e)) => {
                        error!("Extractor execution failed: {}", e);
                        let mut tracker = self.resource_tracker.lock().await;
                        tracker.record_error();
                    }
                    Err(e) => {
                        error!("Task join error: {}", e);
                    }
                }
            }
        }

        let tracker = self.resource_tracker.lock().await;
        debug!(
            "Execution complete. Processed: {}, Errors: {}, Active: {}",
            tracker.total_processed, tracker.total_errors, tracker.active_count
        );

        Ok(aggregated)
    }

    /// Execute a single extractor with proper timeout and resource management
    #[instrument(skip_all)]
    async fn execute_single_extractor(
        extractor: &ExtractorDef,
        project_path: &str,
        language: &str,
    ) -> Result<(String, ExtractorResponse)> {
        let timeout_duration = extractor
            .timeout
            .or(Some(Duration::from_secs(30)))
            .unwrap_or(Duration::from_secs(30));

        let start = Instant::now();

        // Build request with unique ID for tracking
        let request_id = generate_request_id();

        let request = ExtractorRequest {
            request_id,
            project_path: project_path.to_string(),
            language: language.to_string(),
            config: "{}".to_string(),
            timeout_ms: timeout_duration.as_millis() as u32,
            version: PROTOCOL_VERSION.to_string(),
        };

        // Spawn the extractor process with async I/O
        let mut child = TokioCommand::new(&extractor.command)
            .args(&extractor.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| OrchestratorError::SpawnFailed(e.to_string()))?;

        // Execute with timeout using tokio::time::timeout
        let result = timeout(timeout_duration, async {
            Self::communicate_with_extractor(&mut child, request).await
        })
        .await;

        let elapsed = start.elapsed();

        // Clean up the process regardless of result
        let _ = child.kill().await;

        match result {
            Ok(Ok(response)) => {
                debug!("Extractor '{}' completed in {:?}", extractor.name, elapsed);
                Ok((extractor.name.clone(), response))
            }
            Ok(Err(e)) => {
                error!("Extractor '{}' communication error: {}", extractor.name, e);
                Err(e)
            }
            Err(_) => {
                error!(
                    "Extractor '{}' timed out after {:?}",
                    extractor.name, timeout_duration
                );
                Err(OrchestratorError::Timeout)
            }
        }
    }

    /// Communicate with extractor via stdin/stdout using efficient binary protocol
    #[instrument(skip_all)]
    async fn communicate_with_extractor(
        child: &mut Child,
        request: ExtractorRequest,
    ) -> Result<ExtractorResponse> {
        // Serialize request to efficient binary format (JSON for now, Cap'n Proto in future)
        let request_buffer = request
            .to_json()
            .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

        // Send request via async stdin with proper error handling
        let mut stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| OrchestratorError::Io("No stdin available".to_string()))?;

        // Use write_all for atomic writes
        stdin
            .write_all(&request_buffer)
            .await
            .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;

        stdin
            .flush()
            .await
            .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;

        // Read response via async stdout with streaming for large messages
        let mut stdout = child
            .stdout
            .as_mut()
            .ok_or_else(|| OrchestratorError::Io("No stdout available".to_string()))?;

        let mut response_buffer = Vec::new();
        stdout
            .read_to_end(&mut response_buffer)
            .await
            .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;

        if response_buffer.is_empty() {
            return Err(OrchestratorError::ProtoError(
                "Empty response from extractor".to_string(),
            ));
        }

        // Deserialize response
        let response_message = ExtractorMessage::deserialize(&response_buffer)
            .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

        match response_message {
            ExtractorMessage::Response(response) => Ok(response),
            ExtractorMessage::Error(error) => {
                Err(OrchestratorError::AggregatorError(error.error_message))
            }
            _ => Err(OrchestratorError::ProtoError(
                "Unexpected message type".to_string(),
            )),
        }
    }

    /// Execute a single extractor with explicit timeout
    pub async fn execute_with_timeout(
        &self,
        extractor: &ExtractorDef,
        timeout_duration: Duration,
    ) -> Result<ExtractorResponse> {
        match timeout(timeout_duration, async {
            Self::execute_single_extractor(extractor, "/tmp", "rust").await
        })
        .await
        {
            Ok(Ok((_name, response))) => Ok(response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(OrchestratorError::Timeout),
        }
    }

    /// Get current resource usage statistics
    pub async fn get_resource_stats(&self) -> ResourceStats {
        let tracker = self.resource_tracker.lock().await;
        ResourceStats {
            active_extractors: tracker.active_count,
            total_processed: tracker.total_processed,
            total_errors: tracker.total_errors,
            max_concurrent: self.concurrency_semaphore.available_permits(),
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub active_extractors: usize,
    pub total_processed: u64,
    pub total_errors: u64,
    pub max_concurrent: usize,
}

/// Generate a unique request ID using high-resolution time
fn generate_request_id() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

impl Default for ExtractorOrchestrator {
    fn default() -> Self {
        Self::new(ExtractorConfig {
            extractors: Vec::new(),
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let extractor = ExtractorDef {
            name: "sleep".to_string(),
            command: "sleep".to_string(),
            args: vec!["100".to_string()],
            timeout: None,
            env: None,
        };

        let orchestrator = ExtractorOrchestrator::default();
        let result = orchestrator
            .execute_with_timeout(&extractor, Duration::from_millis(100))
            .await;

        assert!(matches!(result, Err(OrchestratorError::Timeout)));
    }

    #[tokio::test]
    async fn test_execute_all_with_empty_config() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
        };

        let orchestrator = ExtractorOrchestrator::new(config);
        let result = orchestrator.execute_all("/tmp", "rust").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().facts.len(), 0);
    }

    #[tokio::test]
    async fn test_resource_tracking() {
        let mut tracker = ResourceTracker::default();
        assert_eq!(tracker.active_count, 0);

        tracker.record_start();
        assert_eq!(tracker.active_count, 1);

        tracker.record_success();
        assert_eq!(tracker.active_count, 0);
        assert_eq!(tracker.total_processed, 1);
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        std::thread::sleep(Duration::from_millis(1));
        let id2 = generate_request_id();

        assert!(id2 > id1);
    }

    #[test]
    fn test_aggregated_ir() {
        let mut aggregated = AggregatedIR::new();

        let response = ExtractorResponse {
            request_id: 123,
            success: true,
            ir: vec![],
            metadata: r#"{"version": "1.0"}"#.to_string(),
            processing_time_ms: 100,
        };

        assert!(aggregated.add_extractor_results("test", &response).is_ok());
        assert_eq!(aggregated.extractor_status.get("test"), Some(&true));
    }
}
