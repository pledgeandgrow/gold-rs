//! Goal 186: Server-side rendering with data loading patterns.
//!
//! `Loader` trait that runs on the server before rendering a route.
//! Type-safe route loaders that fetch data, validate auth, and prefetch resources.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// A boxed future for loader results.
pub type LoaderFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Data loaded by a route loader — passed to the component for rendering.
#[derive(Debug, Clone)]
pub struct LoaderData {
    /// The loaded data, serialized as JSON.
    pub data: String,
    /// Additional metadata (e.g. cache hints, status codes).
    pub meta: LoaderMeta,
}

/// Metadata from a loader.
#[derive(Debug, Clone, Default)]
pub struct LoaderMeta {
    /// HTTP status code override.
    pub status: Option<u16>,
    /// Cache control header value.
    pub cache_control: Option<String>,
    /// Redirect URL (if the loader decides to redirect).
    pub redirect: Option<String>,
    /// Custom headers to set on the response.
    pub headers: HashMap<String, String>,
}

impl LoaderData {
    /// Create new loader data from a JSON string.
    pub fn json(data: impl Into<String>) -> Self {
        Self {
            data: data.into(),
            meta: LoaderMeta::default(),
        }
    }

    /// Set a custom status code.
    pub fn with_status(mut self, status: u16) -> Self {
        self.meta.status = Some(status);
        self
    }

    /// Set a redirect.
    pub fn with_redirect(mut self, url: impl Into<String>) -> Self {
        self.meta.redirect = Some(url.into());
        self
    }

    /// Set cache control.
    pub fn with_cache_control(mut self, cc: impl Into<String>) -> Self {
        self.meta.cache_control = Some(cc.into());
        self
    }

    /// Add a custom header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.meta.headers.insert(key.into(), value.into());
        self
    }
}

/// The Loader trait — runs on the server before rendering a route.
///
/// Implement this trait for each route that needs server-side data loading.
/// The loader receives the route parameters and request, and returns
/// data that will be passed to the component.
pub trait Loader: Send + Sync {
    /// The route path this loader handles (e.g. "/users/:id").
    fn path(&self) -> &str;

    /// Load data for this route.
    fn load(&self, params: &RouteParams, req: &LoaderRequest) -> LoaderFuture<LoaderResult>;

    /// Whether this loader's data should be cached.
    fn cacheable(&self) -> bool {
        false
    }
}

/// Route parameters extracted from the URL.
pub type RouteParams = HashMap<String, String>;

/// A loader request — wraps the HTTP request with additional context.
#[derive(Debug, Clone)]
pub struct LoaderRequest {
    /// The HTTP method.
    pub method: String,
    /// The full request path.
    pub path: String,
    /// Query parameters.
    pub query: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// The session ID, if authenticated.
    pub session_id: Option<String>,
}

impl LoaderRequest {
    /// Create a new GET loader request.
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            session_id: None,
        }
    }

    /// Set the session ID.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add a query parameter.
    pub fn with_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }
}

/// The result of a loader — either data or an error/redirect.
#[derive(Debug, Clone)]
pub enum LoaderResult {
    /// Data loaded successfully.
    Ok(LoaderData),
    /// Loader returned an error.
    Error { message: String, status: u16 },
    /// Loader decided to redirect.
    Redirect { url: String, status: u16 },
}

impl LoaderResult {
    /// Create an OK result.
    pub fn ok(data: LoaderData) -> Self {
        LoaderResult::Ok(data)
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>, status: u16) -> Self {
        LoaderResult::Error {
            message: message.into(),
            status,
        }
    }

    /// Create a redirect result.
    pub fn redirect(url: impl Into<String>, status: u16) -> Self {
        LoaderResult::Redirect {
            url: url.into(),
            status,
        }
    }

    /// Check if the result is OK.
    pub fn is_ok(&self) -> bool {
        matches!(self, LoaderResult::Ok(_))
    }

    /// Check if the result is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, LoaderResult::Error { .. })
    }

    /// Check if the result is a redirect.
    pub fn is_redirect(&self) -> bool {
        matches!(self, LoaderResult::Redirect { .. })
    }
}

/// The loader registry — maps route patterns to loaders.
pub struct LoaderRegistry {
    loaders: Vec<Box<dyn Loader>>,
}

