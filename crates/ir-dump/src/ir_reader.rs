//! IR Reader
//!
//! Reads IR from various formats (JSON, YAML, Cap'n Proto)

use anyhow::{Context, Result};
use hodei_ir::FindingSet;
use serde_json;
use serde_yml;
use std::path::Path;

/// IR Reader implementation
pub struct IRReader;

impl IRReader {
    pub fn new() -> Self {
        Self
    }

    /// Read IR from file (detects format from extension)
    pub async fn read(&self, path: &Path) -> Result<FindingSet> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match extension {
            "json" => self.read_json(path).await,
            "yaml" | "yml" => self.read_yaml(path).await,
            "capnp" => self.read_capnp(path).await,
            _ => Err(anyhow::anyhow!("Unsupported format: {}", extension)),
        }
    }

    async fn read_json(&self, path: &Path) -> Result<FindingSet> {
        let content = tokio::fs::read_to_string(path).await?;

        // Try to parse as array first (old format)
        if let Ok(finding_set) = serde_json::from_str::<FindingSet>(&content) {
            return Ok(finding_set);
        }

        // Try to parse as object with schema (new format)
        #[derive(serde::Deserialize)]
        struct IRWrapper {
            facts: Option<FindingSet>,
            findings: Option<FindingSet>,
            #[allow(dead_code)]
            schema_version: Option<String>,
            #[allow(dead_code)]
            metadata: Option<serde_json::Value>,
        }

        let wrapper: IRWrapper = serde_json::from_str(&content)
            .context("Failed to parse IR as array or object format")?;

        wrapper.facts.or(wrapper.findings)
            .ok_or_else(|| anyhow::anyhow!("No facts or findings field found in IR object"))
    }

    async fn read_yaml(&self, path: &Path) -> Result<FindingSet> {
        let content = tokio::fs::read_to_string(path).await?;

        // Try to parse as array first (old format)
        if let Ok(finding_set) = serde_yml::from_str::<FindingSet>(&content) {
            return Ok(finding_set);
        }

        // Try to parse as object with schema (new format)
        #[derive(serde::Deserialize)]
        struct IRWrapper {
            facts: Option<FindingSet>,
            findings: Option<FindingSet>,
            #[allow(dead_code)]
            schema_version: Option<String>,
            #[allow(dead_code)]
            metadata: Option<serde_json::Value>,
        }

        let wrapper: IRWrapper = serde_yml::from_str(&content)
            .context("Failed to parse IR as array or object format")?;

        wrapper.facts.or(wrapper.findings)
            .ok_or_else(|| anyhow::anyhow!("No facts or findings field found in IR object"))
    }

    async fn read_capnp(&self, _path: &Path) -> Result<FindingSet> {
        // Cap'n Proto implementation would go here
        Err(anyhow::anyhow!("Cap'n Proto support not yet implemented"))
    }
}
