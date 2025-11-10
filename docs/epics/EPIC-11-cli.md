# EPIC-11: CLI (Command-Line Interface)

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02..10 (todos los anteriores)  
**Owner**: CLI Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar CLI ergonómico con comandos para todo el workflow: extract, analyze, check, metrics, convert, etc.

### Objetivo
CLI profesional con:
- Autocompletado (shell completions).
- Progress indicators.
- Colored output.
- Subcomandos bien organizados.

---

## 2. Arquitectura

### 2.1. Estructura de Comandos

```rust
// hodei-cli/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hodei")]
#[command(about = "Unified security & quality scanner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract facts from project
    Extract(ExtractArgs),
    
    /// Analyze IR with rules
    Analyze(AnalyzeArgs),
    
    /// Run full scan (extract + analyze + gates)
    Check(CheckArgs),
    
    /// Convert IR between formats
    Convert(ConvertArgs),
    
    /// Export metrics
    Metrics(MetricsArgs),
    
    /// Query historical data
    History(HistoryArgs),
    
    /// Compare two scans
    Compare(CompareArgs),
    
    /// Validate rules
    ValidateRules(ValidateRulesArgs),
    
    /// Generate shell completions
    Completions(CompletionsArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    match cli.command {
        Commands::Extract(args) => commands::extract::cmd_extract(args).await,
        Commands::Analyze(args) => commands::analyze::cmd_analyze(args).await,
        Commands::Check(args) => commands::check::cmd_check(args).await,
        Commands::Convert(args) => commands::convert::cmd_convert(args).await,
        Commands::Metrics(args) => commands::metrics::cmd_metrics(args).await,
        Commands::History(args) => commands::history::cmd_history(args).await,
        Commands::Compare(args) => commands::compare::cmd_compare(args).await,
        Commands::ValidateRules(args) => commands::validate_rules::cmd_validate_rules(args).await,
        Commands::Completions(args) => commands::completions::cmd_completions(args),
    }
}
```

### 2.2. Comandos Principales

#### Extract
```rust
// hodei-cli/src/commands/extract.rs
#[derive(Args)]
pub struct ExtractArgs {
    /// Project directory to scan
    #[arg(value_name = "DIR")]
    pub project: PathBuf,
    
    /// Output IR file (default: hodei.ir)
    #[arg(short, long, default_value = "hodei.ir")]
    pub output: PathBuf,
    
    /// Extractors to run (comma-separated, default: all)
    #[arg(short, long, value_delimiter = ',')]
    pub extractors: Option<Vec<String>>,
    
    /// Format (capnp, json)
    #[arg(short, long, default_value = "capnp")]
    pub format: IrFormat,
}

pub async fn cmd_extract(args: ExtractArgs) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_message("🔍 Running extractors...");
    
    let ctx = ExtractionContext {
        project_root: args.project,
        config: ExtractorConfig::default(),
        ..Default::default()
    };
    
    let mut runner = ExtractorRunner::new();
    
    // Register extractors
    if let Some(names) = args.extractors {
        for name in names {
            let extractor = create_extractor(&name)?;
            runner.register(extractor);
        }
    } else {
        // Default: all extractors
        runner.register(Box::new(TaintAnalysisExtractor::default()));
        runner.register(Box::new(CoverageExtractor));
        runner.register(Box::new(DependencyExtractor));
    }
    
    let ir = runner.run_all(&ctx).await?;
    pb.finish_with_message("✅ Extraction complete");
    
    // Save IR
    match args.format {
        IrFormat::Capnp => ir.save_capnp(&args.output)?,
        IrFormat::Json => ir.save_json(&args.output)?,
    }
    
    println!("📦 IR saved to {:?} ({} facts)", args.output, ir.facts.len());
    Ok(())
}
```

