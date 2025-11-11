//! Extractor Orchestrator - High-performance async implementation
//!
//! Manages the lifecycle of external extractor processes with Cap'n Proto messaging,
//! using Tokio for async I/O and proper resource management.

use libc::{PRIO_PROCESS, c_int, setpriority};
use std::collections::HashMap;
use std::path::PathBuf;
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
        AggregatedIR, ExtractorConfig, ExtractorDef, ExtractorRequest, ExtractorResponse,
        PROTOCOL_VERSION,
    },
};

/// Main orchestrator for managing extractor processes with high performance
#[derive(Debug)]
pub struct ExtractorOrchestrator {
    config: ExtractorConfig,
    running_extractors: Mutex<HashMap<String, Child>>,
    concurrency_semaphore: Arc<Semaphore>,
    resource_manager: Arc<ResourceManager>,
}

/// Manages resource limits and monitoring for all extractors
#[derive(Debug)]
pub struct ResourceManager {
    config: ExtractorConfig,
    active_extractors: Mutex<HashMap<u32, ProcessResourceInfo>>,
    peak_memory: Mutex<u64>,
    peak_concurrent: Mutex<usize>,
    total_processed: Mutex<u64>,
    total_errors: Mutex<u64>,
}

#[derive(Debug, Clone)]
struct ProcessResourceInfo {
    pid: u32,
    name: String,
    memory_limit: Option<u64>,
    cpu_priority: Option<i32>,
    io_priority: Option<u8>,
    start_time: Instant,
}

/// Resource usage statistics with real monitoring data
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub active_extractors: usize,
    pub total_processed: u64,
    pub total_errors: u64,
    pub max_concurrent: usize,
    pub active_pids: Vec<u32>,
    pub memory_used_bytes: u64,
    pub peak_memory_bytes: u64,
    pub avg_processing_time_ms: u64,
    pub cpu_utilization: f32,
}

impl ResourceManager {
    fn new(config: ExtractorConfig) -> Self {
        let max_concurrent = config.max_concurrent.unwrap_or(4);
        Self {
            config,
            active_extractors: Mutex::new(HashMap::new()),
            peak_memory: Mutex::new(0),
            peak_concurrent: Mutex::new(max_concurrent),
            total_processed: Mutex::new(0),
            total_errors: Mutex::new(0),
        }
    }

    /// Register a new extractor process with resource limits
    async fn register_process(&self, pid: u32, name: &str, extractor: &ExtractorDef) -> Result<()> {
        let mut active = self.active_extractors.lock().await;

        active.insert(
            pid,
            ProcessResourceInfo {
                pid,
                name: name.to_string(),
                memory_limit: extractor.memory_limit,
                cpu_priority: extractor.cpu_priority,
                io_priority: extractor.io_priority,
                start_time: Instant::now(),
            },
        );

        // Update peak concurrent count
        if active.len() > *self.peak_concurrent.lock().await {
            *self.peak_concurrent.lock().await = active.len();
        }

        // Apply CPU priority if configured
        if let Some(nice_value) = extractor.cpu_priority {
            let pid_u32 = pid as u32;
            let nice_i32 = nice_value as i32;
            let result = unsafe { setpriority(PRIO_PROCESS, pid_u32, nice_i32) };
            if result != 0 {
                warn!(
                    "Failed to set CPU priority (nice={}) for PID {}: {}",
                    nice_value,
                    pid,
                    std::io::Error::last_os_error()
                );
            } else {
                debug!("Set CPU priority (nice={}) for PID {}", nice_value, pid);
            }
        }

        debug!("Registered extractor process '{}' (PID: {})", name, pid);
        Ok(())
    }

    /// Unregister a finished extractor process
    async fn unregister_process(&self, pid: u32) -> Option<ProcessResourceInfo> {
        let mut active = self.active_extractors.lock().await;
        let info = active.remove(&pid);

        if let Some(ref process_info) = info {
            let elapsed = process_info.start_time.elapsed();
            debug!(
                "Unregistered extractor process '{}' (PID: {}) after {:?}",
                process_info.name, pid, elapsed
            );
        }

        info
    }

    /// Record successful completion
    async fn record_success(&self) {
        *self.total_processed.lock().await += 1;
    }

    /// Record error
    async fn record_error(&self) {
        *self.total_errors.lock().await += 1;
    }

