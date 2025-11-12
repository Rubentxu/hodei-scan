//! hodei-cli: Command-line interface
//!
//! This crate provides the CLI for hodei-scan.

pub mod analysis;

use clap::{Arg, Command};
use hodei_engine::{EngineConfig, RuleEngine};
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

    // TODO: Add actual extractor implementation
    println!("⚠️  Extractor not yet implemented");

    // Create rule engine
    let config = EngineConfig::default();
    let engine = RuleEngine::new(config);

    println!("🎉 Scan complete!");
    Ok(())
}
