/// gRPC server implementation for hodei-server
mod proto {
    tonic::include_proto!("hodei.server.v1");
}

use crate::modules::error::{Result, ServerError};
use crate::modules::types::{
    AnalysisId, Finding, FindingLocation, FindingStatus, PublishRequest, Severity, StoredAnalysis,
    TrendDirection as DomainTrendDirection,
};
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tracing::{info, warn};

use proto::{
    analysis::Metadata as ProtoMetadata, 
    analysis_server::{Analysis, AnalysisServer},
    health_server::{Health, HealthServer},
    HealthCheckRequest, HealthCheckResponse, HealthStatus as ProtoHealthStatus,
    Finding as ProtoFinding, FindingLocation as ProtoFindingLocation,
    Severity as ProtoSeverity, PublishAnalysisRequest, PublishAnalysisResponse,
    GetAnalysisRequest, GetAnalysisResponse, GetBaselineRequest, GetBaselineResponse,
    UpdateBaselineRequest, UpdateBaselineResponse, StreamNotificationsRequest,
    NotificationEvent, EventType, TrendDirection as ProtoTrendDirection,
    Analysis as ProtoAnalysis,
};

use super::database::DatabaseConnection;

/// gRPC server implementation
pub struct HodeiGrpcServer {
    database: DatabaseConnection,
}

impl HodeiGrpcServer {
    pub fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }
}

#[tonic::async_trait]
impl Analysis for HodeiGrpcServer {
    type PublishAnalysisStream = ReceiverStream<Result<PublishAnalysisResponse, Status>>;
    type StreamNotificationsStream = ReceiverStream<Result<NotificationEvent, Status>>;

    async fn publish_analysis(
        &self,
        request: Request<Streaming<PublishAnalysisRequest>>,
    ) -> Result<Response<Self::PublishAnalysisStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(4);
        let database = self.database.clone();

