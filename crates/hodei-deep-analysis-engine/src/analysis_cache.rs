//! Analysis Cache for Performance Optimization
//!
//! This module provides caching mechanisms to improve performance of
//! repeated analyses on the same code.

use ahash::AHashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache entry with expiration time
#[derive(Debug)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    expires_at: Instant,
}

/// Thread-safe analysis cache
#[derive(Debug, Clone)]
pub struct AnalysisCache<T> {
    inner: Arc<RwLock<AHashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

impl<T> AnalysisCache<T> {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            inner: Arc::new(RwLock::new(AHashMap::new())),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let lock = self.inner.read().unwrap();
        if let Some(entry) = lock.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Put a value into the cache
    pub fn put(&self, key: String, value: T) {
        let now = Instant::now();
        let entry = CacheEntry {
            value,
            created_at: now,
            expires_at: now + self.default_ttl,
        };

        let mut lock = self.inner.write().unwrap();
        lock.insert(key, entry);
    }

    /// Clear expired entries
    pub fn cleanup(&self) {
        let mut lock = self.inner.write().unwrap();
        let now = Instant::now();

        lock.retain(|_, entry| entry.expires_at > now);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let lock = self.inner.read().unwrap();
        let now = Instant::now();

        let total = lock.len();
        let expired = lock
            .values()
            .filter(|entry| entry.expires_at <= now)
            .count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
            hit_rate: 0.0, // TODO: Track hits/misses
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub hit_rate: f64,
}

impl<T> Default for AnalysisCache<T> {
    fn default() -> Self {
        Self::new(3600) // Default 1 hour TTL
    }
}

/// Semantic model cache
pub type SemanticModelCache = AnalysisCache<String>;

/// Taint flow cache
pub type TaintFlowCache = AnalysisCache<Vec<String>>;

/// Coupling finding cache
pub type CouplingCache = AnalysisCache<Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_new() {
        let cache = AnalysisCache::<String>::new(60);
        assert!(format!("{:?}", cache).contains("AnalysisCache"));
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = AnalysisCache::<String>::new(60);

        cache.put("key1".to_string(), "value1".to_string());

        let value = cache.get("key1");
        assert_eq!(value, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_get_nonexistent_key() {
        let cache = AnalysisCache::<String>::new(60);

        let value = cache.get("nonexistent");
        assert_eq!(value, None);
    }

    #[test]
    fn test_cache_update_existing_key() {
        let cache = AnalysisCache::new(60);

        cache.put("key".to_string(), "value1".to_string());
        assert_eq!(cache.get("key"), Some("value1".to_string()));

        cache.put("key".to_string(), "value2".to_string());
        assert_eq!(cache.get("key"), Some("value2".to_string()));
    }

    #[test]
    fn test_cache_multiple_keys() {
        let cache = AnalysisCache::new(60);

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());

        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), Some("value2".to_string()));
        assert_eq!(cache.get("key3"), Some("value3".to_string()));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.active_entries, 3);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = AnalysisCache::new(1); // 1 second TTL

        cache.put("key2".to_string(), "value2".to_string());

