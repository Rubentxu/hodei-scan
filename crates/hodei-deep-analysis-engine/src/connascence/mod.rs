//! Connascence Analysis - Architectural coupling detection
//!
//! This module provides detection of various types of connascence (coupling)
//! to identify architectural design issues and code smells.

pub mod algorithms;
pub mod analyzer;
pub mod findings;
pub mod types;

// Re-exports
pub use algorithms::{
    calculate_strength, detect_algorithm_connascence, detect_meaning_connascence,
    detect_name_connascence, detect_position_connascence, detect_type_connascence,
};
pub use analyzer::ConnascenceAnalyzer;
pub use findings::{CouplingFinding, EntityId};
pub use types::{ConnascenceType, Strength};
