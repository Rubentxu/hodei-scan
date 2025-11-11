# EPIC-13: Backend de Gobernanza - hodei-server

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan   
**Dependencias**: EPIC-10 (ExtractorOrchestrator), EPIC-11 (IR Schema Evolution)  
**Owner**: Platform Team  
**Prioridad**: High Path

---

## 1. Resumen Ejecutivo

Implementar **`hodei-server`**, un backend stateful opcional que transforma hodei-scan de una herramienta stateless de CI/CD a una **plataforma completa de gobernanza de software**. Esta épica habilita análisis de tendencias, gestión de deuda técnica, y dashboards ejecutivos.

### Objetivo de Negocio
Convertir hodei-scan en la **única plataforma** que combine análisis estático en tiempo real (CLI stateless) con **inteligencia histórica y predictiva** (backend stateful).

### Métricas de Éxito
- **Almacenamiento**: 1M+ findings históricos por proyecto
- **Análisis de Tendencias**: Comparación entre commits/ramas en <2s
- **Reducción de Ruido**: 90% menos falsos positivos vía baselining
- **Adopción Ejecutiva**: Dashboard usado por 10+ teams en 3 meses

---

## 2. Contexto Técnico

### 2.1. Problema Actual
hodei-scan v3.2 es **100% stateless**:
- Solo análisis puntual (snapshot)
- No hay contexto histórico
- Imposible distinguir problemas nuevos vs deuda preexistente
- Sin métricas de tendencias para liderazgo

### 2.2. Solución: Arquitectura Híbrida

```
┌─────────────────────────────────────────────────────────────┐
│                  hodei-scan  Platform                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │ hodei-scan CLI  │──publish▶│     hodei-server        │   │
│  │ (Stateless)     │         │     (Stateful)          │   │
│  │                 │         │                         │   │
│  │ • CI/CD         │         │ • Historical Storage    │   │
│  │ • Performance   │         │ • Trend Analysis        │   │
│  │ • Zero Config   │         │ • Debt Management       │   │
│  │                 │◀────────│ • Executive Dashboards  │   │
│  └─────────────────┘   query │ └─────────────────────────┘   │
│                                 │                            │
│                                 ▼                            │
│                       ┌──────────────────┐                 │
│                       │   Database       │                 │
│                       │  (TimescaleDB/   │                 │
│                       │   ClickHouse)    │                 │
│                       └──────────────────┘                 │
└─────────────────────────────────────────────────────────────┘
```

**Contrato de Publicación:**
```bash
hodei-scan analyze --rules security.hodei --publish \
  --server http://hodei-server:8080 \
  --project my-app --branch feature/login-fix
```

---

## 3. Arquitectura Detallada

### 3.1. Componentes Core

#### hodei-server
```rust
pub struct HodeiServer {
    config: ServerConfig,
    db: DatabaseConnection,
    storage: HistoricalStorage,
    analyzer: TrendAnalyzer,
    pubsub: EventBus,  // Para notificaciones
}

impl HodeiServer {
    /// Endpoint principal para recibir análisis
    pub async fn publish_analysis(
        &self,
        project_id: &str,
        analysis: AnalysisSnapshot,
    ) -> Result<PublishResponse, ServerError> {
        // 1. Validar snapshot
        // 2. Store en DB
        // 3. Calcular diff vs baseline
        // 4. PubSub notification
        // 5. Return summary
    }
}
```

#### Database Schema (TimescaleDB)
```sql
-- Findings históricos
CREATE TABLE findings (
    id BIGSERIAL PRIMARY KEY,
    project_id TEXT NOT NULL,
    branch TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    
    -- Finding data (JSONB para flexibility)
    fact_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    location JSONB NOT NULL,  -- file, line, column
    message TEXT NOT NULL,
    metadata JSONB,
    
    -- Tags para filtering
    tags TEXT[],
    
    -- Time-based partitioning
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Hypertable para time-series optimization
SELECT create_hypertable('findings', 'timestamp');

-- Índices para performance
CREATE INDEX idx_findings_project_branch ON findings(project_id, branch);
CREATE INDEX idx_findings_timestamp ON findings USING GIST(timestamp);
CREATE INDEX idx_findings_fact_type ON findings(fact_type);

-- View para trends
CREATE VIEW findings_trend_daily AS
SELECT 
    DATE_TRUNC('day', timestamp) as day,
    fact_type,
    severity,
    COUNT(*) as count
FROM findings
GROUP BY day, fact_type, severity;
```

### 3.2. API Design

