//! CLI interface for ir-dump

use crate::{Format, IRFormatter, IRReader, InteractiveExplorer};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// CLI arguments
#[derive(Parser)]
#[command(name = "ir-dump")]
#[command(about = "IR debug tool for hodei-scan")]
struct Cli {
    /// Input IR file
    #[arg(short, long)]
    input: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "visual")]
    format: Format,

    /// Filter expression (e.g., "type=Vulnerability")
    #[arg(short, long)]
    filter: Option<String>,

    /// Enable interactive mode
    #[arg(short, long)]
    interactive: bool,

    /// Compare two IR files
    #[arg(short = '1', long)]
    input1: Option<PathBuf>,

    /// Second IR file for comparison
    #[arg(short = '2', long)]
    input2: Option<PathBuf>,
}

/// Run the CLI
pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let reader = IRReader::new();
    let formatter = IRFormatter::new();

    if let (Some(path1), Some(path2)) = (cli.input1, cli.input2) {
        // Compare mode
        let ir1 = reader.read(&path1).await?;
        let ir2 = reader.read(&path2).await?;

        println!("Comparing IR files:");
        println!("  File 1: {} ({} findings)", path1.display(), ir1.len());
        println!("  File 2: {} ({} findings)", path2.display(), ir2.len());

        // Simple diff
        let diff_count = (ir1.len() as i32 - ir2.len() as i32).abs();
        println!("\nDifference: {} findings", diff_count);

        return Ok(());
    }

    // Read IR file
    let ir = reader.read(&cli.input).await?;

    if cli.interactive {
        // Interactive mode
        let mut explorer = InteractiveExplorer::new(ir);
        explorer.start().await?;
    } else {
        // Dump mode
        let output = match cli.format {
            Format::Json => formatter.format(&ir, &crate::ir_formatter::Format::Json),
            Format::Yaml => formatter.format(&ir, &crate::ir_formatter::Format::Yaml),
            Format::Visual => formatter.format(&ir, &crate::ir_formatter::Format::Visual),
        }
        .map_err(|e| anyhow::anyhow!("Format error: {}", e))?;

        println!("{}", output);
    }

    Ok(())
}
