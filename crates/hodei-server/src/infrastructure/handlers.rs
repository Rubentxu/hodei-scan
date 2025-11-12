/// REST API Handlers - Using hexagonal architecture
///
/// These handlers are part of the infrastructure layer and use the
/// application layer's use cases.
use crate::application::usecases::*;
use crate::domain::models::*;
use crate::domain::ports::{AnalysisRepository, BaselineRepository};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{info, warn};

/// Application state with repositories and use cases
#[derive(Clone)]
pub struct AppState {
    pub analysis_repo: Arc<dyn AnalysisRepository>,
    pub baseline_repo: Arc<dyn BaselineRepository>,
    pub publish_usecase: Arc<PublishAnalysisUseCase>,
    pub update_baseline_usecase: Arc<UpdateBaselineUseCase>,
}

/// Publish analysis handler - uses hexagonal architecture
pub async fn publish_analysis(
    State(state): State<AppState>,
    Json(request): Json<PublishRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Publishing analysis: project_id={}, branch={}, findings={}",
        request.project_id.as_str(),
        request.branch,
        request.findings.len()
    );

    match state.publish_usecase.execute(request).await {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(e) => {
            warn!("Failed to publish analysis: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update baseline handler - uses hexagonal architecture
pub async fn update_baseline(
    State(state): State<AppState>,
    Json(update): Json<BaselineStatusUpdate>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Updating baseline: project_id={}, branch={}, fingerprint={}",
        update.project_id.as_str(),
        update.branch,
        update.finding_fingerprint
    );

    match state.update_baseline_usecase.execute(update).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            warn!("Failed to update baseline: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get analysis handler
pub async fn get_analysis(
    State(state): State<AppState>,
    Path((project_id, analysis_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    let project_id = ProjectId(project_id);
    let analysis_id =
        AnalysisId(uuid::Uuid::parse_str(&analysis_id).map_err(|_| StatusCode::BAD_REQUEST)?);

    match state.analysis_repo.get_analysis(&analysis_id).await {
        Ok(Some(analysis)) => Ok((StatusCode::OK, Json(analysis))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get project analyses handler
pub async fn get_project_analyses(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    limit: Option<u32>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = limit.unwrap_or(10);
    let project_id = ProjectId(project_id);

    match state
        .analysis_repo
        .get_project_analyses(&project_id, limit)
        .await
    {
        Ok(analyses) => Ok((StatusCode::OK, Json(analyses))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
