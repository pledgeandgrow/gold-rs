//! Goal 205: Native deep linking.
//!
//! Handle universal links (iOS), app links (Android), and URL schemes.
//! `use_deep_link()` hook that fires when the app is opened via a URL.

use std::collections::HashMap;
use std::sync::Mutex;

/// A deep link that opened the app.
#[derive(Debug, Clone, PartialEq)]
pub struct DeepLink {
    /// The full URL.
    pub url: String,
    /// The URL scheme (e.g. "https", "myapp").
    pub scheme: String,
    /// The host (e.g. "example.com").
    pub host: String,
    /// The path segments.
    pub path: Vec<String>,
    /// Query parameters.
    pub query: HashMap<String, String>,
    /// Whether this is a universal link (https) or a custom scheme.
    pub is_universal: bool,
}

impl DeepLink {
    /// Parse a URL string into a DeepLink.
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }

        // Parse scheme
        let scheme_end = url.find("://")?;
        let scheme = url[..scheme_end].to_lowercase();
        let rest = &url[scheme_end + 3..];

        // Parse host and path
        let (host_and_path, query_str) = if let Some(q_pos) = rest.find('?') {
            (&rest[..q_pos], Some(&rest[q_pos + 1..]))
        } else {
            (rest, None)
        };

        let (host, path_str) = if let Some(p_pos) = host_and_path.find('/') {
            (&host_and_path[..p_pos], &host_and_path[p_pos..])
        } else {
            (host_and_path, "")
        };

        let path: Vec<String> = if path_str.is_empty() {
            Vec::new()
        } else {
            path_str
                .trim_start_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        };

        let mut query = HashMap::new();
        if let Some(q) = query_str {
            for pair in q.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = pair[..eq_pos].to_string();
                    let value = pair[eq_pos + 1..].to_string();
                    query.insert(key, value);
                } else if !pair.is_empty() {
                    query.insert(pair.to_string(), String::new());
                }
            }
        }

        let is_universal = scheme == "http" || scheme == "https";

        Some(Self {
            url: url.to_string(),
            scheme,
            host: host.to_string(),
            path,
            query,
            is_universal,
        })
    }

    /// Get a path segment by index.
    pub fn path_segment(&self, index: usize) -> Option<&str> {
        self.path.get(index).map(|s| s.as_str())
    }

    /// Get a query parameter.
    pub fn query_param(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    /// Reconstruct the path as a string.
    pub fn path_string(&self) -> String {
        if self.path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", self.path.join("/"))
        }
    }
}

/// A deep link route handler.
pub struct DeepLinkRoute {
    /// The path pattern (e.g. "/products/:id").
    pub pattern: String,
    /// The handler function.
    handler: Box<dyn Fn(&DeepLink) -> bool + Send + Sync>,
}

impl DeepLinkRoute {
    /// Create a new deep link route.
    pub fn new<F: Fn(&DeepLink) -> bool + Send + Sync + 'static>(pattern: &str, handler: F) -> Self {
        Self {
            pattern: pattern.to_string(),
            handler: Box::new(handler),
        }
    }

    /// Check if this route matches a deep link.
    pub fn matches(&self, link: &DeepLink) -> bool {
        let pattern_segments: Vec<&str> = self
            .pattern
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if pattern_segments.len() != link.path.len() {
            return false;
        }

        for (p, l) in pattern_segments.iter().zip(link.path.iter()) {
            if p.starts_with(':') {
                continue; // Parameter, matches anything
            }
            if *p != l {
                return false;
            }
        }

        true
    }

    /// Handle the deep link.
    pub fn handle(&self, link: &DeepLink) -> bool {
        (self.handler)(link)
    }

    /// Extract parameters from the deep link based on the pattern.
    pub fn extract_params(&self, link: &DeepLink) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let pattern_segments: Vec<&str> = self
            .pattern
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        for (p, l) in pattern_segments.iter().zip(link.path.iter()) {
            if p.starts_with(':') {
                params.insert(p[1..].to_string(), l.clone());
            }
        }

        params
    }
}

impl std::fmt::Debug for DeepLinkRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeepLinkRoute")
            .field("pattern", &self.pattern)
            .finish()
    }
}

/// The deep link manager — handles incoming deep links and routes them.
pub struct DeepLinkManager {
    routes: Mutex<Vec<DeepLinkRoute>>,
    last_link: Mutex<Option<DeepLink>>,
    handled_count: Mutex<u32>,
}

impl DeepLinkManager {
    /// Create a new deep link manager.
    pub fn new() -> Self {
        Self {
            routes: Mutex::new(Vec::new()),
            last_link: Mutex::new(None),
            handled_count: Mutex::new(0),
        }
    }

    /// Register a route.
    pub fn register_route(&self, route: DeepLinkRoute) {
        self.routes.lock().unwrap().push(route);
    }

    /// Handle an incoming deep link URL.
    pub fn handle_url(&self, url: &str) -> bool {
        let link = match DeepLink::parse(url) {
            Some(l) => l,
            None => return false,
        };

        *self.last_link.lock().unwrap() = Some(link.clone());

        let routes = self.routes.lock().unwrap();
        for route in routes.iter() {
            if route.matches(&link) {
                if route.handle(&link) {
                    *self.handled_count.lock().unwrap() += 1;
                    return true;
                }
            }
        }

        false
    }

    /// Get the last received deep link.
    pub fn last_link(&self) -> Option<DeepLink> {
        self.last_link.lock().unwrap().clone()
    }

    /// Get the number of handled deep links.
    pub fn handled_count(&self) -> u32 {
        *self.handled_count.lock().unwrap()
    }