#### REST Endpoints
```yaml
# Publish analysis results
POST /api/v1/projects/{project_id}/analyses
Content-Type: application/json
Authorization: Bearer {token}

{
  "branch": "feature/login-fix",
  "commit": "abc123",
  "findings": [...],  # IR findings
  "metadata": {
    "build_url": "...",
    "author": "...",
    "ci_run_id": "..."
  }
}

# Response
{
  "analysis_id": "uuid",
  "new_findings": 5,
  "resolved_findings": 2,
  "trend": "improving",  # improving, degrading, stable
  "summary_url": "/api/v1/analyses/uuid"
}

# Get trend analysis
GET /api/v1/projects/{project_id}/trends?from=2025-01-01&to=2025-12-31
{
  "period": {
    "start": "2025-01-01T00:00:00Z",
    "end": "2025-12-31T23:59:59Z"
  },
  "metrics": {
    "total_findings": 150,
    "critical_findings": 12,
    "trend": -15,  # % change
    "by_severity": {
      "critical": 12,
      "major": 38,
      "minor": 100
    }
  }
}

# Diff analysis (base vs head)
GET /api/v1/projects/{project_id}/diff?base=main&head=feature-branch
{
  "base_analysis": {
    "commit": "abc123",
    "findings_count": 145
  },
  "head_analysis": {
    "commit": "def456",
    "findings_count": 150
  },
  "diff": {
    "new_findings": [
      {
        "fact_type": "Vulnerability",
        "location": "src/auth.rs:45",
        "message": "SQL Injection risk",
        "severity": "critical"
      }
    ],
    "resolved_findings": [...],
    "status_changed": [...]  # severity changed, won't fix, etc.
  }
}
```

#### gRPC API (alternativa)
```protobuf
service HodeiGovernance {
  rpc PublishAnalysis(PublishRequest) returns (PublishResponse);
  rpc GetTrends(GetTrendsRequest) returns (GetTrendsResponse);
  rpc GetDiff(GetDiffRequest) returns (GetDiffResponse);
  rpc GetBaseline(GetBaselineRequest) returns (GetBaselineResponse);
  rpc SetBaseline(SetBaselineRequest) returns (SetBaselineResponse);
}
```

### 3.3. Flujo de Análisis de Tendencias

```
1. Developer runs hodei-scan
   ↓
2. hodei-scan publishes to hodei-server
   ↓
3. Server stores findings + calculates diff
   ↓
4. Server identifies NEW findings
   ↓
5. Return summary + CI decision
   ↓
6. Update dashboard metrics
```

**Ejemplo de Diff:**
```rust
pub struct AnalysisDiff {
    pub new_findings: Vec<Finding>,
    pub resolved_findings: Vec<Finding>,  // were present in baseline
    pub severity_increased: Vec<Finding>,
    pub severity_decreased: Vec<Finding>,
    pub wont_fix_changed: Vec<Finding>,
}

impl HodeiServer {
    pub async fn calculate_diff(
        &self,
        current: &[Finding],
        baseline: &[Finding],
    ) -> Result<AnalysisDiff, ServerError> {
        // Hash-based comparison para performance
        let current_set: HashSet<_> = current.iter().map(|f| f.fingerprint()).collect();
        let baseline_set: HashSet<_> = baseline.iter().map(|f| f.fingerprint()).collect();

        Ok(AnalysisDiff {
            new_findings: current_set.difference(&baseline_set).cloned().collect(),
            resolved_findings: baseline_set.difference(&current_set).cloned().collect(),
            // ... otros diffs
        })
    }
}
```

---

## 4. Plan de Implementación

### 4.1. Fases

**Fase 1: Core Storage (Semana 1-2)**
- [ ] Setup TimescaleDB/ClickHouse
- [ ] Implement HistoricalStorage
- [ ] Basic publish/get APIs
- [ ] Test con datos reales

**Fase 2: Diff & Trends (Semana 3)**
- [ ] Baseline management
- [ ] Diff calculation engine
- [ ] Trend analysis algorithms
- [ ] Performance optimization

**Fase 3: Dashboard (Semana 4-6)**
- [ ] React/Vue frontend
- [ ] Metrics visualization
- [ ] Real-time updates (WebSocket/SSE)
- [ ] Executive reports

---

## 5. User Stories

### US-13.01: Setup hodei-server Architecture

**Como:** DevOps Engineer  
**Quiero:** Un servidor backend que almacene análisis históricos  
**Para:** Habilitar análisis de tendencias y gestión de deuda

**Criterios de Aceptación:**
- [ ] hodei-server binary compiled and dockerized
- [ ] TimescaleDB/ClickHouse configured
- [ ] Database schema created
- [ ] Basic health check endpoint
- [ ] Authentication (JWT) configured
- [ ] Docker Compose for local dev

