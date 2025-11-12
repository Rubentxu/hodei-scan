//! End-to-End tests for complete workflows
//!
//! These tests verify the integration of all EPIC-14 tools

mod full_workflow;
mod ir_workflow;
mod lsp_workflow;
mod quality_gates;
mod real_world_tests;
mod test_workflow;

pub use full_workflow::*;
pub use ir_workflow::*;
pub use lsp_workflow::*;
pub use quality_gates::*;
pub use real_world_tests::*;
pub use test_workflow::*;
