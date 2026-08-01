//! Goal 214: Layout caching.
//!
//! Cache Taffy layout results for identical component configurations.
//! If a component's props and children haven't changed, reuse the cached layout.

use std::collections::HashMap;
use std::sync::Mutex;

/// A layout result — the computed positions and sizes of nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutResult {
    /// The node ID.
    pub node_id: u64,
    /// The x position.
    pub x: f32,
    /// The y position.
    pub y: f32,
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
    /// The children layout results.
    pub children: Vec<LayoutResult>,
}

impl LayoutResult {
    /// Create a new layout result.
    pub fn new(node_id: u64, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            node_id,
            x,
            y,
            width,
            height,
            children: Vec::new(),
        }
    }

    /// Add a child layout result.
    pub fn add_child(&mut self, child: LayoutResult) {
        self.children.push(child);
    }

    /// Get the total number of nodes (including children).
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }
}

/// A layout configuration — the inputs that determine a layout.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutConfig {
    /// The component type hash.
    pub component_hash: u64,
    /// The props hash.
    pub props_hash: u64,
    /// The children hash.
    pub children_hash: u64,
    /// The available width.
    pub available_width: f32,
    /// The available height.
    pub available_height: f32,
}

impl LayoutConfig {
    /// Create a new layout configuration.
    pub fn new(component_hash: u64, props_hash: u64, children_hash: u64) -> Self {
        Self {
            component_hash,
            props_hash,
            children_hash,
            available_width: 0.0,
            available_height: 0.0,
        }
    }

    /// Set the available size.
    pub fn with_available_size(mut self, width: f32, height: f32) -> Self {
        self.available_width = width;
        self.available_height = height;
        self
    }

    /// Compute the cache key.
    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.component_hash,
            self.props_hash,
            self.children_hash,
            self.available_width.to_bits(),
            self.available_height.to_bits()
        )
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct LayoutCacheStats {
    /// Total cache lookups.
    pub lookups: u64,
    /// Cache hits.
    pub hits: u64,
    /// Cache misses.
    pub misses: u64,
    /// Cache evictions.
    pub evictions: u64,
    /// Current cache size.
    pub cache_size: usize,
}

impl LayoutCacheStats {
    /// Get the hit rate (0.0-1.0).
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            return 0.0;
        }
        self.hits as f64 / self.lookups as f64
    }

    /// Get the miss rate (0.0-1.0).
    pub fn miss_rate(&self) -> f64 {
        if self.lookups == 0 {
            return 0.0;
        }
        self.misses as f64 / self.lookups as f64
    }
}

/// The layout cache — stores computed layout results for reuse.
pub struct LayoutCache {
    cache: Mutex<HashMap<String, LayoutResult>>,
    stats: Mutex<LayoutCacheStats>,
    max_size: usize,
}

impl LayoutCache {
    /// Create a new layout cache with a maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            stats: Mutex::new(LayoutCacheStats::default()),
            max_size,
        }
    }

    /// Try to get a cached layout result.
    pub fn get(&self, config: &LayoutConfig) -> Option<LayoutResult> {
        let key = config.cache_key();
        let mut stats = self.stats.lock().unwrap();
        stats.lookups += 1;

        let result = self.cache.lock().unwrap().get(&key).cloned();
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        result
    }

    /// Insert a layout result into the cache.
    pub fn insert(&self, config: &LayoutConfig, result: LayoutResult) {
        let key = config.cache_key();
        let mut cache = self.cache.lock().unwrap();

        // Evict if at capacity (simple LRU-like: remove random entry)
        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
                self.stats.lock().unwrap().evictions += 1;
            }
        }

        cache.insert(key, result);
        self.stats.lock().unwrap().cache_size = cache.len();
    }

    /// Invalidate a cached layout.
    pub fn invalidate(&self, config: &LayoutConfig) -> bool {
        let key = config.cache_key();
        let removed = self.cache.lock().unwrap().remove(&key).is_some();
        if removed {
            let mut stats = self.stats.lock().unwrap();
            stats.cache_size = self.cache.lock().unwrap().len();
        }
        removed
    }

    /// Invalidate all layouts for a given component hash.
    pub fn invalidate_component(&self, component_hash: u64) -> usize {
        let mut cache = self.cache.lock().unwrap();
        let keys_to_remove: Vec<String> = cache
            .keys()
            .filter(|k| k.starts_with(&format!("{}:", component_hash)))
            .cloned()
            .collect();
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            cache.remove(&key);
        }
        self.stats.lock().unwrap().cache_size = cache.len();
        count
    }

    /// Clear the entire cache.
    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
        let mut stats = self.stats.lock().unwrap();
        stats.cache_size = 0;
    }

    /// Get the current cache size.
    pub fn len(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.lock().unwrap().is_empty()
    }

    /// Get cache statistics.
    pub fn stats(&self) -> LayoutCacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get the max size.
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// A layout cache that also tracks prop/children hashes for invalidation.
pub struct SmartLayoutCache {
    cache: LayoutCache,
    component_hashes: Mutex<HashMap<u64, Vec<u64>>>,
}