**TDD - Red:**
```rust
#[tokio::test]
async fn test_server_startup() {
    let config = ServerConfig {
        db_url: "postgres://...".to_string(),
        port: 8080,
        ..Default::default()
    };
    
    let server = HodeiServer::new(config).await;
    assert!(server.is_ok());
    
    let response = reqwest::get("http://localhost:8080/health").await;
    assert!(response.unwrap().status().is_success());
}
```

**TDD - Green:**
```rust
pub struct HodeiServer {
    config: ServerConfig,
    db: DatabaseConnection,
    app: Router,
}

impl HodeiServer {
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError> {
        let db = DatabaseConnection::connect(&config.db_url).await?;
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/projects/:id/analyses", post(publish_analysis));
        
        Ok(Self { config, db, app })
    }
}
```

**TDD - Refactor:**
- Add connection pooling
- Implement graceful shutdown
- Add structured logging
- Health checks for DB

**Conventional Commit:**
`feat(server): implement hodei-server architecture with TimescaleDB`

---

### US-13.02: Historical Storage APIs

**Como:** hodei-scan CLI  
**Quiero:** Publicar resultados de análisis al backend  
**Para:** Almacenar snapshots históricos

**Criterios de Aceptación:**
- [ ] POST /api/v1/projects/{id}/analyses endpoint
- [ ] Validación de payload
- [ ] Storage optimizado (batch inserts)
- [ ] Response con analysis_id
- [ ] Rate limiting
- [ ] Data retention policies

**TDD - Red:**
```rust
#[tokio::test]
async fn test_publish_analysis() {
    let server = setup_test_server().await;
    
    let payload = PublishRequest {
        project_id: "my-app".to_string(),
        branch: "main".to_string(),
        commit: "abc123".to_string(),
        findings: vec![ /* findings */ ],
    };
    
    let response = server.publish_analysis("my-app", payload).await;
    assert!(response.is_ok());
    
    let analysis = server.get_analysis(response.analysis_id).await.unwrap();
    assert_eq!(analysis.findings.len(), 100);
}
```

**TDD - Green:**
```rust
pub async fn publish_analysis(
    &self,
    project_id: &str,
    request: PublishRequest,
) -> Result<PublishResponse, ServerError> {
    // 1. Validate project exists
    self.validate_project(project_id).await?;
    
    // 2. Store findings (batch insert para performance)
    let analysis_id = self.storage.store_analysis(request).await?;
    
    // 3. Calculate summary metrics
    let summary = self.calculate_summary(&request.findings);
    
    Ok(PublishResponse {
        analysis_id,
        summary,
    })
}
```

**Conventional Commit:**
`feat(server): implement historical storage with batch inserts`

---

### US-13.03: Diff Analysis APIs

**Como:** Developer  
**Quiero:** Ver solo los problemas NUEVOS introducidos en mi PR  
**Para:** No fallar CI por deuda preexistente

**Criterios de Aceptación:**
- [ ] GET /api/v1/projects/{id}/diff endpoint
- [ ] Support base/head branches o commits
- [ ] Hash-based finding comparison
- [ ] Performance <2s for 10K findings
- [ ] Categorize diffs (new, resolved, severity change)
- [ ] Include baseline management

**TDD - Red:**
```rust
#[tokio::test]
async fn test_diff_analysis() {
    let server = setup_server_with_baseline().await;
    
    let diff = server.get_diff("my-app", "main", "feature-branch").await.unwrap();
    
    assert_eq!(diff.new_findings.len(), 5);
    assert_eq!(diff.resolved_findings.len(), 2);
    assert!(diff.severity_increased.is_empty());
}
```

**TDD - Green:**
```rust
pub async fn get_diff(
    &self,
    project_id: &str,
    base: &str,
    head: &str,
) -> Result<AnalysisDiff, ServerError> {
    // 1. Fetch baseline analysis
    let base_analysis = self.storage.get_latest_analysis(project_id, base).await?;
    
    // 2. Fetch head analysis
    let head_analysis = self.storage.get_latest_analysis(project_id, head).await?;
    
    // 3. Calculate diff
    self.diff_engine.calculate(&head_analysis, &base_analysis)
}

impl DiffEngine {
    fn calculate(&self, current: &[Finding], baseline: &[Finding]) -> AnalysisDiff {
        let current_fingerprints: HashSet<_> = current.iter().map(|f| f.fingerprint()).collect();
        let baseline_fingerprints: HashSet<_> = baseline.iter().map(|f| f.fingerprint()).collect();
        
        AnalysisDiff {
            new_findings: current_fingerprints.difference(&baseline_fingerprints)
                .map(|fp| current.iter().find(|f| f.fingerprint() == fp).unwrap().clone())
                .collect(),
            resolved_findings: baseline_fingerprints.difference(&current_fingerprints)
                .map(|fp| baseline.iter().find(|f| f.fingerprint() == fp).unwrap().clone())
                .collect(),
            // ... otros
        }
    }
}
```