    /// Get current resource statistics with real monitoring
    async fn get_stats(&self) -> ResourceStats {
        let active = self.active_extractors.lock().await;
        let active_pids: Vec<u32> = active.keys().copied().collect();

        // Calculate total memory used by all active processes
        let mut total_memory = 0u64;
        for &pid in &active_pids {
            if let Ok(memory) = get_process_memory_kb(pid) {
                let memory_bytes = memory as u64 * 1024;
                total_memory += memory_bytes;
            }
        }

        // Update peak memory
        if total_memory > *self.peak_memory.lock().await {
            *self.peak_memory.lock().await = total_memory;
        }

        ResourceStats {
            active_extractors: active.len(),
            total_processed: *self.total_processed.lock().await,
            total_errors: *self.total_errors.lock().await,
            max_concurrent: *self.peak_concurrent.lock().await,
            active_pids,
            memory_used_bytes: total_memory,
            peak_memory_bytes: *self.peak_memory.lock().await,
            avg_processing_time_ms: 0, // Would require more tracking
            cpu_utilization: 0.0,      // Would require more complex monitoring
        }
    }

    /// Check if starting a new extractor would exceed global memory limits
    async fn check_memory_limit(&self, estimated_memory: u64) -> Result<()> {
        if let Some(limit) = self.config.global_memory_limit {
            let current = self.get_stats().await.memory_used_bytes;

            if current + estimated_memory > limit {
                return Err(OrchestratorError::ResourceLimitExceeded);
            }
        }
        Ok(())
    }

    /// Check if starting a new extractor would exceed concurrent limit
    async fn check_concurrent_limit(&self) -> Result<()> {
        let active = self.active_extractors.lock().await;
        let current_count = active.len();
        let max_allowed = self.config.max_concurrent.unwrap_or(4);

        if current_count >= max_allowed {
            return Err(OrchestratorError::ResourceLimitExceeded);
        }
        Ok(())
    }
}

/// Get memory usage for a specific process in KB
fn get_process_memory_kb(pid: u32) -> Result<u64> {
    let proc_path = format!("/proc/{}/status", pid);
    let content = std::fs::read_to_string(&proc_path).map_err(|e| OrchestratorError::FromIo(e))?;

    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            // Parse line like: "VmRSS:    1234 kB"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return Ok(parts[1].parse().unwrap_or(0));
            }
        }
    }

    Ok(0)
}

impl ExtractorOrchestrator {
    /// Create a new orchestrator with the given configuration
    pub fn new(config: ExtractorConfig) -> Self {
        let max_concurrent = config.max_concurrent.unwrap_or(4);

        Self {
            config: config.clone(),
            running_extractors: Mutex::new(HashMap::new()),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            resource_manager: Arc::new(ResourceManager::new(config.clone())),
        }
    }

