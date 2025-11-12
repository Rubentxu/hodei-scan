//! Cache Manager for Incremental Analysis
//!
//! This module provides high-performance caching capabilities using RocksDB
//! to store and retrieve analysis results based on file hashes and timestamps.

use hodei_ir::Fact;
use rocksdb::{DB, LogLevel, Options, SliceTransform, WriteBatch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Error type for cache operations
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Custom error: {message}")]
    Custom { message: String },
}

impl CacheError {
    /// Create a new custom error
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }
}

/// Cache key structure combining file path, hash, and timestamp
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Absolute path to the file
    pub path: PathBuf,
    /// SHA-256 hash of file contents
    pub file_hash: String,
    /// Last modified timestamp
    pub modified_at: u64,
}

impl CacheKey {
    /// Create a new cache key from a file
    pub fn from_file(path: &Path, file_hash: String) -> Self {
        let modified_at = path
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            path: path.to_path_buf(),
            file_hash,
            modified_at,
        }
    }

    /// Serialize the key to bytes for RocksDB storage
    pub fn to_bytes(&self) -> CacheResult<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    /// Deserialize key from bytes
    pub fn from_bytes(bytes: &[u8]) -> CacheResult<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}

/// Cache value structure storing facts and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheValue {
    /// The cached facts
    pub facts: Vec<Fact>,
    /// When this entry was created
    pub created_at: u64,
    /// When this entry will expire
    pub expires_at: Option<u64>,
    /// Number of times this entry was accessed
    pub access_count: u64,
}

impl CacheValue {
    /// Create a new cache value
    pub fn new(facts: Vec<Fact>, ttl_seconds: Option<u64>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let expires_at = ttl_seconds.map(|ttl| now + ttl);

        Self {
            facts,
            created_at: now,
            expires_at,
            access_count: 0,
        }
    }

    /// Check if the cache entry is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Increment access count
    pub fn access(&mut self) {
        self.access_count += 1;
    }

    /// Serialize the value to bytes
    pub fn to_bytes(&self) -> CacheResult<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    /// Deserialize value from bytes
    pub fn from_bytes(bytes: &[u8]) -> CacheResult<Self> {
        Ok(bincode::deserialize(bytes)?)
    }
}

/// Statistics tracking for cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of entries stored
    pub entries_stored: u64,
    /// Number of entries evicted
    pub entries_evicted: u64,
    /// Total bytes stored
    pub bytes_stored: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

impl CacheStats {
    /// Record a cache hit
    pub fn record_hit(&mut self) {
        self.hits += 1;
        self.update_hit_rate();
    }

    /// Record a cache miss
    pub fn record_miss(&mut self) {
        self.misses += 1;
        self.update_hit_rate();
    }

    /// Record a new entry stored
    pub fn record_stored(&mut self, bytes: u64) {
        self.entries_stored += 1;
        self.bytes_stored += bytes;
    }

    /// Record an entry evicted
    pub fn record_evicted(&mut self) {
        self.entries_evicted += 1;
    }

    /// Update the hit rate
    fn update_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hit_rate = self.hits as f64 / total as f64;
        } else {
            self.hit_rate = 0.0;
        }
    }
}

/// Configuration for the cache manager
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for cache entries in seconds (None = no expiration)
    pub ttl_seconds: Option<u64>,
    /// Maximum number of cache entries (None = unlimited)
    pub max_entries: Option<usize>,
    /// RocksDB write buffer size in MB
    pub write_buffer_size_mb: usize,
    /// RocksDB max write buffer number
    pub max_write_buffers: usize,
    /// Enable compression
    pub compression: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_seconds: Some(7 * 24 * 60 * 60), // 7 days
            max_entries: Some(1_000_000),        // 1 million entries
            write_buffer_size_mb: 64,
            max_write_buffers: 3,
            compression: true,
            cache_dir: PathBuf::from(".hodei-cache"),
        }
    }
}

/// High-performance cache manager using RocksDB
pub struct CacheManager {
    /// The RocksDB instance
    db: Arc<DB>,
    /// Cache configuration
    config: CacheConfig,
    /// Statistics tracking
    stats: Arc<std::sync::Mutex<CacheStats>>,
}

impl CacheManager {
    /// Create a new cache manager with default configuration
    pub fn new() -> CacheResult<Self> {
        let config = CacheConfig::default();
        Self::with_config(config)
    }

