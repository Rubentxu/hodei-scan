use crate::domain::error::DomainResult;
/// Repository factory - Creates PostgreSQL repositories
///
/// This module provides factory functions to create repositories
/// for PostgreSQL database.
use crate::domain::ports::repositories::*;
use crate::infrastructure::database::postgres::{
    PostgresAnalysisRepository, PostgresBaselineRepository,
};

/// Create an analysis repository for PostgreSQL
pub async fn create_analysis_repository(
    database_url: &str,
) -> DomainResult<Box<dyn AnalysisRepository>> {
    let repo = PostgresAnalysisRepository::new(database_url)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to create PostgreSQL repository: {}",
                e
            ))
        })?;
    Ok(Box::new(repo) as Box<dyn AnalysisRepository>)
}

/// Create a baseline repository for PostgreSQL
pub async fn create_baseline_repository(
    database_url: &str,
) -> DomainResult<Box<dyn BaselineRepository>> {
    let repo = PostgresBaselineRepository::new(database_url)
        .await
        .map_err(|e| {
            crate::domain::error::DomainError::internal(&format!(
                "Failed to create PostgreSQL baseline repository: {}",
                e
            ))
        })?;
    Ok(Box::new(repo) as Box<dyn BaselineRepository>)
}