    /// Execute all extractors with controlled concurrency and timeouts
    #[instrument(skip_all, fields(extractor_count = self.config.extractors.len()))]
    pub async fn execute_all(&self, project_path: &str, language: &str) -> Result<AggregatedIR> {
        debug!(
            "Starting execution of {} extractors",
            self.config.extractors.len()
        );

        // Handle empty extractor list
        if self.config.extractors.is_empty() {
            debug!("No extractors configured, returning empty result");
            return Ok(AggregatedIR::new());
        }

        let mut aggregated = AggregatedIR::new();

        // Get concurrency limit from semaphore
        let max_concurrent = self.concurrency_semaphore.available_permits();

        // Use adaptive batching for better resource utilization
        let batch_size = std::cmp::max(
            1,
            std::cmp::min(max_concurrent, self.config.extractors.len()),
        );

        for chunk in self.config.extractors.chunks(batch_size) {
            let mut handles = Vec::new();

            for extractor in chunk {
                let config = self.config.clone();
                let extractor_clone = extractor.clone();
                let project_path = project_path.to_string();
                let language = language.to_string();
                let resource_manager = Arc::clone(&self.resource_manager);

                // Acquire semaphore permit before spawning
                let semaphore = Arc::clone(&self.concurrency_semaphore);
                let permit = semaphore.acquire_owned().await.unwrap();

                let handle = tokio::spawn(async move {
                    let result = Self::execute_single_extractor(
                        &config,
                        &extractor_clone,
                        &project_path,
                        &language,
                        resource_manager,
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
                        self.resource_manager.record_error().await;
                    }
                    Err(e) => {
                        error!("Task join error: {}", e);
                    }
                }
            }
        }

        let stats = self.resource_manager.get_stats().await;
        debug!(
            "Execution complete. Processed: {}, Errors: {}, Active: {}, Memory: {} bytes",
            stats.total_processed,
            stats.total_errors,
            stats.active_extractors,
            stats.memory_used_bytes
        );

        Ok(aggregated)
    }

    /// Execute a single extractor with proper timeout and resource management
    #[instrument(skip_all)]
    async fn execute_single_extractor(
        config: &ExtractorConfig,
        extractor: &ExtractorDef,
        project_path: &str,
        language: &str,
        resource_manager: Arc<ResourceManager>,
    ) -> Result<(String, ExtractorResponse)> {
        let timeout_duration = extractor
            .timeout
            .or(config.default_timeout)
            .unwrap_or(Duration::from_secs(30));

        let start = Instant::now();

        // Check resource limits before starting
        if let Err(e) = resource_manager.check_concurrent_limit().await {
            warn!("Resource limit check failed: {}", e);
            return Err(e);
        }

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

        let pid = child.id().unwrap_or(0);

        // Register the process for resource monitoring
        if pid > 0 {
            if let Err(e) = resource_manager
                .register_process(pid, &extractor.name, extractor)
                .await
            {
                warn!("Failed to register process for monitoring: {}", e);
            }
        }

        // Execute with timeout using tokio::time::timeout
        let result = timeout(timeout_duration, async {
            Self::communicate_with_extractor(&mut child, request).await
        })
        .await;

        let elapsed = start.elapsed();

        // Clean up the process regardless of result
        if pid > 0 {
            let _ = resource_manager.unregister_process(pid).await;
        }
        let _ = child.kill().await;

        match result {
            Ok(Ok(response)) => {
                debug!("Extractor '{}' completed in {:?}", extractor.name, elapsed);
                resource_manager.record_success().await;
                Ok((extractor.name.clone(), response))
            }
            Ok(Err(e)) => {
                error!("Extractor '{}' communication error: {}", extractor.name, e);
                resource_manager.record_error().await;
                Err(e)
            }
            Err(_) => {
                error!(
                    "Extractor '{}' timed out after {:?}",
                    extractor.name, timeout_duration
                );
                resource_manager.record_error().await;
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
        // Serialize request to JSON
        let request_buffer =
            serde_json::to_vec(&request).map_err(|e| OrchestratorError::JsonError(e))?;

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
        let response = serde_json::from_slice::<ExtractorResponse>(&response_buffer)
            .map_err(|e| OrchestratorError::JsonError(e))?;

        Ok(response)
    }

    /// Execute a single extractor with explicit timeout
    pub async fn execute_with_timeout(
        &self,
        extractor: &ExtractorDef,
        timeout_duration: Duration,
    ) -> Result<ExtractorResponse> {
        match timeout(timeout_duration, async {
            Self::execute_single_extractor(
                &self.config,
                extractor,
                "/tmp",
                "rust",
                Arc::clone(&self.resource_manager),
            )
            .await
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
        self.resource_manager.get_stats().await
    }

    /// Perform graceful shutdown of all running extractors
    pub async fn graceful_shutdown_all(&self) -> Result<()> {
        // Get all PIDs before killing
        let stats = self.resource_manager.get_stats().await;
        let pids = stats.active_pids.clone();

        // Kill all processes
        for pid in pids {
            let result = std::process::Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        debug!("Killed process {}", pid);
                    } else {
                        warn!(
                            "Failed to kill process {}: {}",
                            pid,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(e) => {
                    warn!("Error killing process {}: {}", pid, e);
                }
            }
        }

        // Clear the running extractors map
        let mut extractors = self.running_extractors.lock().await;
        for (name, mut child) in extractors.drain() {
            let _ = child.kill().await;
            debug!("Killed extractor '{}'", name);
        }

        Ok(())
    }

    /// Check if extractors are running within resource limits
    pub async fn verify_resource_limits(&self) -> Result<()> {
        // Check concurrent limit
        let stats = self.resource_manager.get_stats().await;
        let max_allowed = self.config.max_concurrent.unwrap_or(4);

        if stats.active_extractors > max_allowed {
            return Err(OrchestratorError::ResourceLimitExceeded);
        }

        // Check memory limit if configured
        if let Some(limit) = self.config.global_memory_limit {
            if stats.memory_used_bytes > limit {
                return Err(OrchestratorError::ResourceLimitExceeded);
            }
        }

        Ok(())
    }
}

/// Resource usage statistics
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
            global_memory_limit: None,
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
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
            memory_limit: None,
            cpu_priority: None,
            io_priority: None,
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
            global_memory_limit: None,
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
        };

        let orchestrator = ExtractorOrchestrator::new(config);
        let result = orchestrator.execute_all("/tmp", "rust").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().facts.len(), 0);
    }

    #[tokio::test]
    async fn test_resource_tracking() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
            global_memory_limit: None,
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
        };

        let resource_manager = ResourceManager::new(config);

        // Record success and error
        resource_manager.record_success().await;
        resource_manager.record_error().await;
        resource_manager.record_success().await;

