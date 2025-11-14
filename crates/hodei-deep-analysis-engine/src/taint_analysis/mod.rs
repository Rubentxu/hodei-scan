//! Taint Analysis Engine - Datalog-based taint propagation
//!
//! This module provides datafrog-based taint propagation using Datalog rules
//! for detecting data flow vulnerabilities.

pub mod propagator;

// Re-exports
pub use crate::policy::{
    DataTag, SanitizerDefinition, SinkDefinition, SourceDefinition, TaintPolicy,
};
pub use propagator::{TaintAnalysisError, TaintFlow, TaintPropagator};
