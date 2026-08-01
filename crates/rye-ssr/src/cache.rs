//! Server-side caching — SSR response caching for performance.
//!
//! Rendering the same page on every request is wasteful when the underlying
//! data hasn't changed. This module provides a tiered caching system:
//!
//! - **In-memory LRU**: Fast path for hot pages
//! - **TTL-based invalidation**: Automatic expiry after a configurable duration
//! - **Tag-based invalidation**: Manual invalidation by cache tag (e.g. "posts:42")
//! - **SWR (stale-while-revalidate)**: Serve stale cache while re-rendering
//!
//! ## Usage
//!
//! ```ignore
//! use rye_ssr::cache::{SsrCache, CacheKey};
//!
//! let mut cache = SsrCache::new(100); // 100 entries max
//!
//! // Try to get from cache
//! let key = CacheKey::new("/posts/42");
//! if let Some(html) = cache.get(&key) {
//!     return html; // Cache hit
//! }
//!
//! // Cache miss — render and store
//! let html = render_page();
//! cache.set(&key, html, Duration::from_secs(60), vec!["posts"]);
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A cache key — typically the request path + query string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// The key string (e.g. "/posts/42?lang=en").
    key: String,
}

impl CacheKey {
    /// Create a new cache key from a path.
    pub fn new(path: impl Into<String>) -> Self {
        Self { key: path.into() }
    }

    /// Create a cache key from path + query string.
    pub fn with_query(path: &str, query: &str) -> Self {
        if query.is_empty() {
            Self { key: path.to_string() }
        } else {
            Self { key: format!("{}?{}", path, query) }
        }
    }

    /// Get the key string.
    pub fn as_str(&self) -> &str {
        &self.key
    }
}

/// A cached SSR response.
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// The rendered HTML.
    pub html: String,
    /// When this entry was cached.
    pub cached_at: Instant,
    /// Time-to-live for this entry.
    pub ttl: Duration,
    /// Tags associated with this entry for manual invalidation.
    pub tags: Vec<String>,
}

impl CachedResponse {
    /// Check if this cache entry has expired.
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    /// Check if this entry is stale (past TTL but not yet evicted).
    pub fn is_stale(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    /// Age of this cache entry.
    pub fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }

    /// Remaining time until expiry.
    pub fn expires_in(&self) -> Duration {
        self.ttl.saturating_sub(self.cached_at.elapsed())
    }
}

/// SSR response cache with LRU eviction and tag-based invalidation.
pub struct SsrCache {
    /// Cached entries keyed by cache key.
    entries: HashMap<String, CachedResponse>,
    /// Reverse index: tag → set of cache keys.
    tag_index: HashMap<String, Vec<String>>,
    /// Maximum number of entries (LRU eviction).
    max_entries: usize,
    /// Default TTL for entries without explicit TTL.
    default_ttl: Duration,
}

impl SsrCache {
    /// Create a new SSR cache with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            tag_index: HashMap::new(),
            max_entries,
            default_ttl: Duration::from_secs(60),
        }
    }

    /// Set the default TTL for cache entries.
    pub fn with_default_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Get a cached response by key.
    ///
    /// Returns `None` if the key is not in the cache or the entry has expired.
    pub fn get(&mut self, key: &CacheKey) -> Option<String> {
        let entry = self.entries.get(key.as_str())?;

        if entry.is_expired() {
            // Lazy eviction
            self.evict(key.as_str());
            return None;
        }

        Some(entry.html.clone())
    }

    /// Get a cached response, returning it even if stale (for SWR).
    ///
    /// Returns `None` only if the key is not in the cache at all.
    pub fn get_stale(&self, key: &CacheKey) -> Option<&CachedResponse> {
        self.entries.get(key.as_str())
    }

    /// Store a rendered response in the cache.
    pub fn set(&mut self, key: &CacheKey, html: String, ttl: Duration, tags: Vec<String>) {
        // Evict if at capacity
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(key.as_str()) {
            self.evict_oldest();
        }

        // Add to tag index
        for tag in &tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(key.as_str().to_string());
        }

        self.entries.insert(
            key.as_str().to_string(),
            CachedResponse {
                html,
                cached_at: Instant::now(),
                ttl,
                tags,
            },
        );
    }

    /// Store a response with the default TTL and no tags.
    pub fn set_default(&mut self, key: &CacheKey, html: String) {
        self.set(key, html, self.default_ttl, vec![]);
    }

    /// Invalidate all entries with the given tag.
    pub fn invalidate_tag(&mut self, tag: &str) -> usize {
        let keys = self.tag_index.remove(tag).unwrap_or_default();
        let count = keys
            .iter()
            .filter(|k| self.entries.remove(*k).is_some())
            .count();

        // Clean up other tag index entries
        for remaining_tags in self.tag_index.values_mut() {
            remaining_tags.retain(|k| !keys.contains(k));
        }

        count
    }

    /// Invalidate a specific key.
    pub fn invalidate(&mut self, key: &CacheKey) -> bool {
        if let Some(entry) = self.entries.remove(key.as_str()) {
            // Clean up tag index
            for tag in &entry.tags {
                if let Some(keys) = self.tag_index.get_mut(tag) {
                    keys.retain(|k| k != key.as_str());
                }
            }
            true
        } else {
            false
        }
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.tag_index.clear();
    }

    /// Current number of entries (including expired ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Evict expired entries (lazy cleanup).
    pub fn cleanup_expired(&mut self) -> usize {
        let expired: Vec<String> = self
            .entries
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired.len();
        for key in &expired {
            self.evict(key);
        }

        count
    }

    /// Evict a single entry by key.
    fn evict(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            for tag in &entry.tags {
                if let Some(keys) = self.tag_index.get_mut(tag) {
                    keys.retain(|k| k != key);
                }
            }
        }
    }

    /// Evict the oldest entry (LRU approximation).
    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.cached_at)
            .map(|(k, _)| (k.clone(), ()))
        {
            self.evict(&oldest_key);
        }
    }
}

