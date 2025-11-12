/// WebSocket support for real-time dashboard updates - US-13.04
use axum::{
    extract::{
        State,
        ws::{WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tracing::{info, warn};

use crate::modules::server::AppState;

/// WebSocket event types for dashboard
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DashboardEvent {
    /// New analysis published
    AnalysisPublished {
        project_id: String,
        analysis_id: String,
        findings_count: u32,
        timestamp: String,
    },
    /// Trend metrics update
    TrendUpdated {
        project_id: String,
        metrics: crate::modules::types::TrendMetrics,
    },
    /// Diff calculation completed
    DiffCalculated {
        project_id: String,
        base_branch: String,
        head_branch: String,
        summary: crate::modules::diff::DiffSummary,
    },
    /// Baseline updated
    BaselineUpdated {
        project_id: String,
        branch: String,
        analysis_id: String,
    },
    /// Server health status
    HealthStatus { status: String, uptime_seconds: u64 },
}

/// WebSocket connection manager
#[derive(Clone)]
pub struct WebSocketManager {
    /// Database connection for metrics
    database: crate::modules::database::DatabaseConnection,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new(database: crate::modules::database::DatabaseConnection) -> Self {
        Self { database }
    }

    /// Handle WebSocket connection
    pub fn handle_connection(
        self: Arc<Self>,
        ws: WebSocketUpgrade,
        State(_state): State<AppState>,
        project_id: Option<String>,
    ) -> impl IntoResponse {
        let client_id = uuid::Uuid::new_v4();

        info!(
            "New WebSocket connection: client_id={}, project_id={:?}",
            client_id, project_id
        );

        let manager = Arc::clone(&self);
        ws.on_upgrade(move |socket| manager.handle_socket(socket, client_id, project_id))
    }

    /// Handle individual WebSocket connection
    pub async fn handle_socket(
        self: Arc<Self>,
        socket: WebSocket,
        _client_id: uuid::Uuid,
        project_id: Option<String>,
    ) {
        // Send periodic health status
        let health_interval = interval(Duration::from_secs(10));
        tokio::pin!(health_interval);

        // Start sending periodic updates
        let mut interval_stream = IntervalStream::new(interval(Duration::from_secs(30)));

        let (mut sender, mut receiver) = socket.split();

        loop {
            tokio::select! {
                _ = health_interval.tick() => {
                    // Send health status update
                    let event = DashboardEvent::HealthStatus {
                        status: "healthy".to_string(),
                        uptime_seconds: 3600, // TODO: Calculate actual uptime
                    };

                    if let Err(e) = sender.send(axum::extract::ws::Message::Text(
                        serde_json::to_string(&event)
                            .unwrap_or_default()
                            .into()
                    )).await {
                        warn!("Failed to send health update: {}", e);
                        break;
                    }
                }
                _ = tokio_stream::StreamExt::next(&mut interval_stream) => {
                    // Send periodic metrics updates (placeholder)
                    if let Some(project) = &project_id {
                        info!("Would send metrics update for project {}", project);
                    }
                }
                msg = futures_util::StreamExt::next(&mut receiver) => {
                    match msg {
                        Some(Ok(msg)) => {
                            // Handle incoming messages
                            match msg {
                                axum::extract::ws::Message::Text(_) => {
                                    info!("Received WebSocket message");
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket disconnected");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Broadcast event (simplified - no-op for now)
    pub async fn broadcast_to_project<T: Serialize>(
        &self,
        _project_id: &str,
        _event: &T,
    ) -> Result<(), crate::modules::error::ServerError> {
        // TODO: Implement broadcasting
        Ok(())
    }
}
