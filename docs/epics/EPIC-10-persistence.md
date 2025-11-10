# EPIC-10: Persistence Layer (JSON/SQLite)

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02 (IR Core), EPIC-06 (Rule Engine)  
**Owner**: Infrastructure Team  
**Prioridad**: Medium

---

## 1. Resumen Ejecutivo

Implementar capa de persistencia para almacenar IRs, findings y métricas. Soporte para JSON (simple) y SQLite (queries complejas).

### Objetivo
- Historial de análisis (trending).
- Queries sobre findings históricos.
- Comparación de métricas entre commits.

---

## 2. Arquitectura

### 2.1. Storage Trait

```rust
// hodei-storage/src/lib.rs
use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_ir(&self, ir: &IntermediateRepresentation) -> Result<String, StorageError>;
    async fn load_ir(&self, id: &str) -> Result<IntermediateRepresentation, StorageError>;
    
    async fn store_findings(&self, scan_id: &str, findings: &[Finding]) -> Result<(), StorageError>;
    async fn load_findings(&self, scan_id: &str) -> Result<Vec<Finding>, StorageError>;
    
    async fn store_metrics(&self, scan_id: &str, metrics: &AggregatedMetrics) -> Result<(), StorageError>;
    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<AggregatedMetrics>, StorageError>;
}

#[derive(Debug, Clone)]
pub struct MetricsQuery {
    pub project: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: usize,
}
```

### 2.2. JSON Storage (Simple)

```rust
// hodei-storage/src/json.rs
pub struct JsonStorage {
    base_path: PathBuf,
}

impl JsonStorage {
    pub fn new(base_path: PathBuf) -> Self {
        std::fs::create_dir_all(&base_path).unwrap();
        Self { base_path }
    }
}

#[async_trait]
impl Storage for JsonStorage {
    async fn store_ir(&self, ir: &IntermediateRepresentation) -> Result<String, StorageError> {
        let id = format!("{}-{}", ir.project, ir.timestamp.timestamp());
        let path = self.base_path.join(format!("{}.ir.json", id));
        
        let json = serde_json::to_string_pretty(ir)?;
        tokio::fs::write(&path, json).await?;
        
        Ok(id)
    }
    
    async fn load_ir(&self, id: &str) -> Result<IntermediateRepresentation, StorageError> {
        let path = self.base_path.join(format!("{}.ir.json", id));
        let json = tokio::fs::read_to_string(&path).await?;
        let ir = serde_json::from_str(&json)?;
        Ok(ir)
    }
    
    async fn store_findings(&self, scan_id: &str, findings: &[Finding]) -> Result<(), StorageError> {
        let path = self.base_path.join(format!("{}.findings.json", scan_id));
        let json = serde_json::to_string_pretty(findings)?;
        tokio::fs::write(&path, json).await?;
        Ok(())
    }
    
    // ... implementar resto
}
```

### 2.3. SQLite Storage (Queryable)

```rust
// hodei-storage/src/sqlite.rs
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(db_path: &str) -> Result<Self, StorageError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}", db_path))
            .await?;
        
        // Crear tablas
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS scans (
                id TEXT PRIMARY KEY,
                project TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                ir_capnp BLOB
            )
        "#).execute(&pool).await?;
        
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS findings (
                id INTEGER PRIMARY KEY,
                scan_id TEXT NOT NULL,
                rule_name TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                file TEXT,
                line_start INTEGER,
                confidence REAL,
                FOREIGN KEY (scan_id) REFERENCES scans(id)
            )
        "#).execute(&pool).await?;
        
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS metrics (
                scan_id TEXT PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                coverage_percentage REAL,
                total_findings INTEGER,
                critical_findings INTEGER,
                high_findings INTEGER,
                technical_debt_hours REAL,
                FOREIGN KEY (scan_id) REFERENCES scans(id)
            )
        "#).execute(&pool).await?;
        
        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn store_ir(&self, ir: &IntermediateRepresentation) -> Result<String, StorageError> {
        let id = format!("{}-{}", ir.project, ir.timestamp.timestamp());
        
        // Serializar a Cap'n Proto
        let capnp_bytes = ir.to_capnp_bytes()?;
        
        sqlx::query(
            "INSERT INTO scans (id, project, timestamp, ir_capnp) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&ir.project)
        .bind(ir.timestamp.timestamp())
        .bind(&capnp_bytes)
        .execute(&self.pool)
        .await?;
        
        Ok(id)
    }
    
    async fn store_findings(&self, scan_id: &str, findings: &[Finding]) -> Result<(), StorageError> {
        for finding in findings {
            sqlx::query(
                "INSERT INTO findings (scan_id, rule_name, severity, message, file, line_start, confidence)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(scan_id)
            .bind(&finding.rule_name)
            .bind(format!("{:?}", finding.severity))
            .bind(&finding.message)
            .bind(finding.location.as_ref().map(|l| l.file.to_string()))
            .bind(finding.location.as_ref().map(|l| l.start.line.0))
            .bind(finding.confidence.value())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
    
    async fn query_metrics(&self, query: MetricsQuery) -> Result<Vec<AggregatedMetrics>, StorageError> {
        let rows = sqlx::query_as::<_, MetricsRow>(
            "SELECT * FROM metrics 
             WHERE scan_id IN (
                 SELECT id FROM scans WHERE project = ?
                 AND timestamp >= ? AND timestamp <= ?
             )
             ORDER BY timestamp DESC
             LIMIT ?"
        )
        .bind(&query.project)
        .bind(query.start_date.unwrap_or_else(|| Utc::now() - Duration::days(30)).timestamp())
        .bind(query.end_date.unwrap_or_else(|| Utc::now()).timestamp())
        .bind(query.limit as i64)
        .fetch_all(&self.pool)
        .await?;
        
        // Convert rows to AggregatedMetrics
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}
```

---

## 3. Plan de Implementación

**Fase 1: JSON Storage** (Semana 1)
- [ ] Implementar `JsonStorage`.
- [ ] Tests básicos (store/load).

**Fase 2: SQLite Storage** (Semana 2)
- [ ] Schema design.
- [ ] Implementar `SqliteStorage`.
- [ ] Tests con queries.

**Fase 3: CLI Integration** (Semana 2)
- [ ] Comando `hodei history`.
- [ ] Comando `hodei compare`.

---

## 4. Criterios de Aceptación

- [ ] JSON y SQLite storage funcionales.
- [ ] Queries sobre findings históricos.
- [ ] CLI para consultas.
- [ ] Tests >85%.

---

**Última Actualización**: 2025-01-XX
