//! Provenance metadata for facts

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub extractor: ExtractorId,
    pub version: String,
    pub confidence: Confidence,
    pub extracted_at: DateTime<Utc>,
}

impl Provenance {
    pub fn new(
        extractor: ExtractorId,
        version: String,
        confidence: Confidence,
    ) -> Self {
        Self {
            extractor,
            version,
            confidence,
            extracted_at: Utc::now(),
        }
    }
}
