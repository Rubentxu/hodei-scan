//! YAML parser adapter
//!
//! Parses test configuration files in YAML format using serde_yml

use crate::domain::models::TestConfig;
use crate::domain::ports::TestConfigParser;
use std::path::Path;
use anyhow::Result;
use tokio::fs;

/// YAML-based test config parser
pub struct YamlTestConfigParser;

impl YamlTestConfigParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl TestConfigParser for YamlTestConfigParser {
    async fn parse_file(&self, path: &Path) -> Result<TestConfig> {
        let content = fs::read_to_string(path).await?;
        
        let config: TestConfig = serde_yml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML: {}", e))?;
        
        Ok(config)
    }
}
