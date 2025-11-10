//! hodei-cli: CLI tool for hodei-scan
//!
//! This crate provides the command-line interface for running code analysis
//! with hodei-scan.

#![warn(missing_docs)]

use clap::Parser;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(name = "hodei")]
#[command(about = "Advanced Static Code Analysis with IR Architecture")]
struct Args {
    /// Project path to analyze
    #[arg(short, long)]
    project: Option<String>,

    /// Rules file path
    #[arg(short, long)]
    rules: Option<String>,
}

fn main() {
    let args = Args::parse();

    println!("hodei-scan v3.0.0");
    println!("Project: {:?}", args.project);
    println!("Rules: {:?}", args.rules);
    println!("CLI initialized successfully");
}
