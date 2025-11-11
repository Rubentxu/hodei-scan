/// Core hodei-server implementation with both REST and gRPC support
use crate::modules::auth::AuthService;
use crate::modules::config::ServerConfig;
use crate::modules::database::DatabaseConnection;
use crate::modules::error::{Result, ServerError};
use crate::modules::grpc::{HodeiGrpcServer, proto::health_server::HealthServer};
use crate::modules::types::{
    AnalysisDiff, AnalysisId, AnalysisMetadata, ApiError, AuthToken, HealthStatus,
    HealthCheckStatus, PublishRequest, PublishResponse, ProjectId, Severity, StoredAnalysis,
    TrendDirection, TrendMetrics, UserId,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;
use tonic::transport::{Server, ServerTlsConfig};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};

/// Main hodei-server instance
pub struct HodeiServer {
    config: ServerConfig,
    database: DatabaseConnection,
    auth_service: AuthService,
    rest_app: Router,
    grpc_server: Option<HodeiGrpcServer>,
    start_time: SystemTime,
    shutdown_sender: broadcast::Sender<()>,
}

impl HodeiServer {
    /// Create a new hodei-server instance
    pub async fn new(config: ServerConfig) -> Result<Self> {
        info!("Initializing hodei-server...");

        // Validate configuration
        config.validate().map_err(|e| {
            ServerError::Config(format!("Configuration validation failed: {}", e))
        })?;

        // Initialize database connection
        info!("Connecting to database...");
        let database = DatabaseConnection::new(&config.database_url, config.db_pool_size)
            .await
            .map_err(|e| {
                ServerError::Config(format!("Database connection failed: {}", e))
            })?;

        // Initialize database schema
        info!("Initializing database schema...");
        database.initialize_schema().await.map_err(|e| {
            ServerError::Config(format!("Database schema initialization failed: {}", e))
        })?;

        // Initialize authentication service
        let auth_service = AuthService::new(config.jwt_secret.clone(), config.jwt_expiration_hours);

        // Create gRPC server
        let grpc_server = Some(HodeiGrpcServer::new(database.clone()));

        // Create the REST router
        let rest_app = Self::create_rest_router(&config, &database, &auth_service).await?;

        let start_time = SystemTime::now();
        let (shutdown_sender, _) = broadcast::channel(1);

        info!("hodei-server initialized successfully on {}", config.bind_address);

        Ok(Self {
            config,
            database,
            auth_service,
            rest_app,
            grpc_server,
            start_time,
            shutdown_sender,
        })
    }

    /// Create the REST router
    async fn create_rest_router(
        config: &ServerConfig,
        database: &DatabaseConnection,
        auth_service: &AuthService,
    ) -> Result<Router> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            // Health check endpoint (no auth required)
            .route("/health", get(health_check))
            
            // Authentication endpoints
            .route("/api/v1/auth/login", post(login))
            .route("/api/v1/auth/refresh", post(refresh_token))
            
            // Analysis endpoints (auth required)
            .route(
                "/api/v1/projects/:project_id/analyses",
                post(publish_analysis),
            )
            .route(
                "/api/v1/projects/:project_id/analyses/:analysis_id",
                get(get_analysis),
            )
            
            // Diff analysis endpoint
            .route(
                "/api/v1/projects/:project_id/diff",
                get(get_diff_analysis),
            )
            
            // Trend analysis endpoint
            .route(
                "/api/v1/projects/:project_id/trends",
                get(get_trends),
            )
            
            // Project endpoints
            .route("/api/v1/projects", get(list_projects))
            .route("/api/v1/projects/:project_id", get(get_project))
            
            // Baseline endpoints
            .route(
                "/api/v1/projects/:project_id/baselines/:branch",
                get(get_baseline),
            )
            .route(
                "/api/v1/projects/:project_id/baselines/:branch",
                post(update_baseline),
            )
            
            // Apply middleware
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .with_state(AppState {
                database: database.clone(),
                auth_service: auth_service.clone(),
                config: config.clone(),
            })
    }

    /// Start the server (both REST and gRPC)
    pub async fn run(self) -> Result<()> {
        let addr = self.config.bind_address;
        info!("Starting hodei-server on {}", addr);

        let (shutdown_recv, shutdown_server) = tokio::sync::oneshot::channel::<()>();
        let shutdown_sender = self.shutdown_sender.clone();

        // Setup graceful shutdown
        let server_handle = tokio::spawn(async move {
            let rest_server = axum::serve(
                tokio::net::TcpListener::bind(addr)
                    .await
                    .map_err(|e| ServerError::Internal(format!("Failed to bind to {}: {}", addr, e)))?,
                self.rest_app,
            )
            .with_graceful_shutdown(async {
                let _ = shutdown_server.await;
            });

            // Start REST server
            info!("REST API listening on {}", addr);
            if let Err(e) = rest_server.await {
                tracing::error!("REST server error: {}", e);
            }
        });

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await.unwrap();
        info!("Received Ctrl+C, shutting down...");

        // Send shutdown signal
        let _ = shutdown_sender.send(());
        let _ = shutdown_recv.send(());

        server_handle.await.map_err(|e| {
            ServerError::Internal(format!("Server task error: {}", e))
        })?;

        Ok(())
    }

    /// Get the server address
    pub fn address(&self) -> SocketAddr {
        self.config.bind_address
    }

    /// Get database connection
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    database: DatabaseConnection,
    auth_service: AuthService,
    config: ServerConfig,
}

