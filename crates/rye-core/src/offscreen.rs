//! Offscreen rendering / prerendering — pre-render components before they're needed.
//!
//! Pre-render components off-screen before they're needed (route preloading,
//! tab pre-rendering). When the user navigates, the pre-rendered content
//! appears instantly.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A prerendered component — cached render output ready for instant display.
pub struct PrerenderedNode {
    /// The rendered HTML/string content.
    pub content: String,
    /// The route or key this was prerendered for.
    pub key: String,
    /// When this was prerendered (monotonic counter).
    pub created_at: u64,
    /// Whether this node has been consumed (mounted to the DOM).
    pub consumed: bool,
}

/// The prerender cache — stores pre-rendered content for instant navigation.
pub struct PrerenderCache {
    entries: RefCell<HashMap<String, PrerenderedNode>>,
    counter: RefCell<u64>,
}

impl PrerenderCache {
    /// Create a new empty prerender cache.
    pub fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
            counter: RefCell::new(0),
        }
    }

    /// Prerender a component and store it in the cache.
    pub fn prerender<F: FnOnce() -> String>(&self, key: &str, render_fn: F) -> PrerenderedNode {
        let content = render_fn();
        let timestamp = {
            let mut c = self.counter.borrow_mut();
            *c += 1;
            *c
        };

        let node = PrerenderedNode {
            content,
            key: key.to_string(),
            created_at: timestamp,
            consumed: false,
        };

        self.entries
            .borrow_mut()
            .insert(key.to_string(), PrerenderedNode {
                content: node.content.clone(),
                key: node.key.clone(),
                created_at: node.created_at,
                consumed: false,
            });

        node
    }

    /// Take a prerendered node from the cache. Returns None if not found or already consumed.
    pub fn take(&self, key: &str) -> Option<PrerenderedNode> {
        let mut entries = self.entries.borrow_mut();
        if let Some(node) = entries.get_mut(key) {
            if node.consumed {
                return None;
            }
            node.consumed = true;
            return Some(PrerenderedNode {
                content: node.content.clone(),
                key: node.key.clone(),
                created_at: node.created_at,
                consumed: true,
            });
        }
        None
    }

    /// Peek at a prerendered node without consuming it.
    pub fn peek(&self, key: &str) -> Option<String> {
        self.entries.borrow().get(key).map(|n| n.content.clone())
    }

    /// Check if a key has been prerendered and is available.
    pub fn has(&self, key: &str) -> bool {
        self.entries
            .borrow()
            .get(key)
            .map(|n| !n.consumed)
            .unwrap_or(false)
    }

    /// Remove a prerendered node from the cache.
    pub fn evict(&self, key: &str) {
        self.entries.borrow_mut().remove(key);
    }

    /// Clear all prerendered nodes.
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }

    /// Get all cached keys.
    pub fn keys(&self) -> Vec<String> {
        self.entries.borrow().keys().cloned().collect()
    }
}

impl Default for PrerenderCache {
    fn default() -> Self {
        Self::new()
    }
}

/// A prerender request — describes what to prerender and how.
#[derive(Debug, Clone)]
pub struct PrerenderRequest {
    /// The key to store the result under (e.g. route path).
    pub key: String,
    /// The priority of this prerender request.
    pub priority: PrerenderPriority,
}

/// Priority for prerender requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrerenderPriority {
    /// Low priority — prerender when idle.
    Idle,
    /// Normal priority — prerender soon.
    Normal,
    /// High priority — prerender immediately (e.g. likely navigation target).
    High,
}

/// A prerender scheduler — manages a queue of prerender requests.
pub struct PrerenderScheduler {
    cache: Rc<PrerenderCache>,
    queue: RefCell<Vec<PrerenderRequest>>,
}

impl PrerenderScheduler {
    /// Create a new scheduler with a shared cache.
    pub fn new(cache: Rc<PrerenderCache>) -> Self {
        Self {
            cache,
            queue: RefCell::new(Vec::new()),
        }
    }

