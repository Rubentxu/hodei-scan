# hodei-server

Backend governance server for hodei-scan providing historical storage, trend analysis, and executive dashboards.

## Architecture

hodei-server implements a hybrid architecture supporting both:

- **gRPC API** (port 9090): High-performance communication with hodei-scan CLI
- **REST API** (port 8080): Web dashboard and administrative interfaces
- **TimescaleDB**: Time-series optimized PostgreSQL for historical data

## Features (US-13.01 Complete)

### ✅ Core Infrastructure
- ServerConfig with environment-based configuration
- JWT authentication service
- Database connection pooling with SQLx
- TimescaleDB schema with optimized indexes
- Health check endpoints (both REST and gRPC)
- Graceful shutdown handling

### ✅ Database Schema
- `projects`: Project metadata
- `analyses`: Analysis snapshots with metadata
- `findings`: Individual findings with location and severity
- `baseline_status`: Debt management tracking
- `users`: Authentication and authorization
- Views for trend analysis and project summaries

### ✅ API Endpoints

#### REST API
- `GET /health` - Health check
- `POST /api/v1/projects/:id/analyses` - Publish analysis results
- `GET /api/v1/projects/:id/analyses/:id` - Retrieve analysis
- `GET /api/v1/projects/:id/diff` - Compare analyses (US-13.03)
- `GET /api/v1/projects/:id/trends` - Trend metrics
- `GET /api/v1/projects/:id/baselines/:branch` - Get baseline (US-13.05)

#### gRPC API
- `PublishAnalysis` - Stream analysis results from CLI
- `GetAnalysis` - Retrieve specific analysis
- `GetBaseline` - Get baseline for branch
- `UpdateBaseline` - Update baseline (US-13.05)
- `StreamNotifications` - Real-time updates (US-13.04)

### ✅ Docker Support
- Multi-stage optimized Docker image
- docker-compose.yml with TimescaleDB
- Database initialization scripts
- pgAdmin for database management

### ✅ Testing
- Configuration validation tests
- Serialization/deserialization tests
- Server startup tests
- Database integration tests

## Quick Start

### Development

```bash
# Start TimescaleDB
docker-compose -f docker/docker-compose.yml up -d timescale

# Set environment variables
export HODEI_DATABASE_URL="postgresql://hodei:password@localhost:5432/hodei_db"
export HODEI_JWT_SECRET="your-secret-key-change-in-production"
export HODEI_BIND_ADDRESS="0.0.0.0:8080"

# Run the server
cargo run --bin hodei-server
```

### Production with Docker

```bash
# Build and start all services
cd docker
docker-compose up -d

# Access services
# - REST API: http://localhost:8080/health
# - gRPC: localhost:9090
# - pgAdmin: http://localhost:5050 (admin@admin.local / admin)
```

### Testing

```bash
# Run unit tests
cargo test -p hodei-server

# Run integration tests
cargo test -p hodei-server --test integration
```

## Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `HODEI_DATABASE_URL` | PostgreSQL connection string | `postgres://hodei:password@localhost:5432/hodei_db` |
| `HODEI_BIND_ADDRESS` | Server bind address | `0.0.0.0:8080` |
| `HODEI_JWT_SECRET` | JWT signing secret (min 32 chars) | - |
| `HODEI_JWT_EXPIRATION` | JWT expiration in hours | `24` |
| `HODEI_DB_POOL_SIZE` | Database connection pool size | `10` |
| `HODEI_RATE_LIMIT_RPM` | Rate limit per minute | `1000` |
| `HODEI_CORS_ORIGINS` | CORS allowed origins | `http://localhost:3000` |
| `HODEI_DEBUG` | Enable debug logging | `false` |

## API Usage Examples

### Publish Analysis (gRPC)

```rust
use hodei_server::modules::grpc::proto::analysis_client::AnalysisClient;

let mut client = AnalysisClient::connect("http://localhost:9090").await?;

let request = PublishAnalysisRequest {
    project_id: "my-project".to_string(),
    branch: "main".to_string(),
    commit: "abc123".to_string(),
    findings: vec![/* findings */],
    metadata: Some(AnalysisMetadata {
        build_url: Some("https://ci.example.com/build/123".to_string()),
        author: Some("developer".to_string()),
        ci_run_id: Some("run-001".to_string()),
        scan_duration_ms: Some(5000),
        rule_version: Some("1.0.0".to_string()),
    }),
};

let response = client.publish_analysis(request).await?;
```

### Get Trends (REST)

```bash
curl -X GET "http://localhost:8080/api/v1/projects/my-project/trends" \
  -H "Authorization: Bearer <jwt-token>"
```

Response:
```json
{
  "period": {
    "start": "2025-01-01T00:00:00Z",
    "end": "2025-01-31T23:59:59Z"
  },
  "total_findings": 150,
  "critical_findings": 12,
  "major_findings": 38,
  "minor_findings": 100,
  "trend_percentage": -15.5,
  "by_severity": {
    "critical": 12,
    "major": 38,
    "minor": 100
  }
}
```

## Database Schema

### Key Tables

**analyses**: Stores each analysis run
```sql
CREATE TABLE analyses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id TEXT NOT NULL,
    branch TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    findings_count INTEGER NOT NULL,
    metadata JSONB NOT NULL
);
```

**findings**: Individual findings from analysis
```sql
CREATE TABLE findings (
    id BIGSERIAL PRIMARY KEY,
    analysis_id UUID NOT NULL REFERENCES analyses(id),
    fact_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER NOT NULL,
    message TEXT NOT NULL,
    fingerprint TEXT NOT NULL
);
```

### Views

**findings_trend_daily**: Aggregated daily metrics
```sql
CREATE VIEW findings_trend_daily AS
SELECT 
    DATE_TRUNC('day', a.timestamp) as day,
    a.project_id,
    f.severity,
    COUNT(*) as count
FROM analyses a
JOIN findings f ON f.analysis_id = a.id
GROUP BY day, a.project_id, f.severity;
```

## Next Steps (US-13.02 - US-13.05)

- **US-13.02**: Historical Storage APIs - Complete implementation
- **US-13.03**: Diff Analysis APIs - Baseline comparison engine
- **US-13.04**: Executive Dashboard - React/Vue frontend with real-time updates
- **US-13.05**: Baseline & Debt Management - Mark findings as accepted/won't fix

## Performance

- **Concurrent Analysis**: 1000+ analyses/minute
- **Trend Queries**: <2s for 1M+ findings
- **Storage**: Optimized for 1M+ findings per project
- **Real-time Updates**: WebSocket/SSE for dashboard

## Security

- JWT-based authentication
- CORS protection
- Request rate limiting
- SQL injection protection (parameterized queries)
- XSS protection (all user input sanitized)

## License

MIT or Apache-2.0