    /// Get the number of registered routes.
    pub fn route_count(&self) -> usize {
        self.routes.lock().unwrap().len()
    }

    /// Clear all routes.
    pub fn clear_routes(&self) {
        self.routes.lock().unwrap().clear();
    }
}

impl Default for DeepLinkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_link_parse_https() {
        let link = DeepLink::parse("https://example.com/products/42?ref=email").unwrap();
        assert_eq!(link.scheme, "https");
        assert_eq!(link.host, "example.com");
        assert_eq!(link.path, vec!["products", "42"]);
        assert_eq!(link.query_param("ref"), Some("email"));
        assert!(link.is_universal);
    }

    #[test]
    fn test_deep_link_parse_custom_scheme() {
        let link = DeepLink::parse("myapp://open/page").unwrap();
        assert_eq!(link.scheme, "myapp");
        assert_eq!(link.host, "open");
        assert_eq!(link.path, vec!["page"]);
        assert!(!link.is_universal);
    }

    #[test]
    fn test_deep_link_parse_empty() {
        assert!(DeepLink::parse("").is_none());
    }

    #[test]
    fn test_deep_link_parse_no_path() {
        let link = DeepLink::parse("https://example.com").unwrap();
        assert_eq!(link.host, "example.com");
        assert!(link.path.is_empty());
        assert_eq!(link.path_string(), "/");
    }

    #[test]
    fn test_deep_link_parse_multiple_query() {
        let link = DeepLink::parse("https://example.com/search?q=hello&page=2").unwrap();
        assert_eq!(link.query_param("q"), Some("hello"));
        assert_eq!(link.query_param("page"), Some("2"));
    }

    #[test]
    fn test_deep_link_path_segment() {
        let link = DeepLink::parse("https://example.com/a/b/c").unwrap();
        assert_eq!(link.path_segment(0), Some("a"));
        assert_eq!(link.path_segment(1), Some("b"));
        assert_eq!(link.path_segment(2), Some("c"));
        assert_eq!(link.path_segment(3), None);
    }

    #[test]
    fn test_deep_link_path_string() {
        let link = DeepLink::parse("https://example.com/a/b").unwrap();
        assert_eq!(link.path_string(), "/a/b");
    }

    #[test]
    fn test_route_matches_exact() {
        let route = DeepLinkRoute::new("/about", |_| true);
        let link = DeepLink::parse("https://example.com/about").unwrap();
        assert!(route.matches(&link));
    }

    #[test]
    fn test_route_matches_param() {
        let route = DeepLinkRoute::new("/products/:id", |_| true);
        let link = DeepLink::parse("https://example.com/products/42").unwrap();
        assert!(route.matches(&link));
    }

    #[test]
    fn test_route_matches_no_match() {
        let route = DeepLinkRoute::new("/products/:id", |_| true);
        let link = DeepLink::parse("https://example.com/about").unwrap();
        assert!(!route.matches(&link));
    }

    #[test]
    fn test_route_matches_different_length() {
        let route = DeepLinkRoute::new("/a/b", |_| true);
        let link = DeepLink::parse("https://example.com/a").unwrap();
        assert!(!route.matches(&link));
    }

    #[test]
    fn test_route_extract_params() {
        let route = DeepLinkRoute::new("/users/:userId/posts/:postId", |_| true);
        let link = DeepLink::parse("https://example.com/users/42/posts/99").unwrap();
        let params = route.extract_params(&link);
        assert_eq!(params.get("userId"), Some(&"42".to_string()));
        assert_eq!(params.get("postId"), Some(&"99".to_string()));
    }

    #[test]
    fn test_route_handle() {
        let route = DeepLinkRoute::new("/test", |link| link.host == "example.com");
        let link = DeepLink::parse("https://example.com/test").unwrap();
        assert!(route.handle(&link));
    }

    #[test]
    fn test_manager_register_route() {
        let mgr = DeepLinkManager::new();
        mgr.register_route(DeepLinkRoute::new("/about", |_| true));
        assert_eq!(mgr.route_count(), 1);
    }

    #[test]
    fn test_manager_handle_url() {
        let mgr = DeepLinkManager::new();
        mgr.register_route(DeepLinkRoute::new("/products/:id", |_| true));
        assert!(mgr.handle_url("https://example.com/products/42"));
        assert!(!mgr.handle_url("https://example.com/nonexistent"));
    }

    #[test]
    fn test_manager_handle_invalid_url() {
        let mgr = DeepLinkManager::new();
        assert!(!mgr.handle_url("not-a-url"));
    }

    #[test]
    fn test_manager_last_link() {
        let mgr = DeepLinkManager::new();
        mgr.register_route(DeepLinkRoute::new("/test", |_| true));
        mgr.handle_url("https://example.com/test");
        assert!(mgr.last_link().is_some());
        assert_eq!(mgr.last_link().unwrap().host, "example.com");
    }

    #[test]
    fn test_manager_handled_count() {
        let mgr = DeepLinkManager::new();
        mgr.register_route(DeepLinkRoute::new("/test", |_| true));
        mgr.handle_url("https://example.com/test");
        mgr.handle_url("https://example.com/test");
        assert_eq!(mgr.handled_count(), 2);
    }

    #[test]
    fn test_manager_clear_routes() {
        let mgr = DeepLinkManager::new();
        mgr.register_route(DeepLinkRoute::new("/a", |_| true));
        mgr.register_route(DeepLinkRoute::new("/b", |_| true));
        mgr.clear_routes();
        assert_eq!(mgr.route_count(), 0);
    }
}