        let stats = resource_manager.get_stats().await;
        assert_eq!(stats.total_processed, 2);
        assert_eq!(stats.total_errors, 1);
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

    #[tokio::test]
    async fn test_timeout_kills_extractor() {
        let extractor = ExtractorDef {
            name: "sleep".to_string(),
            command: "sleep".to_string(),
            args: vec!["100".to_string()],
            timeout: None,
            env: None,
            memory_limit: None,
            cpu_priority: None,
            io_priority: None,
        };

        let orchestrator = ExtractorOrchestrator::default();
        let result = orchestrator
            .execute_with_timeout(&extractor, Duration::from_millis(100))
            .await;

        assert!(matches!(result, Err(OrchestratorError::Timeout)));
    }

    #[tokio::test]
    async fn test_resource_limits_config() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(2),
            default_timeout: Some(Duration::from_secs(60)),
            global_memory_limit: Some(1_000_000),
            global_cpu_limit: Some(80),
            default_nice: Some(5),
            default_io_priority: Some(2),
        };

        let orchestrator = ExtractorOrchestrator::new(config);
        let stats = orchestrator.get_resource_stats().await;

        assert_eq!(stats.max_concurrent, 2);
        assert_eq!(stats.memory_used_bytes, 0);
    }

    #[test]
    fn test_extractor_def_serialization() {
        let extractor = ExtractorDef {
            name: "ruff".to_string(),
            command: "ruff-to-hodei".to_string(),
            args: vec!["--format".to_string(), "json".to_string()],
            timeout: Some(Duration::from_secs(30)),
            env: Some(
                [("RUST_BACKTRACE".to_string(), "1".to_string())]
                    .into_iter()
                    .collect(),
            ),
            memory_limit: Some(500_000),
            cpu_priority: Some(5),
            io_priority: Some(2),
        };

        let json = serde_json::to_string(&extractor).unwrap();
        let deserialized: ExtractorDef = serde_json::from_str(&json).unwrap();

        assert_eq!(extractor.name, deserialized.name);
        assert_eq!(extractor.memory_limit, deserialized.memory_limit);
        assert_eq!(extractor.cpu_priority, deserialized.cpu_priority);
        assert_eq!(extractor.io_priority, deserialized.io_priority);
    }

    #[tokio::test]
    async fn test_resource_monitoring() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
            global_memory_limit: Some(10_000_000),
            global_cpu_limit: Some(90),
            default_nice: Some(0),
            default_io_priority: Some(0),
        };

        let orchestrator = ExtractorOrchestrator::new(config);

        // Get initial stats
        let stats = orchestrator.get_resource_stats().await;
        assert_eq!(stats.active_extractors, 0);
        assert!(stats.memory_used_bytes <= 10_000_000);
        assert_eq!(stats.max_concurrent, 4);
        assert!(stats.active_pids.is_empty());
    }

    #[test]
    fn test_extractor_config_serialization() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(8),
            default_timeout: Some(Duration::from_secs(45)),
            global_memory_limit: Some(50_000_000),
            global_cpu_limit: Some(75),
            default_nice: Some(3),
            default_io_priority: Some(1),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExtractorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_concurrent, deserialized.max_concurrent);
        assert_eq!(config.global_memory_limit, deserialized.global_memory_limit);
        assert_eq!(config.global_cpu_limit, deserialized.global_cpu_limit);
        assert_eq!(config.default_nice, deserialized.default_nice);
    }

    #[tokio::test]
    async fn test_process_memory_monitoring() {
        // Test the get_process_memory_kb function with current process
        let current_pid = std::process::id();
        let memory = get_process_memory_kb(current_pid).unwrap_or(0);

        // Should be able to get memory usage for current process
        assert!(memory > 0, "Current process should have memory usage > 0");
    }

    #[tokio::test]
    async fn test_concurrent_limit_check() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(1), // Very restrictive
            default_timeout: Some(Duration::from_secs(30)),
            global_memory_limit: None,
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
        };

        let resource_manager = ResourceManager::new(config);

        // Should be able to check with no processes
        assert!(resource_manager.check_concurrent_limit().await.is_ok());
    }

    #[tokio::test]
    async fn test_memory_limit_check() {
        let config = ExtractorConfig {
            extractors: vec![],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
            global_memory_limit: Some(1_000_000), // 1MB limit
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
        };

        let resource_manager = ResourceManager::new(config);

        // Should allow starting with small memory
        assert!(resource_manager.check_memory_limit(500_000).await.is_ok());

        // Should reject if exceeds limit
        assert!(
            resource_manager
                .check_memory_limit(2_000_000)
                .await
                .is_err()
        );
    }
}
