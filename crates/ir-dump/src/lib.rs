//! ir-dump
//!
//! IR debug tool for hodei-scan
//!
//! Features:
//! - Dump IR in JSON/YAML/Visual format
//! - Filter IR by fact type
//! - Interactive REPL explorer

pub mod cli;
pub mod interactive_explorer;
pub mod ir_formatter;
pub mod ir_reader;

pub use clap::ValueEnum;
pub use cli::run_cli;

/// Re-export types for convenience
pub use hodei_ir::{Fact, Finding, FindingSet, IntermediateRepresentation};
pub use interactive_explorer::InteractiveExplorer;
pub use ir_formatter::IRFormatter;
pub use ir_reader::IRReader;

/// Format types (CLI version)
#[derive(Debug, Clone, ValueEnum)]
pub enum Format {
    Json,
    Yaml,
    Visual,
}

impl From<Format> for ir_formatter::Format {
    fn from(format: Format) -> Self {
        match format {
            Format::Json => ir_formatter::Format::Json,
            Format::Yaml => ir_formatter::Format::Yaml,
            Format::Visual => ir_formatter::Format::Visual,
        }
    }
}