**Conventional Commit:**
`feat(server): implement diff analysis for PR baselining`

---

### US-13.04: Executive Dashboard

**Como:** Engineering Manager  
**Quiero:** Ver tendencias de calidad y seguridad en dashboard web  
**Para:** Tomar decisiones basadas en datos históricos

**Criterios de Aceptación:**
- [ ] React/Vue dashboard with charts
- [ ] Time-series visualization (findings over time)
- [ ] Severity breakdown (pie/bar charts)
- [ ] Branch comparison
- [ ] Real-time updates via WebSocket
- [ ] Export PDF reports
- [ ] Mobile responsive

**Frontend Stack:**
```typescript
// React + Recharts + TanStack Query
export function TrendDashboard() {
  const { data: trends } = useQuery({
    queryKey: ['trends', projectId, timeRange],
    queryFn: () => api.getTrends(projectId, timeRange),
    refetchInterval: 30000,  // Real-time updates
  });
  
  return (
    <div>
      <h2>Quality Trends - Last 90 Days</h2>
      <LineChart data={trends?.daily}>
        <Line dataKey="critical" stroke="#ff0000" />
        <Line dataKey="major" stroke="#ff9900" />
        <Line dataKey="minor" stroke="#00ff00" />
      </LineChart>
      
      <PieChart data={trends?.bySeverity}>
        <Pie dataKey="count" nameKey="severity" />
      </PieChart>
    </div>
  );
}
```

**Backend Integration:**
```rust
pub async fn websocket_handler(
    ws: WebSocket,
    project_id: String,
) {
    // Stream real-time metrics
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        let metrics = get_latest_metrics(&project_id).await;
        let message = Message::Text(serde_json::to_string(&metrics).unwrap());
        let _ = ws.send(message).await;
    }
}
```

**Conventional Commit:**
`feat(dashboard): implement executive dashboard with real-time trends`

---

### US-13.05: Baseline & Debt Management

**Como:** Team Lead  
**Quiero:** Marcar hallazgos como "aceptados" o "won't fix"  
**Para:** No fallar builds por deuda técnica conocida

**Criterios de Aceptación:**
- [ ] Mark finding as accepted/wontfix
- [ ] Update baseline from current analysis
- [ ] Exclude baseline findings from CI failures
- [ ] Audit trail for baseline changes
- [ ] Bulk baseline operations
- [ ] Restore previous baseline

**API Design:**
```rust
// Update baseline for branch
POST /api/v1/projects/{id}/baselines/{branch}
{
  "action": "update_from_analysis",  // or "restore"
  "analysis_id": "uuid"
}

// Mark individual finding
POST /api/v1/projects/{id}/findings/{finding_id}/status
{
  "status": "accepted",  // accepted, wontfix, false_positive
  "reason": "Technical debt, will refactor in Q2",
  "expires_at": "2025-06-01"
}
```

**Conventional Commit:**
`feat(server): implement baseline and debt management system`

---

## 6. Testing Strategy

### 6.1. Unit Tests
- Diff calculation algorithms
- Database query optimization
- Data validation
- Error handling

### 6.2. Integration Tests
- Test con TimescaleDB real
- End-to-end publish → diff → dashboard
- Performance tests (10K findings)

### 6.3. Load Tests
- Concurrent publishes
- Dashboard real-time updates
- Database query performance

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Database performance | Alto | Alto | Profiling + index optimization |
| TimescaleDB complexity | Medio | Medio | Start with PostgreSQL, migrate later |
| Real-time dashboard lag | Alto | Bajo | WebSocket con buffering |
| Storage cost growth | Alto | Medio | Data retention policies + archival |

---

## 8. Definition of Done

- [ ] hodei-server running y dockerized
- [ ] Historical storage con 1M+ findings test
- [ ] Diff APIs <2s response time
- [ ] Dashboard con real-time updates
- [ ] Baseline management funcionando
- [ ] Performance tests passing (10K findings)
- [ ] CI integration (hodei-scan publish)
- [ ] Documentation completa
- [ ] Security review passed

---

**Estimación Total**: 6 Sprints (12 semanas)  
**Commit Messages**:  
- `feat(server): implement hodei-server architecture`  
- `feat(server): add historical storage APIs`  
- `feat(server): implement diff analysis`  
- `feat(server): add baseline management`  
- `feat(dashboard): build executive dashboard`  

---

**Referencias Técnicas**:
- TimescaleDB: https://docs.timescale.com/
- ClickHouse: https://clickhouse.com/
- Axum (Rust web framework): https://github.com/tokio-rs/axum
- Recharts (React charts): https://recharts.org/
