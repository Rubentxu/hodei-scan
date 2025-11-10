# EPIC-09: Metric Aggregator & Dashboards

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-06 (Rule Engine), EPIC-08 (Quality Gates)  
**Owner**: Observability Team  
**Prioridad**: Medium

---

## 1. Resumen Ejecutivo

Agregar métricas de findings y exportarlas a formatos consumibles (JSON, Prometheus, HTML dashboard). Permite visualizar tendencias de seguridad/calidad a lo largo del tiempo.

### Objetivo de Negocio
Visibilidad de métricas clave (vulnerabilidades por severidad, cobertura, deuda técnica) para toma de decisiones.

### Métricas de Éxito
- Exportación a JSON, Prometheus, HTML.
- Dashboards interactivos (opcional: Grafana).
- API para consultas ad-hoc.

---

## 2. Arquitectura

### 2.1. MetricAggregator

```rust
// hodei-metrics/src/aggregator.rs
use hodei_engine::finding::{Finding, Severity};
use hodei_ir::IntermediateRepresentation;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub timestamp: DateTime<Utc>,
    pub project: String,
    
    pub findings_by_severity: HashMap<Severity, usize>,
    pub findings_by_type: HashMap<String, usize>,
    pub findings_by_confidence: ConfidenceDistribution,
    
    pub coverage: CoverageMetrics,
    pub technical_debt: TechnicalDebtMetrics,
    pub security_posture: SecurityPosture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub uncovered_lines: usize,
    pub coverage_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TechnicalDebtMetrics {
    pub total_hours: f64,
    pub by_severity: HashMap<Severity, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityPosture {
    pub vulnerable_lines: usize,
    pub vulnerable_uncovered_lines: usize,
    pub vulnerability_density: f64,  // vulns per 1000 LOC
}

pub struct MetricAggregator;

impl MetricAggregator {
    pub fn aggregate(
        findings: &[Finding],
        ir: &IntermediateRepresentation,
    ) -> AggregatedMetrics {
        let findings_by_severity = Self::aggregate_by_severity(findings);
        let findings_by_type = Self::aggregate_by_type(findings);
        let coverage = Self::compute_coverage(ir);
        let technical_debt = Self::compute_technical_debt(findings);
        let security_posture = Self::compute_security_posture(findings, ir);
        
        AggregatedMetrics {
            timestamp: Utc::now(),
            project: ir.project.clone(),
            findings_by_severity,
            findings_by_type,
            findings_by_confidence: Self::compute_confidence_distribution(findings),
            coverage,
            technical_debt,
            security_posture,
        }
    }
    
    fn aggregate_by_severity(findings: &[Finding]) -> HashMap<Severity, usize> {
        let mut map = HashMap::new();
        for finding in findings {
            *map.entry(finding.severity).or_insert(0) += 1;
        }
        map
    }
    
    fn compute_coverage(ir: &IntermediateRepresentation) -> CoverageMetrics {
        let covered_lines = ir.facts.iter()
            .filter(|f| matches!(f.fact_type, FactType::CoveredLine))
            .count();
        
        let uncovered_lines = ir.facts.iter()
            .filter(|f| matches!(f.fact_type, FactType::UncoveredLine))
            .count();
        
        let total_lines = covered_lines + uncovered_lines;
        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        CoverageMetrics {
            total_lines,
            covered_lines,
            uncovered_lines,
            coverage_percentage,
        }
    }
    
    fn compute_security_posture(findings: &[Finding], ir: &IntermediateRepresentation) -> SecurityPosture {
        let vulnerable_lines = findings.iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .count();
        
        let vulnerable_uncovered = findings.iter()
            .filter(|f| {
                f.rule_name.contains("Uncovered") &&
                matches!(f.severity, Severity::Critical | Severity::High)
            })
            .count();
        
        let total_loc = ir.facts.iter()
            .filter(|f| matches!(f.fact_type, FactType::CoveredLine | FactType::UncoveredLine))
            .count();
        
        let vulnerability_density = if total_loc > 0 {
            (vulnerable_lines as f64 / total_loc as f64) * 1000.0
        } else {
            0.0
        };
        
        SecurityPosture {
            vulnerable_lines,
            vulnerable_uncovered_lines: vulnerable_uncovered,
            vulnerability_density,
        }
    }
}
```

### 2.2. Exporters

