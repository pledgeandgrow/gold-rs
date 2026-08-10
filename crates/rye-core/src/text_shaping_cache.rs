//! Goal 215: Text shaping cache.
//!
//! Cache `cosmic-text` shaping results for identical text + font + size
//! combinations. Text shaping is expensive — caching eliminates redundant
//! work for static text.

use std::collections::HashMap;
use std::sync::Mutex;

/// A text shaping key — the inputs that determine a shaped text result.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapingKey {
    /// The text content.
    pub text: String,
    /// The font family.
    pub font_family: String,
    /// The font size in pixels.
    pub font_size: u32,
    /// The font weight (100-900).
    pub font_weight: u16,
    /// Whether the text is italic.
    pub italic: bool,
    /// The letter spacing in pixels (0 = default).
    pub letter_spacing: i32,
    /// The line height in pixels (0 = default).
    pub line_height: u32,
}

impl ShapingKey {
    /// Create a new shaping key.
    pub fn new(text: &str, font_family: &str, font_size: u32) -> Self {
        Self {
            text: text.to_string(),
            font_family: font_family.to_string(),
            font_size,
            font_weight: 400,
            italic: false,
            letter_spacing: 0,
            line_height: 0,
        }
    }

    /// Set the font weight.
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set italic.
    pub fn with_italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Set letter spacing.
    pub fn with_letter_spacing(mut self, spacing: i32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set line height.
    pub fn with_line_height(mut self, height: u32) -> Self {
        self.line_height = height;
        self
    }
}

/// A shaped glyph — the result of shaping a single character.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedGlyph {
    /// The glyph ID in the font.
    pub glyph_id: u32,
    /// The x offset.
    pub x_offset: f32,
    /// The y offset.
    pub y_offset: f32,
    /// The advance width.
    pub advance: f32,
    /// The cluster (character index).
    pub cluster: u32,
}

/// A shaped text run — the result of shaping a text string.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapedText {
    /// The glyphs.
    pub glyphs: Vec<ShapedGlyph>,
    /// The total width.
    pub width: f32,
    /// The total height (ascent + descent).
    pub height: f32,
    /// The ascent.
    pub ascent: f32,
    /// The descent.
    pub descent: f32,
}

impl ShapedText {
    /// Create a new empty shaped text.
    pub fn empty() -> Self {
        Self {
            glyphs: Vec::new(),
            width: 0.0,
            height: 0.0,
            ascent: 0.0,
            descent: 0.0,
        }
    }

    /// Create a shaped text from glyphs.
    pub fn from_glyphs(glyphs: Vec<ShapedGlyph>, ascent: f32, descent: f32) -> Self {
        let width = glyphs.iter().map(|g| g.advance).sum();
        let height = ascent + descent;
        Self {
            glyphs,
            width,
            height,
            ascent,
            descent,
        }
    }

    /// Get the number of glyphs.
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// Check if the shaped text is empty.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }
}

/// Cache statistics for text shaping.
#[derive(Debug, Clone, Default)]
pub struct ShapingCacheStats {
    /// Total lookups.
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

impl ShapingCacheStats {
    /// Get the hit rate.
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            return 0.0;
        }
        self.hits as f64 / self.lookups as f64
    }
}

/// The text shaping cache — stores shaped text results for reuse.
pub struct TextShapingCache {
    cache: Mutex<HashMap<ShapingKey, ShapedText>>,
    stats: Mutex<ShapingCacheStats>,
    max_size: usize,
}