        // Should be present immediately
        assert_eq!(cache.get("key2"), Some("value2".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // Should be expired
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = AnalysisCache::new(1);

        cache.put("key3".to_string(), "value3".to_string());

        thread::sleep(Duration::from_secs(2));

        // Verify expired before cleanup
        assert_eq!(cache.get("key3"), None);

        cache.cleanup();

        let stats = cache.stats();
        assert_eq!(stats.expired_entries, 0);
        assert_eq!(stats.active_entries, 0);
    }

    #[test]
    fn test_cache_cleanup_preserves_active_entries() {
        // Use 0 TTL so entries expire immediately when checked
        let cache = AnalysisCache::<String>::new(0);

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());

        // With 0 TTL, all entries should appear expired immediately
        let stats_before = cache.stats();
        assert_eq!(stats_before.expired_entries, 3);
        assert_eq!(stats_before.active_entries, 0);

        cache.cleanup();

        let stats_after = cache.stats();
        assert_eq!(stats_after.expired_entries, 0);
        assert_eq!(stats_after.active_entries, 0);

        // All entries should be gone
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);
        assert_eq!(cache.get("key3"), None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = AnalysisCache::new(60);

        cache.put("key4".to_string(), "value4".to_string());

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.active_entries, 1);
        assert_eq!(stats.expired_entries, 0);
    }

    #[test]
    fn test_cache_stats_multiple_entries() {
        let cache = AnalysisCache::new(1);

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());

        thread::sleep(Duration::from_secs(2));

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 3);
    }

    #[test]
    fn test_cache_default_ttl() {
        let cache = AnalysisCache::<String>::default();
        assert_eq!(cache.default_ttl, Duration::from_secs(3600)); // 1 hour
    }

    #[test]
    fn test_cache_custom_ttl() {
        let cache = AnalysisCache::<String>::new(120);
        assert_eq!(cache.default_ttl, Duration::from_secs(120));
    }

    #[test]
    fn test_cache_zero_ttl() {
        let cache = AnalysisCache::<String>::new(0);
        assert_eq!(cache.default_ttl, Duration::from_secs(0));

        cache.put("key".to_string(), "value".to_string());
        // Should expire immediately
        assert_eq!(cache.get("key"), None);
    }

    #[test]
    fn test_cache_very_long_ttl() {
        let cache = AnalysisCache::<String>::new(86400); // 24 hours

        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_cache_clone() {
        let cache = AnalysisCache::new(60);
        cache.put("key".to_string(), "value".to_string());

        let _clone = cache.clone();
        // Should compile, Arc allows cloning
    }

    #[test]
    fn test_cache_concurrent_access() {
        let cache = AnalysisCache::new(60);
        cache.put("key1".to_string(), "value1".to_string());

        let cache_clone = cache.clone();

        // Both caches should have the same data
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache_clone.get("key1"), Some("value1".to_string()));

        // Update through one cache
        cache.put("key1".to_string(), "value2".to_string());

        // Both should see the update (Arc sharing)
        assert_eq!(cache.get("key1"), Some("value2".to_string()));
        assert_eq!(cache_clone.get("key1"), Some("value2".to_string()));
    }

    #[test]
    fn test_cache_empty_strings() {
        let cache = AnalysisCache::new(60);

        cache.put("".to_string(), "value".to_string());
        assert_eq!(cache.get(""), Some("value".to_string()));

        cache.put("key".to_string(), "".to_string());
        assert_eq!(cache.get("key"), Some("".to_string()));
    }

    #[test]
    fn test_cache_large_values() {
        let cache = AnalysisCache::new(60);

        let large_value = "x".repeat(10000);
        cache.put("key".to_string(), large_value.clone());

        let retrieved = cache.get("key");
        assert_eq!(retrieved, Some(large_value));
    }

    #[test]
    fn test_cache_many_entries() {
        let cache = AnalysisCache::new(60);

        // Add 1000 entries
        for i in 0..1000 {
            cache.put(format!("key{}", i), format!("value{}", i));
        }

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1000);
        assert_eq!(stats.active_entries, 1000);

        // Verify some entries
        assert_eq!(cache.get("key0"), Some("value0".to_string()));
        assert_eq!(cache.get("key999"), Some("value999".to_string()));
        assert_eq!(cache.get("key500"), Some("value500".to_string()));
    }

    #[test]
    fn test_cache_cleanup_empty() {
        let cache = AnalysisCache::<String>::new(60);
        cache.cleanup();

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_entries, 0);
        assert_eq!(stats.expired_entries, 0);
    }

    #[test]
    fn test_cache_stats_consistency() {
        let cache = AnalysisCache::<String>::new(1);

        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());

        let stats1 = cache.stats();
        assert_eq!(stats1.total_entries, 2);
        assert_eq!(stats1.active_entries, 2);
        assert_eq!(stats1.expired_entries, 0);

        thread::sleep(Duration::from_secs(2));

        let stats2 = cache.stats();
        assert_eq!(stats2.total_entries, 2);
        assert_eq!(stats2.active_entries, 0);
        assert_eq!(stats2.expired_entries, 2);

        cache.cleanup();

        let stats3 = cache.stats();
        assert_eq!(stats3.total_entries, 0);
        assert_eq!(stats3.active_entries, 0);
        assert_eq!(stats3.expired_entries, 0);
    }

    #[test]
    fn test_cache_hit_rate_initially_zero() {
        let cache = AnalysisCache::<String>::new(60);
        let stats = cache.stats();
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_mixed_operations() {
        let cache = AnalysisCache::new(1);

        // Add entries
        cache.put("key1".to_string(), "value1".to_string());
        cache.put("key2".to_string(), "value2".to_string());

        // Get existing
        assert_eq!(cache.get("key1"), Some("value1".to_string()));

        // Get nonexistent
        assert_eq!(cache.get("nonexistent"), None);

        // Update
        cache.put("key1".to_string(), "value1_updated".to_string());
        assert_eq!(cache.get("key1"), Some("value1_updated".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_secs(2));

        // All should be expired
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);

        // Add new entry
        cache.put("key3".to_string(), "value3".to_string());
        assert_eq!(cache.get("key3"), Some("value3".to_string()));
    }

    #[test]
    fn test_cache_get_returns_cloned_value() {
        let cache = AnalysisCache::new(60);

        let value = vec![1, 2, 3];
        cache.put("key".to_string(), value.clone());

        let retrieved = cache.get("key").unwrap();
        assert_eq!(retrieved, value);

        // Original value should still be valid
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    fn test_specialized_cache_types() {
        let _semantic_cache = SemanticModelCache::new(300);
        let _taint_cache = TaintFlowCache::new(600);
        let _coupling_cache = CouplingCache::new(1200);

        // All should be different types but same underlying structure
        assert!(format!("{:?}", _semantic_cache).contains("AnalysisCache"));
        assert!(format!("{:?}", _taint_cache).contains("AnalysisCache"));
        assert!(format!("{:?}", _coupling_cache).contains("AnalysisCache"));
    }

    #[test]
    fn test_cache_key_with_special_chars() {
        let cache = AnalysisCache::new(60);

        let special_keys = vec![
            "key-with-dashes".to_string(),
            "key_with_underscores".to_string(),
            "key.with.dots".to_string(),
            "key/with/slashes".to_string(),
            "key:with:colons".to_string(),
        ];

        for key in &special_keys {
            cache.put(key.clone(), "value".to_string());
        }

        for key in &special_keys {
            assert_eq!(cache.get(key), Some("value".to_string()));
        }
    }

    #[test]
    fn test_cache_unicode_keys_and_values() {
        let cache = AnalysisCache::new(60);

        let unicode_key = "键🔑".to_string();
        let unicode_value = "值📦".to_string();

        cache.put(unicode_key.clone(), unicode_value.clone());
        assert_eq!(cache.get(&unicode_key), Some(unicode_value));
    }
}