/// Health check handler
async fn health_check(State(state): State<AppState>) -> Result<impl IntoResponse, ServerError> {
    // Check database health
    let db_healthy = state.database.health_check().await.unwrap_or(false);

    let status = if db_healthy {
        HealthCheckStatus::Healthy
    } else {
        HealthCheckStatus::Unhealthy
    };

    let health = HealthStatus {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: if db_healthy {
            HealthCheckStatus::Healthy
        } else {
            HealthCheckStatus::Unhealthy
        },
        timestamp: Utc::now(),
        uptime_seconds: 0, // TODO: Calculate actual uptime
    };

    Ok((StatusCode::OK, Json(health)))
}

/// Login handler (placeholder)
async fn login() -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement actual login with username/password
    let response = AuthToken {
        token: "mock-jwt-token".to_string(),
        user_id: UserId::new_v4(),
        expires_at: Utc::now() + chrono::Duration::hours(24),
    };
    Ok((StatusCode::OK, Json(response)))
}

/// Refresh token handler (placeholder)
async fn refresh_token() -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement token refresh
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// Publish analysis handler
async fn publish_analysis(
    Path(project_id): Path<ProjectId>,
    State(state): State<AppState>,
    Json(request): Json<PublishRequest>,
) -> Result<impl IntoResponse, ServerError> {
    info!("Publishing analysis for project: {}", project_id);

    // TODO: Add authentication check
    // TODO: Validate project exists

    // Store the analysis
    let analysis_id = state
        .database
        .store_analysis(
            &project_id,
            &request.branch,
            &request.commit,
            &request.findings,
            &request.metadata,
        )
        .await?;

    // Calculate summary metrics
    let new_findings = request.findings.len() as u32;
    let resolved_findings = 0; // TODO: Calculate vs baseline
    let total_findings = new_findings;
    let trend = TrendDirection::Stable;

    let response = PublishResponse {
        analysis_id,
        new_findings,
        resolved_findings,
        total_findings,
        trend,
        summary_url: format!("/api/v1/analyses/{}", analysis_id),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get analysis handler
async fn get_analysis(
    Path((project_id, analysis_id)): Path<(ProjectId, AnalysisId)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement actual analysis retrieval
    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

/// Get diff analysis handler
async fn get_diff_analysis(
    Path(project_id): Path<ProjectId>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement diff analysis
    let diff = AnalysisDiff {
        base_analysis: None,
        head_analysis: None,
        new_findings: vec![],
        resolved_findings: vec![],
        severity_increased: vec![],
        severity_decreased: vec![],
        wont_fix_changed: vec![],
    };
    Ok((StatusCode::OK, Json(diff)))
}

/// Get trends handler
async fn get_trends(
    Path(project_id): Path<ProjectId>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let end = Utc::now();
    let start = end - chrono::Duration::days(30);

    let metrics = state
        .database
        .get_trend_metrics(&project_id, start, end)
        .await?;

    let trend = TrendMetrics {
        period: crate::modules::types::TimePeriod { start, end },
        total_findings: *metrics.get("total_findings").unwrap_or(&0),
        critical_findings: *metrics.get("critical_findings").unwrap_or(&0),
        major_findings: *metrics.get("major_findings").unwrap_or(&0),
        minor_findings: *metrics.get("minor_findings").unwrap_or(&0),
        info_findings: *metrics.get("info_findings").unwrap_or(&0),
        trend_percentage: 0.0, // TODO: Calculate
        by_severity: metrics.clone(),
        by_fact_type: HashMap::new(),
    };

    Ok((StatusCode::OK, Json(trend)))
}

/// List projects handler
async fn list_projects() -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement project listing
    Ok((StatusCode::OK, Json(vec![])))
}

/// Get project handler
async fn get_project(Path(project_id): Path<ProjectId>) -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement project retrieval
    Ok((StatusCode::OK, Json(serde_json::json!({}))))
}

/// Get baseline handler
async fn get_baseline(
    Path((project_id, branch)): Path<(ProjectId, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let baseline = state
        .database
        .get_latest_analysis(&project_id, &branch)
        .await?;

    Ok((StatusCode::OK, Json(baseline)))
}

/// Update baseline handler
async fn update_baseline(
    Path((project_id, branch)): Path<(ProjectId, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    // TODO: Implement baseline update
    Ok(StatusCode::OK)
}