#### Check (Full Scan)
```rust
// hodei-cli/src/commands/check.rs
#[derive(Args)]
pub struct CheckArgs {
    /// Project directory
    pub project: PathBuf,
    
    /// Rules directory
    #[arg(short, long, default_value = ".hodei/rules")]
    pub rules: PathBuf,
    
    /// Quality gates config
    #[arg(short, long, default_value = ".hodei/quality-gates.yaml")]
    pub gates: PathBuf,
    
    /// Output format (text, json, sarif)
    #[arg(short, long, default_value = "text")]
    pub output_format: OutputFormat,
}

pub async fn cmd_check(args: CheckArgs) -> Result<()> {
    let multi_progress = MultiProgress::new();
    
    // Step 1: Extract
    let pb_extract = multi_progress.add(ProgressBar::new_spinner());
    pb_extract.set_message("🔍 Extracting facts...");
    
    let extract_args = ExtractArgs {
        project: args.project.clone(),
        output: PathBuf::from("hodei.ir"),
        extractors: None,
        format: IrFormat::Capnp,
    };
    cmd_extract(extract_args).await?;
    pb_extract.finish_with_message("✅ Extraction complete");
    
    // Step 2: Analyze
    let pb_analyze = multi_progress.add(ProgressBar::new_spinner());
    pb_analyze.set_message("🧠 Analyzing with rules...");
    
    let ir = IntermediateRepresentation::load_capnp("hodei.ir")?;
    let rules = load_rules_from_dir(&args.rules)?;
    
    let engine = RuleEngine::default();
    let eval_result = engine.evaluate(&rules, &ir)?;
    
    pb_analyze.finish_with_message(format!(
        "✅ Analysis complete ({} findings)",
        eval_result.findings.len()
    ));
    
    // Step 3: Quality Gates
    let pb_gates = multi_progress.add(ProgressBar::new_spinner());
    pb_gates.set_message("🚦 Evaluating quality gates...");
    
    let gates = load_quality_gates(&args.gates)?;
    let mut any_failed = false;
    
    for gate in gates {
        let result = QualityGateEvaluator::evaluate(&gate, &eval_result.findings, &ir);
        print_gate_result(&result);
        
        if !result.passed && matches!(result.action, GateAction::Fail | GateAction::Block) {
            any_failed = true;
        }
    }
    
    pb_gates.finish();
    
    // Output findings
    match args.output_format {
        OutputFormat::Text => print_findings_text(&eval_result.findings),
        OutputFormat::Json => print_findings_json(&eval_result.findings)?,
        OutputFormat::Sarif => print_findings_sarif(&eval_result.findings)?,
    }
    
    if any_failed {
        std::process::exit(1);
    }
    
    Ok(())
}
```

#### Compare
```rust
// hodei-cli/src/commands/compare.rs
#[derive(Args)]
pub struct CompareArgs {
    /// Baseline scan ID
    pub baseline: String,
    
    /// Current scan ID
    pub current: String,
    
    /// Storage backend
    #[arg(short, long, default_value = ".hodei/storage")]
    pub storage: PathBuf,
}

pub async fn cmd_compare(args: CompareArgs) -> Result<()> {
    let storage = SqliteStorage::new(args.storage.join("hodei.db").to_str().unwrap()).await?;
    
    let baseline_metrics = storage.load_metrics(&args.baseline).await?;
    let current_metrics = storage.load_metrics(&args.current).await?;
    
    println!("📊 Comparison: {} → {}\n", args.baseline, args.current);
    
    // Compare coverage
    let cov_diff = current_metrics.coverage.coverage_percentage - baseline_metrics.coverage.coverage_percentage;
    println!("Code Coverage: {:.2}% → {:.2}% ({})",
        baseline_metrics.coverage.coverage_percentage,
        current_metrics.coverage.coverage_percentage,
        format_diff(cov_diff)
    );
    
    // Compare findings by severity
    println!("\nFindings by Severity:");
    for severity in [Severity::Critical, Severity::High, Severity::Medium, Severity::Low] {
        let baseline_count = baseline_metrics.findings_by_severity.get(&severity).unwrap_or(&0);
        let current_count = current_metrics.findings_by_severity.get(&severity).unwrap_or(&0);
        let diff = *current_count as i32 - *baseline_count as i32;
        
        println!("  {:?}: {} → {} ({})",
            severity,
            baseline_count,
            current_count,
            format_diff(diff as f64)
        );
    }
    
    Ok(())
}

fn format_diff(diff: f64) -> String {
    if diff > 0.0 {
        format!("🔴 +{:.2}", diff)
    } else if diff < 0.0 {
        format!("🟢 {:.2}", diff)
    } else {
        "⚪ 0".to_string()
    }
}
```

### 2.3. Shell Completions

```rust
// hodei-cli/src/commands/completions.rs
use clap::CommandFactory;
use clap_complete::{generate, Shell};

#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell type
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn cmd_completions(args: CompletionsArgs) -> Result<()> {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "hodei", &mut std::io::stdout());
    Ok(())
}
```

---

## 3. Plan de Implementación

**Fase 1: Core Commands** (Semana 1)
- [ ] `extract`, `analyze`, `check`.
- [ ] Progress bars con indicatrix.

**Fase 2: Advanced Commands** (Semana 2)
- [ ] `metrics`, `history`, `compare`.
- [ ] Colored output con termcolor.

**Fase 3: Polish** (Semana 2-3)
- [ ] Shell completions.
- [ ] Error messages ergonómicos.
- [ ] Man pages.

---

## 4. Criterios de Aceptación

- [ ] Todos los comandos funcionales.
- [ ] Completions para bash/zsh/fish.
- [ ] Progress indicators UX.
- [ ] Tests E2E.

---

**Última Actualización**: 2025-01-XX
