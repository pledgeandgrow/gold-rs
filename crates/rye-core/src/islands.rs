//! Islands architecture — selective hydration for performance.
//!
//! In a traditional SSR+hydration setup, the entire page must be hydrated
//! on the client before any interactivity is available. This is slow for
//! content-heavy pages with small interactive regions.
//!
//! The islands model solves this:
//! - Static HTML is sent as-is (no JS needed)
//! - Interactive components ("islands") are individually hydrated
//! - Each island loads its Wasm chunk lazily on demand
//! - Non-interactive content never ships JS/Wasm
//!
//! ## How it works
//!
//! 1. **SSR**: `Island` components render their content and emit a
//!    `<div data-rye-island="component_id" data-rye-props="serialized_props">`
//!    wrapper around their output.
//!
//! 2. **Client**: A small bootstrap script scans for `[data-rye-island]`
//!    elements, dynamically imports the corresponding Wasm chunk, and
//!    hydrates only that island's DOM subtree.
//!
//! 3. **Lazy loading**: Each island's Wasm is code-split into a separate
//!    `.wasm` file, loaded only when the island is visible or interacted with.
//!
//! ## Usage
//!
//! ```ignore
//! // Server: render page with islands
//! use rye_core::islands::{Island, IslandRegistry};
//!
//! // Static content (no JS shipped)
//! template! {
//!     h1 { "Blog Post" }
//!     p { "Static content..." }
//!
//!     // Interactive island (hydrated lazily)
//!     Island::new("comments", || {
//!         template! { Comments { post_id: 42 } }
//!     })
//! }
//! ```
//!
//! ## Performance
//!
//! | Metric | Full hydration | Islands |
//! |--------|---------------|---------|
//! | JS shipped | Full bundle | Only interactive chunks |
//! | Time to interactive | Slow (full hydrate) | Fast (per-island) |
//! | Wasm parse | Entire binary | Only needed chunks |

use std::collections::HashMap;
use std::sync::Mutex;

/// Unique identifier for an island.
pub type IslandId = String;

/// Metadata about an island embedded in SSR HTML.
#[derive(Debug, Clone)]
pub struct IslandMeta {
    /// Unique island identifier (typically the component name).
    pub id: IslandId,
    /// Serialized props for the island component.
    pub props: String,
    /// Wasm chunk URL for lazy loading (empty if inline).
    pub chunk_url: String,
    /// Whether to hydrate on visible (IntersectionObserver) or on load.
    pub strategy: HydrationStrategy,
}

/// When an island should be hydrated on the client.
#[derive(Debug, Clone, PartialEq)]
pub enum HydrationStrategy {
    /// Hydrate immediately when the bootstrap script runs.
    OnLoad,
    /// Hydrate when the island scrolls into view (lazy).
    OnVisible,
    /// Hydrate on first user interaction (click, focus, etc.).
    OnIdle,
    /// Hydrate only when explicitly triggered by user code.
    Manual,
}

impl Default for HydrationStrategy {
    fn default() -> Self {
        HydrationStrategy::OnLoad
    }
}

/// Registry of island components available for hydration.
///
/// On the server, this maps island IDs to their SSR render functions.
/// On the client, this maps island IDs to their hydration functions.
pub struct IslandRegistry {
    /// Map of island ID → SSR render function.
    ssr_renderers: HashMap<IslandId, Box<dyn Fn(&str) -> String + Send + Sync>>,
    /// Map of island ID → chunk URL.
    chunk_urls: HashMap<IslandId, String>,
}

static REGISTRY: Mutex<Option<IslandRegistry>> = Mutex::new(None);

