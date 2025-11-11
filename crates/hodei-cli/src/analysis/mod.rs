//! Analysis module
//!
//! This module provides utilities for analyzing code changes and supporting
//! incremental analysis capabilities.

pub mod cache_manager;
pub mod change_detector;
pub mod incremental;

pub use change_detector::{
    ChangeDetector, ChangeDetectorFactory, ChangeError, ChangeResult, ChangeType, ChangedFile,
    GitDetector, HashBasedDetector,
};

pub use cache_manager::{
    CacheConfig, CacheError, CacheKey, CacheManager, CacheResult, CacheStats, CacheValue,
};

pub use incremental::{
    AnalysisError, AnalysisResult, AnalysisStats, FactExtractor, IncrementalAnalyzer,
    IncrementalConfig, MockExtractor,
};
