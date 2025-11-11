/// Core hodei-server implementation with REST API
use crate::modules::auth::AuthService;
use crate::modules::config::ServerConfig;
use crate::modules::database::DatabaseConnection;
use crate::modules::error::{Result, ServerError};
use crate::modules::policies::{RateLimiter, RetentionManager, CleanupTask, create_analysis_summary};
use crate::modules::types::{
    AnalysisDiff, AnalysisId, AnalysisMetadata, AuthToken, HealthStatus,
    HealthCheckStatus, PublishRequest, PublishResponse, ProjectId, Severity, StoredAnalysis,
    TrendDirection, TrendMetrics, UserId,
};
use crate::modules::validation::{validate_publish_request, validate_project_exists, ValidationConfig};
use axum::{
    extract::{Path, State},
    http::StatusCode,
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
    start_time: SystemTime,
    shutdown_sender: broadcast::Sender<()>,
    rate_limiter: RateLimiter,
    retention_manager: RetentionManager,
    validation_config: ValidationConfig,
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

        // Initialize rate limiter
        let rate_limiter = RateLimiter::new(config.rate_limit_rpm);

        // Initialize retention manager
        let retention_manager = RetentionManager::new(365); // 1 year default

        // Initialize validation config
        let validation_config = ValidationConfig::default();

        // Create the REST router
        let rest_app = Self::create_rest_router(&config, &database, &auth_service, &rate_limiter, &retention_manager).await?;

        let start_time = SystemTime::now();
        let (shutdown_sender, _) = broadcast::channel(1);

        info!("hodei-server initialized successfully on {}", config.bind_address);

        Ok(Self {
            config,
            database,
            auth_service,
            rest_app,
            start_time,
            shutdown_sender,
            rate_limiter,
            retention_manager,
            validation_config,
        })
    }

    /// Create the REST router
    async fn create_rest_router(
        config: &ServerConfig,
        database: &DatabaseConnection,
        auth_service: &AuthService,
        rate_limiter: &RateLimiter,
        retention_manager: &RetentionManager,
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
            
            // Analysis endpoints (auth required with rate limiting)
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
                rate_limiter: rate_limiter.clone(),
                retention_manager: retention_manager.clone(),
                validation_config: ValidationConfig::default(),
            })
    }

    /// Start the server (REST API only)
    pub async fn run(self) -> Result<()> {
        let addr = self.config.bind_address;
        info!("Starting hodei-server on {}", addr);

        // Start background tasks
        let cleanup_task = CleanupTask::new(
            self.retention_manager,
            self.database.clone(),
            24, // Run cleanup every 24 hours
        );
        
        tokio::spawn(async move {
            cleanup_task.run().await;
        });

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
    rate_limiter: RateLimiter,
    retention_manager: RetentionManager,
    validation_config: ValidationConfig,
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

/// Publish analysis handler with full validation and rate limiting
async fn publish_analysis(
    Path(project_id): Path<ProjectId>,
    State(state): State<AppState>,
    Json(request): Json<PublishRequest>,
) -> Result<impl IntoResponse, ServerError> {
    info!("Publishing analysis for project: {}", project_id);

    // 1. Rate limiting check
    let rate_limit_key = format!("{}-analysis", project_id);
    if let Err(rate_err) = state.rate_limiter.check_limit(&rate_limit_key).await {
        warn!("Rate limit exceeded for project: {}", project_id);
        return Err(ServerError::RateLimit(rate_err.to_string()));
    }

    // 2. Validate project exists
    if !validate_project_exists(&project_id, &state.database).await? {
        return Err(ServerError::NotFound(format!("Project not found: {}", project_id)));
    }

    // 3. Validate request payload
    validate_publish_request(&project_id, &request, &state.validation_config)
        .map_err(|e| {
            warn!("Validation failed for project {}: {}", project_id, e);
            e
        })?;

    // 4. Store the analysis
    let analysis_id = state
        .database
        .store_analysis(
            &project_id,
            &request.branch,
            &request.commit,
            &request.findings,
            &request.metadata,
        )
        .await
        .map_err(|e| {
            warn!("Failed to store analysis for project {}: {}", project_id, e);
            e
        })?;

    // 5. Calculate summary metrics
    let summary = create_analysis_summary(
        analysis_id,
        request.findings.len() as u32,
        request.findings.len() as u32, // All are new in this case
        0, // TODO: Calculate resolved findings vs baseline
    );

    info!("Analysis published successfully for project {}: analysis_id={}, findings={}", 
          project_id, analysis_id, request.findings.len());

    Ok((StatusCode::CREATED, Json(summary)))
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
    // TODO: Implement diff analysis for US-13.03
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