    /// Create a new cache manager with custom configuration
    pub fn with_config(config: CacheConfig) -> CacheResult<Self> {
        let db = Self::open_database(&config)?;
        Ok(Self {
            db: Arc::new(db),
            config,
            stats: Arc::new(std::sync::Mutex::new(CacheStats::default())),
        })
    }

    /// Create a temporary cache manager (for testing)
    pub fn new_temp() -> CacheResult<Self> {
        let temp_dir = tempfile::tempdir().map_err(|e| CacheError::Custom {
            message: format!("Failed to create temp dir: {}", e),
        })?;

        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        Self::with_config(config)
    }

    /// Open the RocksDB database
    fn open_database(config: &CacheConfig) -> CacheResult<DB> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(if config.compression {
            rocksdb::DBCompressionType::Zstd
        } else {
            rocksdb::DBCompressionType::None
        });

        // Optimize for write performance
        opts.set_write_buffer_size(config.write_buffer_size_mb * 1024 * 1024);
        opts.set_max_background_jobs(4);
        opts.set_keep_log_file_num(10);

        // Enable prefix bloom filter for faster lookups
        let prefix_extractor = SliceTransform::create_fixed_prefix(16);
        opts.set_prefix_extractor(prefix_extractor);

        let db = DB::open(&opts, &config.cache_dir).map_err(CacheError::from)?;

        Ok(db)
    }

    /// Store facts in the cache
    pub fn store_facts(&self, key: &CacheKey, facts: &[Fact]) -> CacheResult<()> {
        let value = CacheValue::new(facts.to_vec(), self.config.ttl_seconds);
        let key_bytes = key.to_bytes()?;
        let value_bytes = value.to_bytes()?;

        self.db
            .put(&key_bytes, &value_bytes)
            .map_err(CacheError::from)?;

        {
            let mut stats = self.stats.lock().unwrap();
            stats.record_stored(value_bytes.len() as u64);
        }

        // Check if we need to evict entries
        if let Some(max_entries) = self.config.max_entries {
            self.maybe_evict_entries(max_entries)?;
        }

        Ok(())
    }

    /// Retrieve facts from the cache
    pub fn get_facts(&self, key: &CacheKey) -> CacheResult<(Vec<Fact>, bool)> {
        let key_bytes = key.to_bytes()?;

        match self.db.get(&key_bytes) {
            Ok(Some(value_bytes)) => {
                let value = CacheValue::from_bytes(&value_bytes)?;

                // Check if expired
                if value.is_expired() {
                    // Delete expired entry
                    let _ = self.db.delete(&key_bytes);
                    {
                        let mut stats = self.stats.lock().unwrap();
                        stats.record_miss();
                    }
                    return Ok((Vec::new(), false));
                }

                // Update access count
                let mut value = value;
                value.access();
                let updated_value_bytes = value.to_bytes()?;
                let _ = self.db.put(&key_bytes, &updated_value_bytes);

                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.record_hit();
                }

                Ok((value.facts, true))
            }
            Ok(None) => {
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.record_miss();
                }
                Ok((Vec::new(), false))
            }
            Err(e) => {
                {
                    let mut stats = self.stats.lock().unwrap();
                    stats.record_miss();
                }
                Err(CacheError::from(e))
            }
        }
    }

    /// Check if a key exists in the cache
    pub fn exists(&self, key: &CacheKey) -> CacheResult<bool> {
        let key_bytes = key.to_bytes()?;
        Ok(self.db.get_pinned(&key_bytes)?.is_some())
    }

    /// Remove a specific entry from the cache
    pub fn remove(&self, key: &CacheKey) -> CacheResult<bool> {
        let key_bytes = key.to_bytes()?;
        Ok(self.db.delete(&key_bytes).map(|_| true)?)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) -> CacheResult<()> {
        let mut batch = WriteBatch::default();
        let mut iter = self.db.iterator(rocksdb::IteratorMode::Start);

        while let Some(item) = iter.next() {
            let (key, _value) = item.map_err(CacheError::from)?;
            batch.delete(&key);
        }

        self.db.write(batch).map_err(CacheError::from)?;

        {
            let mut stats = self.stats.lock().unwrap();
            *stats = CacheStats::default();
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) -> CacheResult<u32> {
        let mut removed_count = 0;
        let mut iter = self.db.iterator(rocksdb::IteratorMode::Start);

        while let Some(item) = iter.next() {
            let (key, value) = item.map_err(CacheError::from)?;

            if let Ok(value) = CacheValue::from_bytes(&value) {
                if value.is_expired() {
                    self.db.delete(&key).map_err(CacheError::from)?;
                    removed_count += 1;
                }
            }
        }

        {
            let mut stats = self.stats.lock().unwrap();
            stats.record_evicted();
        }

        Ok(removed_count)
    }

    /// Evict oldest entries if cache is full
    fn maybe_evict_entries(&self, max_entries: usize) -> CacheResult<()> {
        // This is a simplified eviction strategy
        // In production, you'd want to track access times
        // and use a more sophisticated LRU or LFU strategy
        let stats = self.get_stats();

        if stats.entries_stored > max_entries as u64 {
            // Clean up expired entries first
            let _ = self.cleanup_expired()?;
        }

        Ok(())
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.config.cache_dir
    }
}