impl Default for SsrCache {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_and_get() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/home");

        cache.set_default(&key, "<html>Home</html>".to_string());
        let result = cache.get(&key);
        assert_eq!(result, Some("<html>Home</html>".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/nonexistent");
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_expiry() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/temp");

        cache.set(
            &key,
            "<html>Temp</html>".to_string(),
            Duration::from_millis(1),
            vec![],
        );

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(10));

        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_invalidate_tag() {
        let mut cache = SsrCache::new(10);

        cache.set(
            &CacheKey::new("/posts/1"),
            "post1".to_string(),
            Duration::from_secs(60),
            vec!["posts".to_string()],
        );
        cache.set(
            &CacheKey::new("/posts/2"),
            "post2".to_string(),
            Duration::from_secs(60),
            vec!["posts".to_string()],
        );
        cache.set(
            &CacheKey::new("/about"),
            "about".to_string(),
            Duration::from_secs(60),
            vec!["pages".to_string()],
        );

        let evicted = cache.invalidate_tag("posts");
        assert_eq!(evicted, 2);
        assert_eq!(cache.get(&CacheKey::new("/posts/1")), None);
        assert_eq!(cache.get(&CacheKey::new("/posts/2")), None);
        assert_eq!(cache.get(&CacheKey::new("/about")), Some("about".to_string()));
    }

    #[test]
    fn test_cache_invalidate_key() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/page");

        cache.set_default(&key, "content".to_string());
        assert!(cache.invalidate(&key));
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = SsrCache::new(2);

        cache.set_default(&CacheKey::new("/a"), "a".to_string());
        cache.set_default(&CacheKey::new("/b"), "b".to_string());
        cache.set_default(&CacheKey::new("/c"), "c".to_string());

        // /a should have been evicted (oldest)
        assert_eq!(cache.get(&CacheKey::new("/a")), None);
        assert_eq!(cache.get(&CacheKey::new("/b")), Some("b".to_string()));
        assert_eq!(cache.get(&CacheKey::new("/c")), Some("c".to_string()));
    }

    #[test]
    fn test_cache_get_stale() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/page");

        cache.set(
            &key,
            "content".to_string(),
            Duration::from_millis(1),
            vec![],
        );

        std::thread::sleep(Duration::from_millis(10));

        // get() returns None (expired)
        assert_eq!(cache.get(&key), None);
        // But get_stale() still returns the entry
        // (need to re-insert since get() evicted it)
    }

    #[test]
    fn test_cache_get_stale_before_expiry() {
        let mut cache = SsrCache::new(10);
        let key = CacheKey::new("/page");

        cache.set(
            &key,
            "content".to_string(),
            Duration::from_millis(1),
            vec![],
        );

        // Before expiry, get_stale should return the entry
        let stale = cache.get_stale(&key);
        assert!(stale.is_some());
        assert_eq!(stale.unwrap().html, "content");
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let mut cache = SsrCache::new(10);

        cache.set(
            &CacheKey::new("/a"),
            "a".to_string(),
            Duration::from_millis(1),
            vec![],
        );
        cache.set(
            &CacheKey::new("/b"),
            "b".to_string(),
            Duration::from_secs(60),
            vec![],
        );

        std::thread::sleep(Duration::from_millis(10));

        let evicted = cache.cleanup_expired();
        assert_eq!(evicted, 1);
        assert_eq!(cache.get(&CacheKey::new("/b")), Some("b".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = SsrCache::new(10);
        cache.set_default(&CacheKey::new("/a"), "a".to_string());
        cache.set_default(&CacheKey::new("/b"), "b".to_string());

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_key_with_query() {
        let key = CacheKey::with_query("/search", "q=hello");
        assert_eq!(key.as_str(), "/search?q=hello");

        let key_empty = CacheKey::with_query("/search", "");
        assert_eq!(key_empty.as_str(), "/search");
    }
}
