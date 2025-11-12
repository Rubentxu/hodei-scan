//! IR Formatter
//!
//! Formats IR data in various output formats

use hodei_ir::FindingSet;
use serde_json;
use serde_yml;

/// Output formats
#[derive(Clone)]
pub enum Format {
    Json,
    Yaml,
    Visual,
}

/// IR Formatter
pub struct IRFormatter;

impl IRFormatter {
    pub fn new() -> Self {
        Self
    }

    /// Format IR to string
    pub fn format(&self, ir: &FindingSet, format: &Format) -> Result<String, String> {
        match format {
            Format::Json => self.format_json(ir),
            Format::Yaml => self.format_yaml(ir),
            Format::Visual => self.format_visual(ir),
        }
    }

    fn format_json(&self, ir: &FindingSet) -> Result<String, String> {
        serde_json::to_string_pretty(&ir).map_err(|e| e.to_string())
    }

    fn format_yaml(&self, ir: &FindingSet) -> Result<String, String> {
        serde_yml::to_string(&ir).map_err(|e| e.to_string())
    }

    fn format_visual(&self, ir: &FindingSet) -> Result<String, String> {
        let mut output = String::new();
        output.push_str("IR Structure:\n");
        output.push_str(&"=".repeat(60));
        output.push('\n');
        output.push('\n');

        for (i, finding) in ir.iter().enumerate() {
            output.push_str(&format!("Finding #{}\n", i + 1));
            output.push_str(&"-".repeat(60));
            output.push('\n');
            output.push_str(&format!("Fact Type: {}\n", finding.fact_type));
            output.push_str(&format!("Message: {}\n", finding.message));
            output.push_str(&format!("Location: {}\n", finding.location));
            output.push('\n');
        }

        output.push('\n');
        output.push_str(&format!("Total findings: {}\n", ir.len()));

        Ok(output)
    }
}
