/// WebSocket support for real-time dashboard updates - US-13.04
use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use crate::modules::error::Result;
use crate::modules::server::AppState;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tracing::{info, warn};

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
        metrics: TrendMetrics,
    },
    /// Diff calculation completed
    DiffCalculated {
        project_id: String,
        base_branch: String,
        head_branch: String,
        summary: DiffSummary,
    },
    /// Baseline updated
    BaselineUpdated {
        project_id: String,
        branch: String,
        analysis_id: String,
    },
    /// Server health status
    HealthStatus {
        status: String,
        uptime_seconds: u64,
    },
}

/// Extended trend metrics for dashboard
#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardTrendMetrics {
    pub period: TimePeriod,
    pub total_findings: u64,
    pub critical_findings: u64,
    pub major_findings: u64,
    pub minor_findings: u64,
    pub info_findings: u64,
    pub trend_percentage: f64,
    pub by_severity: std::collections::HashMap<String, u64>,
    pub by_fact_type: std::collections::HashMap<String, u64>,
    pub daily_breakdown: Vec<DailyFindingCount>,
    pub branch_comparison: Vec<BranchMetrics>,
    pub top_files: Vec<FileFindingCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyFindingCount {
    pub date: String,
    pub total: u32,
    pub critical: u32,
    pub major: u32,
    pub minor: u32,
    pub info: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchMetrics {
    pub branch: String,
    pub findings_count: u32,
    pub trend: String,
    pub last_analysis: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileFindingCount {
    pub file: String,
    pub findings_count: u32,
    pub critical_count: u32,
    pub major_count: u32,
}

/// WebSocket connection manager
pub struct WebSocketManager {
    /// Connected clients per project
    clients: Arc<dashmap::DashMap<String, dashmap::DashSet<uuid::Uuid>>>,
    /// All connected clients
    all_clients: Arc<dashmap::DashMap<uuid::Uuid, WebSocket>>,
    /// Database connection for metrics
    database: crate::modules::database::DatabaseConnection,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new(database: crate::modules::database::DatabaseConnection) -> Self {
        Self {
            clients: Arc::new(dashmap::DashMap::new()),
            all_clients: Arc::new(dashmap::DashMap::new()),
            database,
        }
    }

    /// Handle WebSocket connection upgrade
    pub async fn handle_connection(
        &self,
        ws: WebSocketUpgrade,
        State(state): State<AppState>,
        project_id: Option<String>,
    ) -> impl IntoResponse {
        let client_id = uuid::Uuid::new_v4();
        
        info!("New WebSocket connection: client_id={}, project_id={:?}", 
              client_id, project_id);

        ws.on_upgrade(move |socket| self.handle_socket(socket, client_id, project_id, state))
    }

    /// Handle individual WebSocket connection
    async fn handle_socket(
        &self,
        socket: WebSocket,
        client_id: uuid::Uuid,
        project_id: Option<String>,
        _state: AppState,
    ) {
        // Register client
        if let Some(project) = &project_id {
            let project_clients = self.clients.entry(project.clone()).or_insert_with(dashmap::DashSet::new);
            project_clients.insert(client_id);
        }
        
        self.all_clients.insert(client_id, socket);

        // Start sending periodic updates
        let mut interval_stream = IntervalStream::new(interval(Duration::from_secs(30))).filter_map(|_| async { 
            Some(())
        });

        // Track if client is still connected
        let (mut sender, mut receiver) = self.all_clients.get(&client_id).unwrap().split();
        
        // Send periodic health status
        let health_interval = interval(Duration::from_secs(10));
        tokio::pin!(health_interval);

        loop {
            tokio::select! {
                _ = health_interval.tick() => {
                    // Send health status update
                    if let Err(e) = self.send_to_client(
                        &client_id, 
                        &DashboardEvent::HealthStatus {
                            status: "healthy".to_string(),
                            uptime_seconds: 3600, // TODO: Calculate actual uptime
                        }
                    ).await {
                        warn!("Failed to send health update to client {}: {}", client_id, e);
                        break;
                    }
                }
                _ = interval_stream.next() => {
                    // Send periodic metrics updates
                    if let Some(project) = &project_id {
                        if let Err(e) = self.send_metrics_update(project).await {
                            warn!("Failed to send metrics update for project {}: {}", project, e);
                        }
                    }
                }
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(msg)) => {
                            // Handle incoming messages
                            if msg.is_text() {
                                info!("Received message from client {}: {:?}", client_id, msg);
                            }
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error for client {}: {}", client_id, e);
                            break;
                        }
                        None => {
                            info!("Client {} disconnected", client_id);
                            break;
                        }
                    }
                }
            }
        }

        // Clean up client
        self.remove_client(client_id, project_id.as_deref());
    }

    /// Send event to specific client
    pub async fn send_to_client<T: Serialize>(
        &self,
        client_id: &uuid::Uuid,
        event: &T,
    ) -> Result<()> {
        if let Some(mut socket) = self.all_clients.get_mut(client_id) {
            let message = serde_json::to_string(event)
                .map_err(|e| crate::modules::error::ServerError::Serialization(e))?;
            
            socket.send(axum::extract::ws::Message::Text(message))
                .await
                .map_err(|e| crate::modules::error::ServerError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Broadcast event to all clients of a project
    pub async fn broadcast_to_project<T: Serialize>(
        &self,
        project_id: &str,
        event: &T,
    ) -> Result<()> {
        if let Some(project_clients) = self.clients.get(project_id) {
            let message = serde_json::to_string(event)
                .map_err(|e| crate::modules::error::ServerError::Serialization(e))?;
            
            for client_id in project_clients.value().iter() {
                if let Some(mut socket) = self.all_clients.get_mut(client_id) {
                    if let Err(e) = socket.send(axum::extract::ws::Message::Text(message.clone())).await {
                        warn!("Failed to send to client {}: {}", client_id, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Send metrics update for a project
    async fn send_metrics_update(&self, project_id: &str) -> Result<()> {
        // Get enhanced trend metrics
        let metrics = self.get_enhanced_metrics(project_id).await?;
        
        self.broadcast_to_project(
            project_id,
            &DashboardEvent::TrendUpdated {
                project_id: project_id.to_string(),
                metrics,
            }
        ).await?;
        
        Ok(())
    }

    /// Get enhanced metrics for dashboard
    async fn get_enhanced_metrics(&self, project_id: &str) -> Result<DashboardTrendMetrics> {
        use chrono::Utc;

        let end = Utc::now();
        let start = end - chrono::Duration::days(30);

        // Get basic metrics
        let basic_metrics = self.database
            .get_trend_metrics(project_id, start, end)
            .await?;

        // Generate daily breakdown (mock for now)
        let daily_breakdown = self.generate_daily_breakdown(start, end);

        // Get branch comparison (mock for now)
        let branch_comparison = self.get_branch_comparison(project_id).await?;

        // Get top files (mock for now)
        let top_files = self.get_top_finding_files(project_id).await?;

        Ok(DashboardTrendMetrics {
            period: crate::modules::types::TimePeriod { start, end },
            total_findings: *basic_metrics.get("total_findings").unwrap_or(&0),
            critical_findings: *basic_metrics.get("critical_findings").unwrap_or(&0),
            major_findings: *basic_metrics.get("major_findings").unwrap_or(&0),
            minor_findings: *basic_metrics.get("minor_findings").unwrap_or(&0),
            info_findings: *basic_metrics.get("info_findings").unwrap_or(&0),
            trend_percentage: 0.0, // TODO: Calculate actual trend
            by_severity: basic_metrics,
            by_fact_type: std::collections::HashMap::new(),
            daily_breakdown,
            branch_comparison,
            top_files,
        })
    }

    /// Generate daily breakdown (mock implementation)
    fn generate_daily_breakdown(&self, start: chrono::DateTime<Utc>, end: chrono::DateTime<Utc>) -> Vec<DailyFindingCount> {
        let mut breakdown = vec![];
        let mut current = start.date().and_hms(0, 0, 0);
        let end_date = end.date().and_hms(0, 0, 0);

        while current <= end_date {
            breakdown.push(DailyFindingCount {
                date: current.format("%Y-%m-%d").to_string(),
                total: (current.day() * 10) as u32, // Mock data
                critical: (current.day() % 5) as u32,
                major: (current.day() % 10) as u32,
                minor: (current.day() % 15) as u32,
                info: (current.day() % 20) as u32,
            });
            current = current + chrono::Duration::days(1);
        }

        breakdown
    }

    /// Get branch comparison (mock implementation)
    async fn get_branch_comparison(&self, project_id: &str) -> Result<Vec<BranchMetrics>> {
        // TODO: Query database for actual branch metrics
        Ok(vec![
            BranchMetrics {
                branch: "main".to_string(),
                findings_count: 150,
                trend: "improving".to_string(),
                last_analysis: Utc::now().format("%Y-%m-%d %H:%M").to_string(),
            },
            BranchMetrics {
                branch: "develop".to_string(),
                findings_count: 120,
                trend: "stable".to_string(),
                last_analysis: (Utc::now() - chrono::Duration::hours(2)).format("%Y-%m-%d %H:%M").to_string(),
            },
        ])
    }

    /// Get top files with findings (mock implementation)
    async fn get_top_finding_files(&self, project_id: &str) -> Result<Vec<FileFindingCount>> {
        // TODO: Query database for actual file finding counts
        Ok(vec![
            FileFindingCount {
                file: "src/auth/security.rs".to_string(),
                findings_count: 25,
                critical_count: 5,
                major_count: 10,
            },
            FileFindingCount {
                file: "src/api/validation.rs".to_string(),
                findings_count: 18,
                critical_count: 3,
                major_count: 8,
            },
        ])
    }

    /// Remove client from all tracking
    fn remove_client(&self, client_id: uuid::Uuid, project_id: Option<&str>) {
        // Remove from project tracking
        if let Some(project) = project_id {
            if let Some(mut project_clients) = self.clients.get_mut(project) {
                project_clients.remove(&client_id);
                // Clean up empty project entries
                if project_clients.is_empty() {
                    drop(project_clients);
                    self.clients.remove(project);
                }
            }
        }

        // Remove from global tracking
        self.all_clients.remove(&client_id);

        info!("Removed client: {}", client_id);
    }

    /// Get number of connected clients
    pub fn client_count(&self) -> usize {
        self.all_clients.len()
    }

    /// Get number of clients for a project
    pub fn project_client_count(&self, project_id: &str) -> usize {
        self.clients.get(project_id)
            .map(|entry| entry.len())
            .unwrap_or(0)
    }
}

/// Helper function to get dashboard trend metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: chrono::DateTime<Utc>,
    pub end: chrono::DateTime<Utc>,
}

/// Helper function to convert DiffSummary
impl From<crate::modules::diff::DiffSummary> for DiffSummary {
    fn from(summary: crate::modules::diff::DiffSummary) -> Self {
        Self {
            total_changes: summary.total_changes,
            new_findings_count: summary.new_findings_count,
            resolved_findings_count: summary.resolved_findings_count,
            severity_increased_count: summary.severity_increased_count,
            severity_decreased_count: summary.severity_decreased_count,
            net_change: summary.net_change,
            severity_score: summary.severity_score,
            trend: match summary.trend {
                crate::modules::types::TrendDirection::Improving => "improving".to_string(),
                crate::modules::types::TrendDirection::Degrading => "degrading".to_string(),
                crate::modules::types::TrendDirection::Stable => "stable".to_string(),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub new_findings_count: usize,
    pub resolved_findings_count: usize,
    pub severity_increased_count: usize,
    pub severity_decreased_count: usize,
    pub net_change: isize,
    pub severity_score: isize,
    pub trend: String,
}
