//! hodei-test
//!
//! Rule testing framework for hodei-scan
//!
//! This crate provides:
//! - YAML-based test file format
//! - Rule testing runner
//! - Snapshot testing
//! - CI integration

pub mod domain;
pub mod application;
pub mod infrastructure;

/// Re-export main types
pub use application::test_runner::HodeiTestRunner;
pub use application::snapshot::SnapshotManager;
pub use domain::models::{TestConfig, TestCase, TestResults};
