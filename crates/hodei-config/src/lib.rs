//! Configuration management for hodei-scan
//!
//! This module handles parsing and validation of hodei.toml configuration files

use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, info};

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Errors that can occur during configuration parsing or validation
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    #[error("Missing configuration file")]
    MissingFile,
}

/// Top-level configuration structure
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HodeiConfig {
    /// Extractor configuration
    pub extractors: ExtractorConfig,
}

/// Extractor configuration section
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtractorConfig {
    /// Whether extractors are enabled
    pub enabled: Option<bool>,

    /// Maximum number of extractors to run concurrently
    pub max_concurrent: Option<usize>,

    /// Default timeout for all extractors
    pub default_timeout: Option<String>,

    /// Global memory limit in bytes
    pub global_memory_limit: Option<u64>,

    /// Global CPU limit percentage (0-100)
    pub global_cpu_limit: Option<u8>,

    /// Default nice value for CPU priority (-20 to 19)
    pub default_nice: Option<i32>,

    /// Default I/O priority (0-7, where 0 is highest)
    pub default_io_priority: Option<u8>,

    /// Individual extractor definitions
    pub def: Option<Vec<ExtractorDef>>,
}

/// Definition of a single extractor
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtractorDef {
    /// Unique name for the extractor
    pub name: String,

    /// Command to execute
    pub command: String,

    /// Command-line arguments
    pub args: Option<Vec<String>>,

    /// Timeout for this specific extractor
    pub timeout: Option<String>,

    /// Environment variables
    pub env: Option<std::collections::HashMap<String, String>>,

    /// Memory limit for this extractor in bytes
    pub memory_limit: Option<u64>,

    /// CPU priority (nice value)
    pub cpu_priority: Option<i32>,

    /// I/O priority (0-7)
    pub io_priority: Option<u8>,

    /// Source tool name (e.g., "ruff", "semgrep")
    pub source: Option<String>,
}

/// Parse hodei.toml configuration file
pub fn parse_hodei_toml(path: &Path) -> Result<HodeiConfig> {
    debug!("Parsing configuration file: {}", path.display());

    if !path.exists() {
        error!("Configuration file not found: {}", path.display());
        return Err(ConfigError::MissingFile);
    }

    let content = std::fs::read_to_string(path)?;
    let config: HodeiConfig = toml::from_str(&content)?;

    validate_config(&config)?;

    info!("Successfully parsed configuration from {}", path.display());
    Ok(config)
}

/// Parse hodei.toml from string content
pub fn parse_hodei_toml_str(content: &str) -> Result<HodeiConfig> {
    debug!("Parsing configuration from string");

    let config: HodeiConfig = toml::from_str(content)?;
    validate_config(&config)?;

    Ok(config)
}

/// Merge configuration from global config with project-specific config
pub fn merge_config(global: &HodeiConfig, project: &HodeiConfig) -> HodeiConfig {
    let mut merged = project.clone();

    // Merge global settings if project doesn't have them
    if merged.extractors.enabled.is_none() {
        merged.extractors.enabled = global.extractors.enabled.clone();
    }

    if merged.extractors.max_concurrent.is_none() {
        merged.extractors.max_concurrent = global.extractors.max_concurrent.clone();
    }

    if merged.extractors.default_timeout.is_none() {
        merged.extractors.default_timeout = global.extractors.default_timeout.clone();
    }

    if merged.extractors.global_memory_limit.is_none() {
        merged.extractors.global_memory_limit = global.extractors.global_memory_limit.clone();
    }

    if merged.extractors.global_cpu_limit.is_none() {
        merged.extractors.global_cpu_limit = global.extractors.global_cpu_limit.clone();
    }

    if merged.extractors.default_nice.is_none() {
        merged.extractors.default_nice = global.extractors.default_nice.clone();
    }

    if merged.extractors.default_io_priority.is_none() {
        merged.extractors.default_io_priority = global.extractors.default_io_priority.clone();
    }

    // Merge extractors - project extractors override global ones with same name
    let mut merged_extractors = global.extractors.def.clone().unwrap_or_default();

    if let Some(project_extractors) = &project.extractors.def {
        for project_extractor in project_extractors {
            if let Some(global_idx) = merged_extractors
                .iter()
                .position(|e| e.name == project_extractor.name)
            {
                // Override global extractor with project-specific one
                merged_extractors[global_idx] = project_extractor.clone();
            } else {
                // Add new project-specific extractor
                merged_extractors.push(project_extractor.clone());
            }
        }
    }

    merged.extractors.def = Some(merged_extractors);

    merged
}

