//! Runtime Sandboxing
//!
//! This module provides sandboxing capabilities to prevent resource exhaustion
//! and limit execution time/memory usage during rule evaluation.

use crate::ast::*;
use crate::error::ParseResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for execution sandbox
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum execution time per rule (in milliseconds)
    pub time_limit_ms: u64,
    /// Maximum memory usage (in bytes)
    pub memory_limit_bytes: usize,
    /// Maximum number of iterations in loops
    pub max_iterations: usize,
    /// Maximum call depth for nested function calls
    pub max_call_depth: usize,
    /// Maximum number of facts that can be scanned
    pub max_facts_scanned: usize,
    /// Maximum output size (in bytes)
    pub max_output_size: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            time_limit_ms: 5000,                   // 5 seconds
            memory_limit_bytes: 100 * 1024 * 1024, // 100 MB
            max_iterations: 1_000_000,
            max_call_depth: 100,
            max_facts_scanned: 1_000_000,
            max_output_size: 1024 * 1024, // 1 MB
        }
    }
}

/// Statistics about sandbox execution
#[derive(Debug, Clone, Default)]
pub struct SandboxStats {
    pub execution_time_ms: u64,
    pub memory_used_bytes: usize,
    pub facts_scanned: usize,
    pub iterations: usize,
    pub call_depth: u32,
    pub timed_out: bool,
    pub memory_exceeded: bool,
}

/// Result of sandboxed execution
#[derive(Debug, Clone)]
pub struct SandboxResult<T> {
    pub result: ParseResult<T>,
    pub stats: SandboxStats,
}

/// Sandbox for executing DSL rules with resource limits
pub struct ExecutionSandbox {
    config: SandboxConfig,
    stats: Arc<Mutex<SandboxStats>>,
    start_time: Option<Instant>,
}

impl ExecutionSandbox {
    /// Create a new sandbox with default configuration
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// Create a sandbox with custom configuration
    pub fn with_config(config: SandboxConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(SandboxStats::default())),
            start_time: None,
        }
    }

    /// Execute a closure within the sandbox
    pub fn execute<T, F>(&mut self, f: F) -> SandboxResult<T>
    where
        F: FnOnce() -> ParseResult<T>,
    {
        self.start_time = Some(Instant::now());

        // Set up monitoring thread
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();
        let monitor_handle = thread::spawn(move || {
            let mut last_stats = SandboxStats::default();
            loop {
                thread::sleep(Duration::from_millis(10));

                let mut stats = stats.lock().unwrap();
                stats.execution_time_ms = last_stats.execution_time_ms + 10;

                if stats.execution_time_ms > config.time_limit_ms {
                    stats.timed_out = true;
                    break;
                }

                // Check memory (simplified check)
                // In a real implementation, we'd use getrusage or similar
                if stats.memory_used_bytes > config.memory_limit_bytes {
                    stats.memory_exceeded = true;
                    break;
                }

                last_stats = stats.clone();
            }
        });

        // Execute the function
        let result = f();

        // Wait for monitoring to finish
        let _ = monitor_handle.join();

        let stats = {
            let guard = self.stats.lock().unwrap();
            guard.clone()
        };

        SandboxResult { result, stats }
    }

    /// Update statistics during execution
    pub fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut SandboxStats),
    {
        if let Ok(mut stats) = self.stats.lock() {
            updater(&mut stats);
        }
    }

    /// Check if execution should be aborted
    pub fn should_abort(&self) -> bool {
        if let Ok(stats) = self.stats.lock() {
            stats.timed_out || stats.memory_exceeded
        } else {
            false
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> SandboxStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            SandboxStats::default()
        }
    }
}

impl Default for ExecutionSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule complexity analyzer
pub struct RuleComplexityAnalyzer {
    config: SandboxConfig,
}