impl IslandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            ssr_renderers: HashMap::new(),
            chunk_urls: HashMap::new(),
        }
    }

    /// Register an island component with its SSR render function.
    pub fn register<F>(&mut self, id: &str, chunk_url: &str, render_fn: F)
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.ssr_renderers
            .insert(id.to_string(), Box::new(render_fn));
        self.chunk_urls
            .insert(id.to_string(), chunk_url.to_string());
    }

    /// SSR-render an island by ID with serialized props.
    pub fn render(&self, id: &str, props: &str) -> Option<String> {
        let render_fn = self.ssr_renderers.get(id)?;
        let chunk_url = self.chunk_urls.get(id).map(|s| s.as_str()).unwrap_or("");
        let content = render_fn(props);
        Some(format!(
            r#"<div data-rye-island="{}" data-rye-props="{}" data-rye-chunk="{}">{}</div>"#,
            html_escape_attr(id),
            html_escape_attr(props),
            html_escape_attr(chunk_url),
            content
        ))
    }

    /// Get the chunk URL for an island.
    pub fn chunk_url(&self, id: &str) -> Option<&str> {
        self.chunk_urls.get(id).map(|s| s.as_str())
    }

    /// List all registered island IDs.
    pub fn ids(&self) -> Vec<&str> {
        self.ssr_renderers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for IslandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global island registry.
pub fn init_registry(registry: IslandRegistry) {
    let mut global = REGISTRY.lock().unwrap();
    *global = Some(registry);
}

/// SSR-render an island using the global registry.
pub fn render_island(id: &str, props: &str) -> Option<String> {
    let global = REGISTRY.lock().unwrap();
    global.as_ref().and_then(|reg| reg.render(id, props))
}

/// An island component wrapper for use in templates.
///
/// Wraps interactive content with `data-rye-island` attributes for
/// selective client-side hydration.
pub struct Island {
    /// Island identifier.
    id: IslandId,
    /// Serialized props.
    props: String,
    /// Hydration strategy.
    strategy: HydrationStrategy,
    /// Pre-rendered SSR content.
    ssr_content: String,
}

impl Island {
    /// Create a new island with the given ID and SSR content.
    pub fn new(id: impl Into<String>, ssr_content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            props: String::new(),
            strategy: HydrationStrategy::OnLoad,
            ssr_content: ssr_content.into(),
        }
    }

    /// Set serialized props for the island.
    pub fn with_props(mut self, props: impl Into<String>) -> Self {
        self.props = props.into();
        self
    }

    /// Set the hydration strategy.
    pub fn with_strategy(mut self, strategy: HydrationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Render the island to SSR HTML with hydration markers.
    pub fn to_html(&self) -> String {
        let strategy_attr = match &self.strategy {
            HydrationStrategy::OnLoad => "load",
            HydrationStrategy::OnVisible => "visible",
            HydrationStrategy::OnIdle => "idle",
            HydrationStrategy::Manual => "manual",
        };

        format!(
            r#"<div data-rye-island="{}" data-rye-props="{}" data-rye-hydrate="{}">{}</div>"#,
            html_escape_attr(&self.id),
            html_escape_attr(&self.props),
            strategy_attr,
            self.ssr_content
        )
    }
}

/// Generate the client-side bootstrap script for islands.
///
/// This script scans for `[data-rye-island]` elements and hydrates
/// them according to their `data-rye-hydrate` strategy.
pub fn bootstrap_script() -> &'static str {
    r#"<script>
(function() {
    function hydrateIsland(el) {
        var id = el.getAttribute('data-rye-island');
        var props = el.getAttribute('data-rye-props') || '';
        var chunk = el.getAttribute('data-rye-chunk') || '';
        if (chunk) {
            import(chunk).then(function(mod) {
                if (mod.hydrate) mod.hydrate(el, props);
            }).catch(function(err) {
                console.error('Failed to hydrate island ' + id + ':', err);
            });
        }
    }

    function setupIdleHydration(el) {
        requestIdleCallback(function() { hydrateIsland(el); });
    }

    function setupVisibleHydration(el) {
        if ('IntersectionObserver' in window) {
            var observer = new IntersectionObserver(function(entries) {
                entries.forEach(function(entry) {
                    if (entry.isIntersecting) {
                        observer.unobserve(el);
                        hydrateIsland(el);
                    }
                });
            });
            observer.observe(el);
        } else {
            hydrateIsland(el);
        }
    }

    function init() {
        var islands = document.querySelectorAll('[data-rye-island]');
        islands.forEach(function(el) {
            var strategy = el.getAttribute('data-rye-hydrate') || 'load';
            switch (strategy) {
                case 'load': hydrateIsland(el); break;
                case 'idle': setupIdleHydration(el); break;
                case 'visible': setupVisibleHydration(el); break;
                case 'manual': break;
            }
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
</script>"#
}

/// Escape a string for use in an HTML attribute value.
fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_island_html() {
        let island = Island::new("counter", "<button>0</button>")
            .with_props("42")
            .with_strategy(HydrationStrategy::OnLoad);

        let html = island.to_html();
        assert!(html.contains("data-rye-island=\"counter\""));
        assert!(html.contains("data-rye-props=\"42\""));
        assert!(html.contains("data-rye-hydrate=\"load\""));
        assert!(html.contains("<button>0</button>"));
    }

    #[test]
    fn test_island_lazy_strategy() {
        let island = Island::new("comments", "<div>Comments</div>")
            .with_strategy(HydrationStrategy::OnVisible);

        let html = island.to_html();
        assert!(html.contains("data-rye-hydrate=\"visible\""));
    }

    #[test]
    fn test_island_manual_strategy() {
        let island =
            Island::new("widget", "<div>Widget</div>").with_strategy(HydrationStrategy::Manual);

        let html = island.to_html();
        assert!(html.contains("data-rye-hydrate=\"manual\""));
    }

    #[test]
    fn test_island_html_escaping() {
        let island = Island::new("test", "<p>content</p>").with_props("\"quoted\" & <tagged>");

        let html = island.to_html();
        assert!(html.contains("&quot;quoted&quot;"));
        assert!(html.contains("&amp;"));
        assert!(html.contains("&lt;tagged&gt;"));
    }

    #[test]
    fn test_registry_register_and_render() {
        let mut registry = IslandRegistry::new();
        registry.register("counter", "/chunks/counter.wasm", |props| {
            format!("<button>Count: {}</button>", props)
        });

        let html = registry.render("counter", "42").unwrap();
        assert!(html.contains("data-rye-island=\"counter\""));
        assert!(html.contains("data-rye-chunk=\"/chunks/counter.wasm\""));
        assert!(html.contains("<button>Count: 42</button>"));
    }

    #[test]
    fn test_registry_unknown_island() {
        let registry = IslandRegistry::new();
        assert!(registry.render("nonexistent", "").is_none());
    }

    #[test]
    fn test_registry_ids() {
        let mut registry = IslandRegistry::new();
        registry.register("a", "/a.wasm", |_| "<div>A</div>".to_string());
        registry.register("b", "/b.wasm", |_| "<div>B</div>".to_string());

        let ids = registry.ids();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_bootstrap_script_contains_islands_logic() {
        let script = bootstrap_script();
        assert!(script.contains("data-rye-island"));
        assert!(script.contains("hydrateIsland"));
        assert!(script.contains("IntersectionObserver"));
        assert!(script.contains("requestIdleCallback"));
    }
}