impl Drop for CacheManager {
    fn drop(&mut self) {
        // Ensure all data is flushed
        let _ = self.db.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::{ExtractorId, Fact, FactId, FactType, Provenance, Severity};

    fn create_test_fact() -> Fact {
        use hodei_ir::{Confidence, ExtractorId, LineNumber, ProjectPath, SourceLocation};

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "1.0".to_string(),
            Confidence::new(0.5).unwrap(),
        );

        let location = SourceLocation::new(
            ProjectPath::new(PathBuf::from("test.rs")),
            LineNumber::new(1).unwrap(),
            None,
            LineNumber::new(1).unwrap(),
            None,
        );

        Fact::new_with_message(
            FactType::CodeSmell {
                smell_type: "TODO".to_string(),
                severity: Severity::Minor,
                message: "Test".to_string(),
            },
            location,
            provenance,
        )
    }

    #[test]
    fn test_cache_key_creation() {
        let path = PathBuf::from("src/main.rs");
        let hash = "abc123".to_string();
        let key = CacheKey::from_file(&path, hash.clone());

        assert_eq!(key.path, path);
        assert_eq!(key.file_hash, hash);
    }

    #[test]
    fn test_cache_roundtrip() {
        let mut cache = CacheManager::new_temp().unwrap();
        let file_path = PathBuf::from("src/main.rs");
        let file_hash = "abc123".to_string();
        let key = CacheKey::from_file(&file_path, file_hash.clone());
        let facts = vec![create_test_fact()];

        // Store in cache
        cache.store_facts(&key, &facts).unwrap();

        // Retrieve from cache
        let (cached_facts, is_hit) = cache.get_facts(&key).unwrap();
        assert!(is_hit);
        assert_eq!(cached_facts.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = CacheManager::new_temp().unwrap();
        let file_path = PathBuf::from("src/nonexistent.rs");
        let file_hash = "xyz789".to_string();
        let key = CacheKey::from_file(&file_path, file_hash);

        // Try to retrieve non-existent entry
        let (facts, is_hit) = cache.get_facts(&key).unwrap();
        assert!(!is_hit);
        assert!(facts.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = CacheManager::new_temp().unwrap();
        let file_path = PathBuf::from("src/main.rs");
        let file_hash = "abc123".to_string();
        let key = CacheKey::from_file(&file_path, file_hash.clone());
        let facts = vec![create_test_fact()];

        // Initial stats
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Store facts
        cache.store_facts(&key, &facts).unwrap();
        let stats = cache.get_stats();
        assert_eq!(stats.entries_stored, 1);

        // Hit
        let (_, is_hit) = cache.get_facts(&key).unwrap();
        assert!(is_hit);
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);

        // Miss
        let file_path2 = PathBuf::from("src/other.rs");
        let key2 = CacheKey::from_file(&file_path2, "different".to_string());
        let (_, is_hit) = cache.get_facts(&key2).unwrap();
        assert!(!is_hit);
        let stats = cache.get_stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_value_expiration() {
        let mut value = CacheValue::new(vec![], Some(0)); // Expires immediately
        assert!(value.is_expired());

        value = CacheValue::new(vec![], Some(1000)); // 1000 seconds
        assert!(!value.is_expired());
    }

    #[test]
    fn test_cache_serialization() {
        let key = CacheKey {
            path: PathBuf::from("src/main.rs"),
            file_hash: "abc123".to_string(),
            modified_at: 1234567890,
        };

        let key_bytes = key.to_bytes().unwrap();
        let key2 = CacheKey::from_bytes(&key_bytes).unwrap();
        assert_eq!(key, key2);
    }
}