impl RuleComplexityAnalyzer {
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    pub fn with_config(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Estimate the complexity of a rule
    pub fn estimate_complexity(&self, rule: &RuleDef) -> ParseResult<ComplexityEstimate> {
        let mut estimate = ComplexityEstimate::default();

        // Count patterns
        estimate.pattern_count = rule.match_block.patterns.len();

        // Count expressions in where clause
        if let Some(where_clause) = &rule.match_block.where_clause {
            estimate.expression_count = self.count_expressions(where_clause);
        }

        // Estimate based on patterns and expressions
        estimate.estimated_facts_scanned = self.estimate_facts_scanned(rule);
        estimate.estimated_iterations = self.estimate_iterations(rule);
        estimate.estimated_call_depth = self.estimate_call_depth(rule);
        estimate.complexity_score = self.calculate_complexity_score(&estimate);

        // Check if rule exceeds limits
        estimate.exceeds_limits = self.check_limits(&estimate)?;

        Ok(estimate)
    }

    /// Count expressions in an expression tree
    fn count_expressions(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) => 1,
            Expr::Path(_) => 1,
            Expr::FunctionCall { args, .. } => {
                1 + args
                    .iter()
                    .map(|a| self.count_expressions(a))
                    .sum::<usize>()
            }
            Expr::Binary { left, right, .. } => {
                1 + self.count_expressions(left) + self.count_expressions(right)
            }
        }
    }

    /// Estimate number of facts that will be scanned
    fn estimate_facts_scanned(&self, rule: &RuleDef) -> usize {
        let mut estimate = 1;

        for pattern in &rule.match_block.patterns {
            // Pattern is a struct with fact_type field
            estimate *= 10000;
        }

        if rule.match_block.where_clause.is_some() {
            // Where clause filters results
            estimate = (estimate as f64 * 0.1) as usize;
        }

        estimate
    }

    /// Estimate number of iterations
    fn estimate_iterations(&self, rule: &RuleDef) -> usize {
        let mut estimate = 1;

        for pattern in &rule.match_block.patterns {
            // Pattern is a struct
            estimate *= 1000;
        }

        estimate
    }

    /// Estimate maximum call depth
    fn estimate_call_depth(&self, rule: &RuleDef) -> u32 {
        let mut depth = 1;

        if rule.match_block.where_clause.is_some() {
            depth += 2;
        }

        depth
    }

    /// Calculate overall complexity score (0-100)
    fn calculate_complexity_score(&self, estimate: &ComplexityEstimate) -> u32 {
        let pattern_score = (estimate.pattern_count as f32 * 10.0).min(30.0);
        let expression_score = (estimate.expression_count as f32 * 0.5).min(20.0);
        let fact_score = (estimate.estimated_facts_scanned as f32 / 10000.0).min(25.0);
        let iteration_score = (estimate.estimated_iterations as f32 / 10000.0).min(15.0);
        let depth_score = (estimate.estimated_call_depth as f32 * 2.0).min(10.0);

        (pattern_score + expression_score + fact_score + iteration_score + depth_score) as u32
    }

    /// Check if estimate exceeds configured limits
    fn check_limits(&self, estimate: &ComplexityEstimate) -> ParseResult<bool> {
        let mut exceeds = false;

        if estimate.estimated_iterations > self.config.max_iterations {
            exceeds = true;
        }

        if estimate.estimated_call_depth > self.config.max_call_depth as u32 {
            exceeds = true;
        }

        if estimate.estimated_facts_scanned > self.config.max_facts_scanned {
            exceeds = true;
        }

        Ok(exceeds)
    }
}

/// Complexity estimate for a rule
#[derive(Debug, Clone, Default)]
pub struct ComplexityEstimate {
    pub pattern_count: usize,
    pub expression_count: usize,
    pub estimated_facts_scanned: usize,
    pub estimated_iterations: usize,
    pub estimated_call_depth: u32,
    pub complexity_score: u32, // 0-100
    pub exceeds_limits: bool,
}

impl ComplexityEstimate {
    pub fn complexity_level(&self) -> ComplexityLevel {
        match self.complexity_score {
            0..=20 => ComplexityLevel::Low,
            21..=50 => ComplexityLevel::Medium,
            51..=80 => ComplexityLevel::High,
            _ => ComplexityLevel::VeryHigh,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl std::fmt::Display for ComplexityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexityLevel::Low => write!(f, "Low"),
            ComplexityLevel::Medium => write!(f, "Medium"),
            ComplexityLevel::High => write!(f, "High"),
            ComplexityLevel::VeryHigh => write!(f, "Very High"),
        }
    }
}
