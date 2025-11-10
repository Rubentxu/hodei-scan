//! hodei-cli: Command-line interface
//!
//! This crate provides the CLI for hodei-scan.

use clap::{Arg, Command};
use hodei_engine::{EngineConfig, RuleEngine};
use hodei_extractors::{Extractor, RegexExtractor};
use hodei_ir::{Confidence, ExtractorId, FactType, Severity};
use std::path::Path;

/// Main CLI entry point
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("hodei-scan")
        .version("0.1.0")
        .about("Advanced Static Code Analysis with IR Architecture")
        .subcommand(
            Command::new("scan")
                .about("Scan a directory")
                .arg(Arg::new("path").required(true).help("Path to scan")),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("scan", sub_matches)) => {
            let path = sub_matches.get_one::<String>("path").unwrap();
            scan_path(Path::new(path))?;
        }
        _ => {
            eprintln!("No subcommand specified. Use 'hodei-scan scan --help' for help.");
        }
    }

    Ok(())
}

/// Scan a path
fn scan_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning path: {:?}", path);

    // Create a simple extractor
    let extractor = RegexExtractor::new(
        ExtractorId::Custom,
        "1.0.0",
        vec![(
            "TODO".to_string(),
            FactType::CodeSmell {
                smell_type: "TODO".to_string(),
                severity: Severity::Minor,
                message: "TODO comment found".to_string(),
            },
        )],
    );

    // Extract facts
    let facts = extractor.extract(Path::new(path))?;
    println!("✅ Extracted {} facts", facts.len());

    // Create rule engine
    let config = EngineConfig::default();
    let engine = RuleEngine::new(config);

    println!("🎉 Scan complete!");
    Ok(())
}
