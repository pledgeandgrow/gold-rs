//! Goal 127: Server-side data prefetching.
//!
//! `prefetch!()` macro and `PrefetchContext` for loading data during SSR
//! and passing it to the client for hydration. Avoids client-side loading
//! states for initial page load.

use std::collections::HashMap;

/// Prefetched data entry.
#[derive(Debug, Clone)]
pub struct PrefetchedData {
    /// Key (usually a query/path identifier).
    pub key: String,
    /// Serialized data (JSON).
    pub data: String,
    /// Whether the data was loaded successfully.
    pub success: bool,
    /// Error message if loading failed.
    pub error: Option<String>,
}

/// Prefetch context — collects data during SSR for client hydration.
pub struct PrefetchContext {
    /// Prefetched data keyed by query key.
    data: HashMap<String, PrefetchedData>,
}

impl PrefetchContext {
    /// Create a new empty prefetch context.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Add prefetched data.
    pub fn insert(&mut self, key: impl Into<String>, data: impl Into<String>) {
        let key = key.into();
        self.data.insert(
            key.clone(),
            PrefetchedData {
                key,
                data: data.into(),
                success: true,
                error: None,
            },
        );
    }

    /// Add a prefetch error.
    pub fn insert_error(&mut self, key: impl Into<String>, error: impl Into<String>) {
        let key = key.into();
        self.data.insert(
            key.clone(),
            PrefetchedData {
                key,
                data: String::new(),
                success: false,
                error: Some(error.into()),
            },
        );
    }

    /// Get prefetched data by key.
    pub fn get(&self, key: &str) -> Option<&PrefetchedData> {
        self.data.get(key)
    }

    /// Serialize all prefetched data for client hydration.
    pub fn to_json(&self) -> String {
        let mut entries = Vec::new();
        for (key, data) in &self.data {
            let escaped_data = data.data.replace('\\', "\\\\").replace('"', "\\\"");
            let entry = if data.success {
                format!(r#""{}":{{"data":"{}","success":true}}"#, key, escaped_data)
            } else if let Some(err) = &data.error {
                let escaped_err = err.replace('\\', "\\\\").replace('"', "\\\"");
                format!(
                    r#""{}":{{"data":"","success":false,"error":"{}"}}"#,
                    key, escaped_err
                )
            } else {
                format!(r#""{}":{{"data":"","success":false}}"#, key)
            };
            entries.push(entry);
        }
        format!("{{{}}}", entries.join(","))
    }

    /// Generate the script tag for injecting prefetched data.
    pub fn to_script_tag(&self) -> String {
        format!(
            r#"<script>window.__RYE_PREFETCHED__ = {};</script>"#,
            self.to_json()
        )
    }

    /// Number of prefetched entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the context is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for PrefetchContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Prefetch rule — declares what data to prefetch for a route.
#[derive(Debug, Clone)]
pub struct PrefetchRule {
    /// Route pattern (e.g. "/users/:id").
    pub route: String,
    /// Data keys to prefetch.
    pub keys: Vec<String>,
    /// Whether to prefetch on link hover.
    pub on_hover: bool,
    /// Whether to prefetch on link visible.
    pub on_visible: bool,
}

impl PrefetchRule {
    /// Create a new prefetch rule for a route.
    pub fn for_route(route: impl Into<String>) -> Self {
        Self {
            route: route.into(),
            keys: Vec::new(),
            on_hover: true,
            on_visible: false,
        }
    }

    /// Add a data key to prefetch.
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.keys.push(key.into());
        self
    }

    /// Enable prefetch on link visible.
    pub fn on_visible(mut self) -> Self {
        self.on_visible = true;
        self
    }
}

/// Generate the JS for client-side prefetching on hover/visible.
pub fn prefetch_script() -> &'static str {
    r#"<script>
(function() {
  var prefetched = new Set();

  function prefetchRoute(href) {
    if (prefetched.has(href)) return;
    prefetched.add(href);

    // Use fetch with low priority for prefetching
    fetch(href, {
      headers: { 'X-Rye-Prefetch': 'true' },
      priority: 'low'
    }).catch(function() {});
  }

  // Prefetch on hover
  document.addEventListener('mouseover', function(e) {
    var link = e.target.closest('a[data-prefetch]');
    if (link && link.href) {
      prefetchRoute(link.href);
    }
  }, { passive: true });

  // Prefetch on visible (using IntersectionObserver)
  if ('IntersectionObserver' in window) {
    var observer = new IntersectionObserver(function(entries) {
      entries.forEach(function(entry) {
        if (entry.isIntersecting) {
          var link = entry.target;
          if (link.href) {
            prefetchRoute(link.href);
            observer.unobserve(link);
          }
        }
      });
    }, { rootMargin: '200px' });

    document.querySelectorAll('a[data-prefetch-visible]').forEach(function(link) {
      observer.observe(link);
    });
  }
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_context_insert() {
        let mut ctx = PrefetchContext::new();
        ctx.insert("users/123", r#"{"name":"Alice"}"#);
        assert_eq!(ctx.len(), 1);
        assert!(ctx.get("users/123").unwrap().success);
    }

    #[test]
    fn test_prefetch_context_error() {
        let mut ctx = PrefetchContext::new();
        ctx.insert_error("users/999", "Not found");
        let data = ctx.get("users/999").unwrap();
        assert!(!data.success);
        assert_eq!(data.error, Some("Not found".to_string()));
    }

    #[test]
    fn test_prefetch_context_json() {
        let mut ctx = PrefetchContext::new();
        ctx.insert("key1", "value1");
        ctx.insert("key2", "value2");
        let json = ctx.to_json();
        assert!(json.contains("key1"));
        assert!(json.contains("value1"));
        assert!(json.contains("key2"));
        assert!(json.contains("success\":true"));
    }

    #[test]
    fn test_prefetch_context_script_tag() {
        let mut ctx = PrefetchContext::new();
        ctx.insert("data", "test");
        let tag = ctx.to_script_tag();
        assert!(tag.contains("__RYE_PREFETCHED__"));
        assert!(tag.contains("test"));
    }

    #[test]
    fn test_prefetch_rule() {
        let rule = PrefetchRule::for_route("/users/:id")
            .key("user_profile")
            .key("user_posts")
            .on_visible();
        assert_eq!(rule.route, "/users/:id");
        assert_eq!(rule.keys, vec!["user_profile", "user_posts"]);
        assert!(rule.on_visible);
    }

    #[test]
    fn test_prefetch_script() {
        let script = prefetch_script();
        assert!(script.contains("IntersectionObserver"));
        assert!(script.contains("data-prefetch"));
        assert!(script.contains("X-Rye-Prefetch"));
    }
}