impl TextShapingCache {
    /// Create a new text shaping cache.
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            stats: Mutex::new(ShapingCacheStats::default()),
            max_size,
        }
    }

    /// Try to get a cached shaped text.
    pub fn get(&self, key: &ShapingKey) -> Option<ShapedText> {
        let mut stats = self.stats.lock().unwrap();
        stats.lookups += 1;

        let result = self.cache.lock().unwrap().get(key).cloned();
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        result
    }

    /// Insert a shaped text into the cache.
    pub fn insert(&self, key: ShapingKey, shaped: ShapedText) {
        let mut cache = self.cache.lock().unwrap();

        if cache.len() >= self.max_size && !cache.contains_key(&key) {
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
                self.stats.lock().unwrap().evictions += 1;
            }
        }

        cache.insert(key, shaped);
        self.stats.lock().unwrap().cache_size = cache.len();
    }

    /// Invalidate a cached entry.
    pub fn invalidate(&self, key: &ShapingKey) -> bool {
        let removed = self.cache.lock().unwrap().remove(key).is_some();
        if removed {
            self.stats.lock().unwrap().cache_size = self.cache.lock().unwrap().len();
        }
        removed
    }

    /// Invalidate all entries for a given font family.
    pub fn invalidate_font(&self, font_family: &str) -> usize {
        let mut cache = self.cache.lock().unwrap();
        let keys_to_remove: Vec<ShapingKey> = cache
            .keys()
            .filter(|k| k.font_family == font_family)
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
        self.stats.lock().unwrap().cache_size = 0;
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
    pub fn stats(&self) -> ShapingCacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get or compute a shaped text.
    pub fn get_or_compute<F: FnOnce(&ShapingKey) -> ShapedText>(
        &self,
        key: &ShapingKey,
        compute: F,
    ) -> ShapedText {
        if let Some(cached) = self.get(key) {
            return cached;
        }
        let shaped = compute(key);
        self.insert(key.clone(), shaped.clone());
        shaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaping_key_new() {
        let key = ShapingKey::new("Hello", "Arial", 16);
        assert_eq!(key.text, "Hello");
        assert_eq!(key.font_family, "Arial");
        assert_eq!(key.font_size, 16);
        assert_eq!(key.font_weight, 400);
        assert!(!key.italic);
    }

    #[test]
    fn test_shaping_key_builder() {
        let key = ShapingKey::new("Bold", "Inter", 20)
            .with_weight(700)
            .with_italic()
            .with_letter_spacing(2)
            .with_line_height(28);
        assert_eq!(key.font_weight, 700);
        assert!(key.italic);
        assert_eq!(key.letter_spacing, 2);
        assert_eq!(key.line_height, 28);
    }

    #[test]
    fn test_shaping_key_equality() {
        let k1 = ShapingKey::new("Hi", "Arial", 16);
        let k2 = ShapingKey::new("Hi", "Arial", 16);
        assert_eq!(k1, k2);

        let k3 = ShapingKey::new("Hi", "Arial", 20);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_shaped_text_empty() {
        let text = ShapedText::empty();
        assert!(text.is_empty());
        assert_eq!(text.glyph_count(), 0);
        assert_eq!(text.width, 0.0);
    }

    #[test]
    fn test_shaped_text_from_glyphs() {
        let glyphs = vec![
            ShapedGlyph {
                glyph_id: 1,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 8.0,
                cluster: 0,
            },
            ShapedGlyph {
                glyph_id: 2,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 10.0,
                cluster: 1,
            },
        ];
        let text = ShapedText::from_glyphs(glyphs, 12.0, 4.0);
        assert_eq!(text.glyph_count(), 2);
        assert_eq!(text.width, 18.0);
        assert_eq!(text.height, 16.0);
        assert_eq!(text.ascent, 12.0);
        assert_eq!(text.descent, 4.0);
    }

    #[test]
    fn test_shaping_cache_get_miss() {
        let cache = TextShapingCache::new(100);
        let key = ShapingKey::new("Hello", "Arial", 16);
        assert!(cache.get(&key).is_none());
        let stats = cache.stats();
        assert_eq!(stats.lookups, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_shaping_cache_insert_get() {
        let cache = TextShapingCache::new(100);
        let key = ShapingKey::new("Hello", "Arial", 16);
        let shaped = ShapedText::from_glyphs(
            vec![ShapedGlyph {
                glyph_id: 1,
                x_offset: 0.0,
                y_offset: 0.0,
                advance: 8.0,
                cluster: 0,
            }],
            12.0,
            4.0,
        );
        cache.insert(key.clone(), shaped.clone());

        let cached = cache.get(&key).unwrap();
        assert_eq!(cached.glyph_count(), 1);
        assert_eq!(cached.width, 8.0);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_shaping_cache_invalidate() {
        let cache = TextShapingCache::new(100);
        let key = ShapingKey::new("Test", "Arial", 16);
        cache.insert(key.clone(), ShapedText::empty());
        assert!(cache.invalidate(&key));
        assert!(cache.get(&key).is_none());
        assert!(!cache.invalidate(&key));
    }

    #[test]
    fn test_shaping_cache_invalidate_font() {
        let cache = TextShapingCache::new(100);
        cache.insert(ShapingKey::new("A", "Arial", 16), ShapedText::empty());
        cache.insert(ShapingKey::new("B", "Arial", 20), ShapedText::empty());
        cache.insert(ShapingKey::new("C", "Inter", 16), ShapedText::empty());

        let count = cache.invalidate_font("Arial");
        assert_eq!(count, 2);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_shaping_cache_clear() {
        let cache = TextShapingCache::new(100);
        cache.insert(ShapingKey::new("A", "Arial", 16), ShapedText::empty());
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_shaping_cache_eviction() {
        let cache = TextShapingCache::new(2);
        cache.insert(ShapingKey::new("A", "F1", 16), ShapedText::empty());
        cache.insert(ShapingKey::new("B", "F2", 16), ShapedText::empty());
        cache.insert(ShapingKey::new("C", "F3", 16), ShapedText::empty());

        let stats = cache.stats();
        assert!(stats.evictions >= 1);
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_shaping_cache_get_or_compute() {
        let cache = TextShapingCache::new(100);
        let key = ShapingKey::new("Hello", "Arial", 16);

        // First call: miss, compute
        let result1 = cache.get_or_compute(&key, |k| {
            ShapedText::from_glyphs(
                vec![ShapedGlyph {
                    glyph_id: 1,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    advance: 8.0,
                    cluster: 0,
                }],
                12.0,
                4.0,
            )
        });
        assert_eq!(result1.glyph_count(), 1);

        // Second call: hit, cached
        let result2 = cache.get_or_compute(&key, |_| panic!("Should not compute"));
        assert_eq!(result2.glyph_count(), 1);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_shaping_cache_stats_hit_rate() {
        let cache = TextShapingCache::new(100);
        let key = ShapingKey::new("Test", "Arial", 16);
        cache.insert(key.clone(), ShapedText::empty());

        cache.get(&key); // hit
        cache.get(&ShapingKey::new("Other", "Arial", 16)); // miss

        let stats = cache.stats();
        assert_eq!(stats.hit_rate(), 0.5);
    }
}