    /// Enqueue a prerender request.
    pub fn enqueue(&self, key: &str, priority: PrerenderPriority) {
        self.queue.borrow_mut().push(PrerenderRequest {
            key: key.to_string(),
            priority,
        });
        // Sort by priority (highest first)
        self.queue.borrow_mut().sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Process the next prerender request in the queue.
    pub fn process_next<F: FnOnce(&str) -> String>(&self, render_fn: F) -> Option<String> {
        let request = self.queue.borrow_mut().first().cloned();
        if let Some(req) = request {
            self.queue.borrow_mut().remove(0);
            let content = render_fn(&req.key);
            self.cache.prerender(&req.key, || content.clone());
            Some(content)
        } else {
            None
        }
    }

    /// Process all queued prerender requests.
    pub fn process_all<F: Fn(&str) -> String>(&self, render_fn: F) {
        let requests: Vec<PrerenderRequest> = self.queue.borrow_mut().drain(..).collect();
        for req in requests {
            let content = render_fn(&req.key);
            self.cache.prerender(&req.key, || content);
        }
    }

    /// Get the number of pending requests.
    pub fn pending_count(&self) -> usize {
        self.queue.borrow().len()
    }

    /// Clear the queue.
    pub fn clear_queue(&self) {
        self.queue.borrow_mut().clear();
    }

    /// Get a reference to the cache.
    pub fn cache(&self) -> &PrerenderCache {
        &self.cache
    }
}

// Global prerender cache for convenience.
thread_local! {
    static GLOBAL_CACHE: RefCell<Option<Rc<PrerenderCache>>> = const { RefCell::new(None) };
}

/// Initialize the global prerender cache.
pub fn init_global_cache() {
    GLOBAL_CACHE.with(|c| {
        *c.borrow_mut() = Some(Rc::new(PrerenderCache::new()));
    });
}

/// Get the global prerender cache, if initialized.
pub fn global_cache() -> Option<Rc<PrerenderCache>> {
    GLOBAL_CACHE.with(|c| c.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prerender_cache_basic() {
        let cache = PrerenderCache::new();
        cache.prerender("/about", || "<h1>About</h1>".to_string());
        assert!(cache.has("/about"));
        assert_eq!(cache.peek("/about"), Some("<h1>About</h1>".to_string()));
    }

    #[test]
    fn test_prerender_cache_take() {
        let cache = PrerenderCache::new();
        cache.prerender("/home", || "<h1>Home</h1>".to_string());
        let node = cache.take("/home").unwrap();
        assert_eq!(node.content, "<h1>Home</h1>");
        assert!(node.consumed);
        // Second take returns None
        assert!(cache.take("/home").is_none());
    }

    #[test]
    fn test_prerender_cache_evict() {
        let cache = PrerenderCache::new();
        cache.prerender("/temp", || "temp".to_string());
        assert!(cache.has("/temp"));
        cache.evict("/temp");
        assert!(!cache.has("/temp"));
    }

    #[test]
    fn test_prerender_cache_clear() {
        let cache = PrerenderCache::new();
        cache.prerender("a", || "a".to_string());
        cache.prerender("b", || "b".to_string());
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_prerender_cache_keys() {
        let cache = PrerenderCache::new();
        cache.prerender("x", || "x".to_string());
        cache.prerender("y", || "y".to_string());
        let keys = cache.keys();
        assert!(keys.contains(&"x".to_string()));
        assert!(keys.contains(&"y".to_string()));
    }

    #[test]
    fn test_prerender_scheduler_enqueue_process() {
        let cache = Rc::new(PrerenderCache::new());
        let scheduler = PrerenderScheduler::new(Rc::clone(&cache));

        scheduler.enqueue("/about", PrerenderPriority::Normal);
        scheduler.enqueue("/contact", PrerenderPriority::High);

        // High priority should be processed first
        let result = scheduler.process_next(|key| format!("<h1>{}</h1>", key));
        assert!(result.is_some());
        assert!(cache.has("/contact"));
    }

    #[test]
    fn test_prerender_scheduler_process_all() {
        let cache = Rc::new(PrerenderCache::new());
        let scheduler = PrerenderScheduler::new(Rc::clone(&cache));

        scheduler.enqueue("a", PrerenderPriority::Normal);
        scheduler.enqueue("b", PrerenderPriority::Normal);
        scheduler.enqueue("c", PrerenderPriority::Normal);

        scheduler.process_all(|key| format!("content-{}", key));

        assert_eq!(cache.len(), 3);
        assert_eq!(scheduler.pending_count(), 0);
    }

    #[test]
    fn test_prerender_scheduler_clear_queue() {
        let cache = Rc::new(PrerenderCache::new());
        let scheduler = PrerenderScheduler::new(cache);
        scheduler.enqueue("a", PrerenderPriority::Normal);
        scheduler.enqueue("b", PrerenderPriority::Normal);
        assert_eq!(scheduler.pending_count(), 2);
        scheduler.clear_queue();
        assert_eq!(scheduler.pending_count(), 0);
    }

    #[test]
    fn test_prerender_priority_ordering() {
        assert!(PrerenderPriority::High > PrerenderPriority::Normal);
        assert!(PrerenderPriority::Normal > PrerenderPriority::Idle);
    }

    #[test]
    fn test_global_cache() {
        init_global_cache();
        let cache = global_cache().unwrap();
        cache.prerender("/global", || "global".to_string());
        assert!(cache.has("/global"));
    }
}
