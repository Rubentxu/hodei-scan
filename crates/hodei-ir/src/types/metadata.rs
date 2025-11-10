use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: ProjectPath,
    pub line: LineNumber,
    pub column: Option<ColumnNumber>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub extractor: ExtractorId,
    pub version: String,
    pub confidence: Confidence,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FactId(pub uuid::Uuid);