/// Convert HodeiConfig to ExtractorConfig for use with ExtractorOrchestrator
impl From<&HodeiConfig> for hodei_engine::extractor::protocol::ExtractorConfig {
    fn from(config: &HodeiConfig) -> Self {
        let extractors = config
            .extractors
            .def
            .as_ref()
            .map_or_else(Vec::new, |defs| {
                defs.iter()
                    .map(|def| hodei_engine::extractor::protocol::ExtractorDef {
                        name: def.name.clone(),
                        command: def.command.clone(),
                        args: def.args.clone().unwrap_or_default(),
                        timeout: def
                            .timeout
                            .as_ref()
                            .map(|t| parse_duration(t).unwrap_or_default()),
                        env: def.env.clone(),
                        memory_limit: def.memory_limit,
                        cpu_priority: def.cpu_priority,
                        io_priority: def.io_priority,
                    })
                    .collect()
            });

        let default_timeout = config
            .extractors
            .default_timeout
            .as_ref()
            .map(|t| parse_duration(t).unwrap_or_default());

        hodei_engine::extractor::protocol::ExtractorConfig {
            extractors,
            max_concurrent: config.extractors.max_concurrent,
            default_timeout,
            global_memory_limit: config.extractors.global_memory_limit,
            global_cpu_limit: config.extractors.global_cpu_limit,
            default_nice: config.extractors.default_nice,
            default_io_priority: config.extractors.default_io_priority,
        }
    }
}

/// Parse duration string (e.g., "30s", "5m", "1h") to tokio::time::Duration
fn parse_duration(s: &str) -> Option<tokio::time::Duration> {
    use std::time::Duration;

    if s.ends_with("ms") {
        let val: u64 = s[..s.len() - 2].parse().ok()?;
        Some(Duration::from_millis(val))
    } else if s.ends_with('s') {
        let val: u64 = s[..s.len() - 1].parse().ok()?;
        Some(Duration::from_secs(val))
    } else if s.ends_with('m') {
        let val: u64 = s[..s.len() - 1].parse().ok()?;
        Some(Duration::from_secs(val * 60))
    } else if s.ends_with('h') {
        let val: u64 = s[..s.len() - 1].parse().ok()?;
        Some(Duration::from_secs(val * 3600))
    } else {
        None
    }
}

/// Validate configuration structure
fn validate_config(config: &HodeiConfig) -> Result<()> {
    // Validate max_concurrent
    if let Some(max) = config.extractors.max_concurrent {
        if max == 0 {
            return Err(ConfigError::ValidationError(
                "max_concurrent must be greater than 0".to_string(),
            ));
        }
    }

    // Validate default_timeout if present
    if let Some(timeout) = &config.extractors.default_timeout {
        if parse_duration(timeout).is_none() {
            return Err(ConfigError::ValidationError(format!(
                "Invalid timeout format: {}",
                timeout
            )));
        }
    }

    // Validate default_nice range
    if let Some(nice) = config.extractors.default_nice {
        if nice < -20 || nice > 19 {
            return Err(ConfigError::ValidationError(
                "default_nice must be between -20 and 19".to_string(),
            ));
        }
    }

    // Validate default_io_priority range
    if let Some(priority) = config.extractors.default_io_priority {
        if priority > 7 {
            return Err(ConfigError::ValidationError(
                "default_io_priority must be between 0 and 7".to_string(),
            ));
        }
    }

    // Validate global_cpu_limit range
    if let Some(limit) = config.extractors.global_cpu_limit {
        if limit > 100 {
            return Err(ConfigError::ValidationError(
                "global_cpu_limit must be between 0 and 100".to_string(),
            ));
        }
    }

    // Validate individual extractors
    if let Some(extractors) = &config.extractors.def {
        for extractor in extractors {
            validate_extractor(extractor)?;
        }
    }

    Ok(())
}