impl SmartLayoutCache {
    /// Create a new smart layout cache.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: LayoutCache::new(max_size),
            component_hashes: Mutex::new(HashMap::new()),
        }
    }

    /// Register a component instance with its props and children hashes.
    pub fn register(&self, component_hash: u64, props_hash: u64, children_hash: u64) {
        let mut hashes = self.component_hashes.lock().unwrap();
        hashes
            .entry(component_hash)
            .or_insert_with(Vec::new)
            .push(props_hash ^ children_hash);
    }

    /// Invalidate all cached layouts for a component.
    pub fn invalidate_component(&self, component_hash: u64) -> usize {
        self.component_hashes.lock().unwrap().remove(&component_hash);
        self.cache.invalidate_component(component_hash)
    }

    /// Try to get a cached layout.
    pub fn get(&self, config: &LayoutConfig) -> Option<LayoutResult> {
        self.cache.get(config)
    }

    /// Insert a layout result.
    pub fn insert(&self, config: &LayoutConfig, result: LayoutResult) {
        self.cache.insert(config, result);
    }

    /// Get cache statistics.
    pub fn stats(&self) -> LayoutCacheStats {
        self.cache.stats()
    }

    /// Clear the cache.
    pub fn clear(&self) {
        self.cache.clear();
        self.component_hashes.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_result_new() {
        let result = LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0);
        assert_eq!(result.node_id, 1);
        assert_eq!(result.width, 100.0);
        assert_eq!(result.children.len(), 0);
    }

    #[test]
    fn test_layout_result_add_child() {
        let mut parent = LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0);
        parent.add_child(LayoutResult::new(2, 10.0, 10.0, 80.0, 30.0));
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_layout_result_node_count() {
        let mut parent = LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0);
        parent.add_child(LayoutResult::new(2, 0.0, 0.0, 50.0, 25.0));
        parent.add_child(LayoutResult::new(3, 50.0, 0.0, 50.0, 25.0));
        assert_eq!(parent.node_count(), 3);
    }

    #[test]
    fn test_layout_config_cache_key() {
        let config1 = LayoutConfig::new(1, 2, 3);
        let config2 = LayoutConfig::new(1, 2, 3);
        assert_eq!(config1.cache_key(), config2.cache_key());

        let config3 = LayoutConfig::new(1, 3, 3);
        assert_ne!(config1.cache_key(), config3.cache_key());
    }

    #[test]
    fn test_layout_config_with_size() {
        let config = LayoutConfig::new(1, 2, 3).with_available_size(800.0, 600.0);
        assert_eq!(config.available_width, 800.0);
        assert_eq!(config.available_height, 600.0);
    }

    #[test]
    fn test_layout_cache_stats_hit_rate() {
        let mut stats = LayoutCacheStats::default();
        stats.lookups = 100;
        stats.hits = 80;
        stats.misses = 20;
        assert_eq!(stats.hit_rate(), 0.8);
        assert_eq!(stats.miss_rate(), 0.2);
    }

    #[test]
    fn test_layout_cache_stats_empty() {
        let stats = LayoutCacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_layout_cache_get_miss() {
        let cache = LayoutCache::new(100);
        let config = LayoutConfig::new(1, 2, 3);
        assert!(cache.get(&config).is_none());
        let stats = cache.stats();
        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_layout_cache_insert_get() {
        let cache = LayoutCache::new(100);
        let config = LayoutConfig::new(1, 2, 3);
        let result = LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0);
        cache.insert(&config, result.clone());

        let cached = cache.get(&config).unwrap();
        assert_eq!(cached.node_id, 1);
        assert_eq!(cached.width, 100.0);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_layout_cache_invalidate() {
        let cache = LayoutCache::new(100);
        let config = LayoutConfig::new(1, 2, 3);
        cache.insert(&config, LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0));
        assert!(cache.invalidate(&config));
        assert!(cache.get(&config).is_none());
        assert!(!cache.invalidate(&config));
    }

    #[test]
    fn test_layout_cache_invalidate_component() {
        let cache = LayoutCache::new(100);
        cache.insert(&LayoutConfig::new(1, 10, 20), LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0));
        cache.insert(&LayoutConfig::new(1, 20, 30), LayoutResult::new(2, 0.0, 0.0, 200.0, 100.0));
        cache.insert(&LayoutConfig::new(2, 40, 50), LayoutResult::new(3, 0.0, 0.0, 300.0, 150.0));

        let count = cache.invalidate_component(1);
        assert_eq!(count, 2);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_layout_cache_clear() {
        let cache = LayoutCache::new(100);
        cache.insert(&LayoutConfig::new(1, 2, 3), LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0));
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_layout_cache_eviction() {
        let cache = LayoutCache::new(2);
        cache.insert(&LayoutConfig::new(1, 1, 1), LayoutResult::new(1, 0.0, 0.0, 10.0, 10.0));
        cache.insert(&LayoutConfig::new(2, 2, 2), LayoutResult::new(2, 0.0, 0.0, 20.0, 20.0));
        cache.insert(&LayoutConfig::new(3, 3, 3), LayoutResult::new(3, 0.0, 0.0, 30.0, 30.0));

        let stats = cache.stats();
        assert!(stats.evictions >= 1);
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_smart_layout_cache_register_invalidate() {
        let cache = SmartLayoutCache::new(100);
        cache.register(1, 10, 20);
        cache.insert(&LayoutConfig::new(1, 10, 20), LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0));

        assert!(cache.get(&LayoutConfig::new(1, 10, 20)).is_some());
        let count = cache.invalidate_component(1);
        assert_eq!(count, 1);
        assert!(cache.get(&LayoutConfig::new(1, 10, 20)).is_none());
    }

    #[test]
    fn test_smart_layout_cache_clear() {
        let cache = SmartLayoutCache::new(100);
        cache.register(1, 10, 20);
        cache.insert(&LayoutConfig::new(1, 10, 20), LayoutResult::new(1, 0.0, 0.0, 100.0, 50.0));
        cache.clear();
        assert_eq!(cache.stats().cache_size, 0);
    }
}