```rust
// hodei-metrics/src/exporters/json.rs
pub struct JsonExporter;

impl JsonExporter {
    pub fn export(metrics: &AggregatedMetrics, output: &Path) -> Result<(), ExporterError> {
        let json = serde_json::to_string_pretty(metrics)?;
        std::fs::write(output, json)?;
        Ok(())
    }
}

// hodei-metrics/src/exporters/prometheus.rs
pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn export(metrics: &AggregatedMetrics) -> String {
        let mut output = String::new();
        
        // Findings by severity
        for (severity, count) in &metrics.findings_by_severity {
            output.push_str(&format!(
                "hodei_findings_total{{severity=\"{:?}\"}} {}\n",
                severity, count
            ));
        }
        
        // Coverage
        output.push_str(&format!(
            "hodei_coverage_percentage {:.2}\n",
            metrics.coverage.coverage_percentage
        ));
        
        // Technical debt
        output.push_str(&format!(
            "hodei_technical_debt_hours {:.2}\n",
            metrics.technical_debt.total_hours
        ));
        
        // Vulnerability density
        output.push_str(&format!(
            "hodei_vulnerability_density {:.2}\n",
            metrics.security_posture.vulnerability_density
        ));
        
        output
    }
}

// hodei-metrics/src/exporters/html.rs
pub struct HtmlDashboardExporter;

impl HtmlDashboardExporter {
    pub fn export(metrics: &AggregatedMetrics, output: &Path) -> Result<(), ExporterError> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Hodei Scan - Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ font-family: Arial; margin: 20px; }}
        .metric {{ margin: 20px 0; }}
        .metric h2 {{ color: #333; }}
        canvas {{ max-width: 600px; }}
    </style>
</head>
<body>
    <h1>Hodei Scan Dashboard</h1>
    <p>Project: {}</p>
    <p>Timestamp: {}</p>
    
    <div class="metric">
        <h2>Findings by Severity</h2>
        <canvas id="severityChart"></canvas>
    </div>
    
    <div class="metric">
        <h2>Code Coverage</h2>
        <p>Coverage: {:.2}%</p>
        <progress value="{}" max="100"></progress>
    </div>
    
    <div class="metric">
        <h2>Technical Debt</h2>
        <p>Total: {:.1} hours</p>
    </div>
    
    <script>
        const severityData = {};
        new Chart(document.getElementById('severityChart'), {{
            type: 'bar',
            data: {{
                labels: ['Critical', 'High', 'Medium', 'Low', 'Info'],
                datasets: [{{
                    label: 'Findings',
                    data: [
                        {},
                        {},
                        {},
                        {},
                        {}
                    ],
                    backgroundColor: ['#dc3545', '#fd7e14', '#ffc107', '#28a745', '#17a2b8']
                }}]
            }}
        }});
    </script>
</body>
</html>
        "#,
            metrics.project,
            metrics.timestamp,
            metrics.coverage.coverage_percentage,
            metrics.coverage.coverage_percentage,
            metrics.technical_debt.total_hours,
            serde_json::to_string(&metrics.findings_by_severity)?,
            metrics.findings_by_severity.get(&Severity::Critical).unwrap_or(&0),
            metrics.findings_by_severity.get(&Severity::High).unwrap_or(&0),
            metrics.findings_by_severity.get(&Severity::Medium).unwrap_or(&0),
            metrics.findings_by_severity.get(&Severity::Low).unwrap_or(&0),
            metrics.findings_by_severity.get(&Severity::Info).unwrap_or(&0),
        );
        
        std::fs::write(output, html)?;
        Ok(())
    }
}
```

### 2.3. CLI Integration

```rust
// hodei-cli/src/commands/metrics.rs
pub async fn cmd_metrics(args: MetricsArgs) -> Result<()> {
    let ir = load_ir(&args.ir_file)?;
    let findings = load_findings(&args.findings_file)?;
    
    let metrics = MetricAggregator::aggregate(&findings, &ir);
    
    match args.format {
        OutputFormat::Json => {
            JsonExporter::export(&metrics, &args.output)?;
        }
        OutputFormat::Prometheus => {
            let prom_text = PrometheusExporter::export(&metrics);
            std::fs::write(&args.output, prom_text)?;
        }
        OutputFormat::Html => {
            HtmlDashboardExporter::export(&metrics, &args.output)?;
        }
    }
    
    println!("✅ Metrics exported to {:?}", args.output);
    Ok(())
}
```

---

## 3. Plan de Implementación

**Fase 1: Aggregator** (Semana 1)
- [ ] Implementar `MetricAggregator`.
- [ ] Tests con fixtures.

**Fase 2: Exporters** (Semana 2)
- [ ] JSON exporter.
- [ ] Prometheus exporter.
- [ ] HTML dashboard.

**Fase 3: CLI & CI Integration** (Semana 2)
- [ ] Comando `hodei metrics`.
- [ ] Ejemplo de pipeline con upload a S3/GCS.

---

## 4. Criterios de Aceptación

- [ ] Exportación a JSON, Prometheus, HTML.
- [ ] Dashboard HTML con charts.
- [ ] CLI funcional.
- [ ] Tests >85%.

---

**Última Actualización**: 2025-01-XX