impl LoaderRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// Register a loader.
    pub fn register<L: Loader + 'static>(&mut self, loader: L) {
        self.loaders.push(Box::new(loader));
    }

    /// Find a loader for the given path.
    pub fn find_loader(&self, path: &str) -> Option<&dyn Loader> {
        for loader in &self.loaders {
            if route_matches(loader.path(), path) {
                return Some(loader.as_ref());
            }
        }
        None
    }

    /// Run the loader for the given path.
    pub async fn load_for(&self, path: &str, req: &LoaderRequest) -> Option<LoaderResult> {
        let loader = self.find_loader(path)?;
        let params = extract_params(loader.path(), path);
        Some(loader.load(&params, req).await)
    }

    /// Get the number of registered loaders.
    pub fn len(&self) -> usize {
        self.loaders.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.loaders.is_empty()
    }
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a route pattern matches a path.
/// Supports `:param` segments.
pub fn route_matches(pattern: &str, path: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pattern_parts.len() != path_parts.len() {
        return false;
    }

    for (pp, ap) in pattern_parts.iter().zip(path_parts.iter()) {
        if pp.starts_with(':') {
            continue; // wildcard match
        }
        if pp != ap {
            return false;
        }
    }

    true
}

/// Extract route parameters from a path based on a pattern.
pub fn extract_params(pattern: &str, path: &str) -> RouteParams {
    let mut params = HashMap::new();
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    for (pp, ap) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(name) = pp.strip_prefix(':') {
            params.insert(name.to_string(), ap.to_string());
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UserLoader;

    impl Loader for UserLoader {
        fn path(&self) -> &str {
            "/users/:id"
        }

        fn load(&self, params: &RouteParams, _req: &LoaderRequest) -> LoaderFuture<LoaderResult> {
            let id = params.get("id").cloned().unwrap_or_default();
            Box::pin(async move {
                LoaderResult::ok(LoaderData::json(format!("{{\"user_id\":\"{}\"}}", id)))
            })
        }
    }

    #[tokio::test]
    async fn test_loader_basic() {
        let loader = UserLoader;
        let params = extract_params("/users/:id", "/users/42");
        let req = LoaderRequest::get("/users/42");
        let result = loader.load(&params, &req).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_loader_registry() {
        let mut registry = LoaderRegistry::new();
        registry.register(UserLoader);
        assert_eq!(registry.len(), 1);

        let req = LoaderRequest::get("/users/123");
        let result = registry.load_for("/users/123", &req).await;
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());
    }

    #[tokio::test]
    async fn test_loader_not_found() {
        let mut registry = LoaderRegistry::new();
        registry.register(UserLoader);

        let req = LoaderRequest::get("/posts/123");
        let result = registry.load_for("/posts/123", &req).await;
        assert!(result.is_none());
    }

    #[test]
    fn test_route_matches() {
        assert!(route_matches("/users/:id", "/users/42"));
        assert!(route_matches("/users/:id", "/users/abc"));
        assert!(!route_matches("/users/:id", "/users"));
        assert!(!route_matches("/users/:id", "/posts/42"));
        assert!(route_matches("/", "/"));
    }

    #[test]
    fn test_extract_params() {
        let params = extract_params("/users/:id/posts/:post_id", "/users/42/posts/7");
        assert_eq!(params.get("id"), Some(&"42".to_string()));
        assert_eq!(params.get("post_id"), Some(&"7".to_string()));
    }

    #[test]
    fn test_loader_data_builder() {
        let data = LoaderData::json("{\"key\":\"value\"}")
            .with_status(201)
            .with_redirect("/new-path")
            .with_cache_control("max-age=3600")
            .with_header("X-Custom", "test");

        assert_eq!(data.meta.status, Some(201));
        assert_eq!(data.meta.redirect, Some("/new-path".to_string()));
        assert_eq!(data.meta.cache_control, Some("max-age=3600".to_string()));
        assert_eq!(data.meta.headers.get("X-Custom"), Some(&"test".to_string()));
    }

    #[test]
    fn test_loader_result_variants() {
        let ok = LoaderResult::ok(LoaderData::json("{}"));
        assert!(ok.is_ok());
        assert!(!ok.is_error());

        let err = LoaderResult::error("Not found", 404);
        assert!(err.is_error());
        assert!(!err.is_ok());

        let redirect = LoaderResult::redirect("/login", 302);
        assert!(redirect.is_redirect());
    }

    #[test]
    fn test_loader_request_builder() {
        let req = LoaderRequest::get("/api/data")
            .with_session("abc123")
            .with_query("page", "1");
        assert_eq!(req.session_id, Some("abc123".to_string()));
        assert_eq!(req.query.get("page"), Some(&"1".to_string()));
    }
}