        tokio::spawn(async move {
            while let Some(request_result) = stream.next().await {
                match request_result {
                    Ok(request) => {
                        match process_publish_request(&database, &request).await {
                            Ok(response) => {
                                if let Err(e) = tx.send(Ok(response)).await {
                                    warn!("Failed to send response: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                if let Err(send_err) = tx.send(Err(Status::internal(e.to_string()))).await {
                                    warn!("Failed to send error: {}", send_err);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_analysis(
        &self,
        request: Request<GetAnalysisRequest>,
    ) -> Result<Response<GetAnalysisResponse>, Status> {
        let req = request.into_inner();
        let analysis_id = AnalysisId::parse_str(&req.analysis_id)
            .map_err(|_| Status::invalid_argument("Invalid analysis ID"))?;

        // TODO: Implement actual retrieval
        let response = GetAnalysisResponse {
            analysis: None, // Return actual analysis
        };

        Ok(Response::new(response))
    }

    async fn get_baseline(
        &self,
        request: Request<GetBaselineRequest>,
    ) -> Result<Response<GetBaselineResponse>, Status> {
        let req = request.into_inner();
        
        let baseline = self.database
            .get_latest_analysis(&req.project_id, &req.branch)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_baseline = baseline.as_ref().map(convert_analysis_to_proto);

        Ok(Response::new(GetBaselineResponse {
            baseline: proto_baseline,
        }))
    }

    async fn update_baseline(
        &self,
        request: Request<UpdateBaselineRequest>,
    ) -> Result<Response<UpdateBaselineResponse>, Status> {
        let req = request.into_inner();
        
        // TODO: Implement baseline update logic
        let response = UpdateBaselineResponse {
            success: true,
            message: "Baseline updated successfully".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn stream_notifications(
        &self,
        request: Request<StreamNotificationsRequest>,
    ) -> Result<Response<Self::StreamNotificationsStream>, Status> {
        let _req = request.into_inner();
        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            // TODO: Implement real-time notifications
            let notification = NotificationEvent {
                event_type: EventType::AnalysisPublished as i32,
                project_id: "test-project".to_string(),
                analysis_id: "test-analysis".to_string(),
                message: "Analysis published successfully".to_string(),
                timestamp: Utc::now().timestamp(),
            };

            let _ = tx.send(Ok(notification)).await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tonic::async_trait]
impl Health for HodeiGrpcServer {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let db_healthy = self.database
            .health_check()
            .await
            .unwrap_or(false);

        let status = if db_healthy {
            ProtoHealthStatus::Healthy
        } else {
            ProtoHealthStatus::Unhealthy
        };

        let response = HealthCheckResponse {
            status: status as i32,
            version: env!("CARGO_PKG_VERSION").to_string(),
            message: if db_healthy { "Healthy" } else { "Database unreachable" }.to_string(),
        };

        Ok(Response::new(response))
    }
}

/// Helper function to process a publish analysis request
async fn process_publish_request(
    database: &DatabaseConnection,
    request: &PublishAnalysisRequest,
) -> Result<PublishAnalysisResponse, ServerError> {
    info!("Processing publish analysis request for project: {}", request.project_id);

    // Convert request to domain types
    let findings = request.findings
        .iter()
        .map(convert_finding_from_proto)
        .collect::<Result<Vec<_>, _>>()?;

    let metadata = convert_metadata_from_proto(&request.metadata);

    // Store analysis
    let analysis_id = database
        .store_analysis(
            &request.project_id,
            &request.branch,
            &request.commit,
            &findings,
            &metadata,
        )
        .await?;

    // Convert trend direction
    let trend = convert_trend_direction_proto(DomainTrendDirection::Stable);

    Ok(PublishAnalysisResponse {
        analysis_id: analysis_id.to_string(),
        new_findings: findings.len() as u32,
        resolved_findings: 0, // TODO: Calculate vs baseline
        total_findings: findings.len() as u32,
        trend: trend as i32,
    })
}

/// Conversion functions between protobuf and domain types

fn convert_finding_from_proto(finding: &ProtoFinding) -> Result<Finding, Status> {
    Ok(Finding {
        fact_type: finding.fact_type.clone(),
        severity: convert_severity_from_proto(finding.severity),
        location: FindingLocation {
            file: finding.location.file.clone(),
            line: finding.location.line,
            column: finding.location.column,
            end_line: if finding.location.end_line > 0 {
                Some(finding.location.end_line)
            } else {
                None
            },
            end_column: if finding.location.end_column > 0 {
                Some(finding.location.end_column)
            } else {
                None
            },
        },
        message: finding.message.clone(),
        metadata: if finding.metadata.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&finding.metadata).unwrap_or_default())
        },
        tags: finding.tags.clone(),
        fingerprint: finding.fingerprint.clone(),
    })
}

fn convert_severity_from_proto(severity: i32) -> Severity {
    match severity {
        0 => Severity::Critical,
        1 => Severity::Major,
        2 => Severity::Minor,
        _ => Severity::Info,
    }
}

fn convert_metadata_from_proto(metadata: &Option<ProtoMetadata>) -> crate::modules::types::AnalysisMetadata {
    if let Some(meta) = metadata {
        crate::modules::types::AnalysisMetadata {
            build_url: if meta.build_url.is_empty() { None } else { Some(meta.build_url.clone()) },
            author: if meta.author.is_empty() { None } else { Some(meta.author.clone()) },
            ci_run_id: if meta.ci_run_id.is_empty() { None } else { Some(meta.ci_run_id.clone()) },
            scan_duration_ms: if meta.scan_duration_ms > 0 { Some(meta.scan_duration_ms) } else { None },
            rule_version: if meta.rule_version.is_empty() { None } else { Some(meta.rule_version.clone()) },
        }
    } else {
        crate::modules::types::AnalysisMetadata {
            build_url: None,
            author: None,
            ci_run_id: None,
            scan_duration_ms: None,
            rule_version: None,
        }
    }
}

fn convert_analysis_to_proto(analysis: &StoredAnalysis) -> ProtoAnalysis {
    ProtoAnalysis {
        id: analysis.id.to_string(),
        project_id: analysis.project_id.clone(),
        branch: analysis.branch.clone(),
        commit: analysis.commit.clone(),
        timestamp: Some(Timestamp::from(analysis.timestamp)).encode_to_vec(),
        findings_count: analysis.findings_count,
        metadata: Some(convert_metadata_to_proto(&analysis.metadata)),
        created_at: Some(Timestamp::from(analysis.created_at)).encode_to_vec(),
    }
}

fn convert_metadata_to_proto(metadata: &crate::modules::types::AnalysisMetadata) -> ProtoMetadata {
    ProtoMetadata {
        build_url: metadata.build_url.as_ref().unwrap_or(&"".to_string()).clone(),
        author: metadata.author.as_ref().unwrap_or(&"".to_string()).clone(),
        ci_run_id: metadata.ci_run_id.as_ref().unwrap_or(&"".to_string()).clone(),
        scan_duration_ms: metadata.scan_duration_ms.unwrap_or(0),
        rule_version: metadata.rule_version.as_ref().unwrap_or(&"".to_string()).clone(),
    }
}

fn convert_trend_direction_proto(direction: DomainTrendDirection) -> ProtoTrendDirection {
    match direction {
        DomainTrendDirection::Improving => ProtoTrendDirection::Improving,
        DomainTrendDirection::Degrading => ProtoTrendDirection::Degrading,
        DomainTrendDirection::Stable => ProtoTrendDirection::Stable,
    }
}