/// Validate a single extractor definition
fn validate_extractor(extractor: &ExtractorDef) -> Result<()> {
    // Check that name is not empty
    if extractor.name.is_empty() {
        return Err(ConfigError::ValidationError(
            "Extractor name cannot be empty".to_string(),
        ));
    }

    // Check that command is not empty
    if extractor.command.is_empty() {
        return Err(ConfigError::ValidationError(format!(
            "Extractor '{}' command cannot be empty",
            extractor.name
        )));
    }

    // Validate timeout if present
    if let Some(timeout) = &extractor.timeout {
        if parse_duration(timeout).is_none() {
            return Err(ConfigError::ValidationError(format!(
                "Invalid timeout for extractor '{}': {}",
                extractor.name, timeout
            )));
        }
    }

    // Validate cpu_priority range
    if let Some(priority) = extractor.cpu_priority {
        if priority < -20 || priority > 19 {
            return Err(ConfigError::ValidationError(format!(
                "Extractor '{}' cpu_priority must be between -20 and 19",
                extractor.name
            )));
        }
    }

    // Validate io_priority range
    if let Some(priority) = extractor.io_priority {
        if priority > 7 {
            return Err(ConfigError::ValidationError(format!(
                "Extractor '{}' io_priority must be between 0 and 7",
                extractor.name
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_toml() {
        let toml = r#"
[extractors]
max_concurrent = 4

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        assert_eq!(config.extractors.max_concurrent, Some(4));
        assert_eq!(config.extractors.def.as_ref().unwrap().len(), 1);
        assert_eq!(config.extractors.def.as_ref().unwrap()[0].name, "ruff");
    }

    #[test]
    fn parse_toml_with_timeout() {
        let toml = r#"
[extractors]
default_timeout = "30s"

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
timeout = "60s"
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        assert_eq!(config.extractors.default_timeout, Some("30s".to_string()));
        assert_eq!(
            config.extractors.def.as_ref().unwrap()[0].timeout,
            Some("60s".to_string())
        );
    }

    #[test]
    fn parse_toml_with_multiple_extractors() {
        let toml = r#"
[extractors]
max_concurrent = 2

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"

[[extractors.def]]
name = "sarif"
command = "sarif-to-hodei"
source = "semgrep"
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        assert_eq!(config.extractors.max_concurrent, Some(2));
        assert_eq!(config.extractors.def.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn parse_toml_with_args() {
        let toml = r#"
[[extractors.def]]
name = "ruff"
command = "ruff"
args = ["--format", "json"]
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        let def = &config.extractors.def.as_ref().unwrap()[0];
        assert_eq!(def.args.as_ref().unwrap().len(), 2);
        assert_eq!(def.args.as_ref().unwrap()[0], "--format");
        assert_eq!(def.args.as_ref().unwrap()[1], "json");
    }

    #[test]
    fn parse_toml_with_env() {
        let toml = r#"
[[extractors.def]]
name = "ruff"
command = "ruff"
env = { RUST_BACKTRACE = "1", LOG_LEVEL = "debug" }
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        let def = &config.extractors.def.as_ref().unwrap()[0];
        let env = def.env.as_ref().unwrap();
        assert_eq!(env.get("RUST_BACKTRACE"), Some(&"1".to_string()));
        assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
    }

    #[test]
    fn parse_duration_formats() {
        assert_eq!(
            parse_duration("100ms"),
            Some(tokio::time::Duration::from_millis(100))
        );
        assert_eq!(
            parse_duration("5s"),
            Some(tokio::time::Duration::from_secs(5))
        );
        assert_eq!(
            parse_duration("2m"),
            Some(tokio::time::Duration::from_secs(120))
        );
        assert_eq!(
            parse_duration("1h"),
            Some(tokio::time::Duration::from_secs(3600))
        );
        assert_eq!(parse_duration("invalid"), None);
    }

    #[test]
    fn validate_config_with_invalid_max_concurrent() {
        let toml = r#"
[extractors]
max_concurrent = 0
"#;

        let config = parse_hodei_toml_str(toml);
        assert!(matches!(config, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn validate_config_with_invalid_nice() {
        let toml = r#"
[extractors]
default_nice = 25

[[extractors.def]]
name = "ruff"
command = "ruff"
"#;

        let config = parse_hodei_toml_str(toml);
        assert!(matches!(config, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn validate_extractor_with_empty_name() {
        let toml = r#"
[[extractors.def]]
name = ""
command = "ruff"
"#;

        let config = parse_hodei_toml_str(toml);
        assert!(matches!(config, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn validate_extractor_with_empty_command() {
        let toml = r#"
[[extractors.def]]
name = "ruff"
command = ""
"#;

        let config = parse_hodei_toml_str(toml);
        assert!(matches!(config, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn validate_extractor_with_invalid_timeout() {
        let toml = r#"
[[extractors.def]]
name = "ruff"
command = "ruff"
timeout = "invalid"
"#;

        let config = parse_hodei_toml_str(toml);
        assert!(matches!(config, Err(ConfigError::ValidationError(_))));
    }

    #[test]
    fn convert_to_extractor_config() {
        let toml = r#"
[extractors]
max_concurrent = 2
default_timeout = "30s"

[[extractors.def]]
name = "ruff"
command = "ruff"
args = ["--format", "json"]
timeout = "60s"
"#;

        let config = parse_hodei_toml_str(toml).unwrap();
        let orchestrator_config: hodei_engine::extractor::protocol::ExtractorConfig =
            (&config).into();

        assert_eq!(orchestrator_config.max_concurrent, Some(2));
        assert!(orchestrator_config.default_timeout.is_some());
        assert_eq!(orchestrator_config.extractors.len(), 1);
        assert_eq!(orchestrator_config.extractors[0].name, "ruff");
    }

    #[test]
    fn merge_config_inheritance() {
        let global_toml = r#"
[extractors]
max_concurrent = 4
default_timeout = "30s"
default_nice = 5

[[extractors.def]]
name = "ruff"
command = "ruff-global"
"#;

        let project_toml = r#"
[extractors]
max_concurrent = 2

[[extractors.def]]
name = "sarif"
command = "sarif-local"
"#;

        let global_config = parse_hodei_toml_str(global_toml).unwrap();
        let project_config = parse_hodei_toml_str(project_toml).unwrap();

        let merged = merge_config(&global_config, &project_config);

        // Project settings override global
        assert_eq!(merged.extractors.max_concurrent, Some(2));

        // Global settings inherit when project doesn't specify
        assert_eq!(merged.extractors.default_timeout, Some("30s".to_string()));
        assert_eq!(merged.extractors.default_nice, Some(5));

        // Both extractors present
        let extractors = merged.extractors.def.as_ref().unwrap();
        assert_eq!(extractors.len(), 2);

        // Sarif extractor from project
        let sarif = extractors.iter().find(|e| e.name == "sarif").unwrap();
        assert_eq!(sarif.command, "sarif-local");

        // Ruff extractor from global
        let ruff = extractors.iter().find(|e| e.name == "ruff").unwrap();
        assert_eq!(ruff.command, "ruff-global");
    }

    #[test]
    fn merge_config_override() {
        let global_toml = r#"
[[extractors.def]]
name = "ruff"
command = "ruff-global"
timeout = "60s"
"#;

        let project_toml = r#"
[[extractors.def]]
name = "ruff"
command = "ruff-local"
timeout = "30s"
"#;

        let global_config = parse_hodei_toml_str(global_toml).unwrap();
        let project_config = parse_hodei_toml_str(project_toml).unwrap();

        let merged = merge_config(&global_config, &project_config);

        let extractors = merged.extractors.def.as_ref().unwrap();
        assert_eq!(extractors.len(), 1);

        // Project overrides global extractor
        let ruff = &extractors[0];
        assert_eq!(ruff.name, "ruff");
        assert_eq!(ruff.command, "ruff-local");
        assert_eq!(ruff.timeout, Some("30s".to_string()));
    }

    #[test]
    fn merge_config_empty_project() {
        let global_toml = r#"
[extractors]
max_concurrent = 4

[[extractors.def]]
name = "ruff"
command = "ruff"
"#;

        let project_toml = r#"
[extractors]
enabled = false
"#;

        let global_config = parse_hodei_toml_str(global_toml).unwrap();
        let project_config = parse_hodei_toml_str(project_toml).unwrap();

        let merged = merge_config(&global_config, &project_config);

        // Project enabled setting
        assert_eq!(merged.extractors.enabled, Some(false));

        // Global settings inherit
        assert_eq!(merged.extractors.max_concurrent, Some(4));

        // Global extractor still present
        let extractors = merged.extractors.def.as_ref().unwrap();
        assert_eq!(extractors.len(), 1);
        assert_eq!(extractors[0].name, "ruff");
    }
}
