//! Goal 129: Server middleware pipeline.
//!
//! Composable middleware chain for server-side request processing.
//! Middleware can modify requests, responses, or short-circuit.

use std::collections::HashMap;

/// HTTP method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// A server request.
#[derive(Debug, Clone)]
pub struct ServerRequest {
    /// HTTP method.
    pub method: HttpMethod,
    /// Request path.
    pub path: String,
    /// Query parameters.
    pub query: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body.
    pub body: Vec<u8>,
}

impl ServerRequest {
    /// Create a new GET request.
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::Get,
            path: path.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Create a new POST request.
    pub fn post(path: impl Into<String>, body: Vec<u8>) -> Self {
        Self {
            method: HttpMethod::Post,
            path: path.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body,
        }
    }

    /// Add a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// A server response.
#[derive(Debug, Clone)]
pub struct ServerResponse {
    /// Status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: Vec<u8>,
}

impl ServerResponse {
    /// Create a 200 OK response.
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Create a response with a status code.
    pub fn status(status: u16, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.into(),
        }
    }

    /// Add a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Whether the response is a redirect.
    pub fn is_redirect(&self) -> bool {
        self.status >= 300 && self.status < 400
    }

    /// Whether the response is an error.
    pub fn is_error(&self) -> bool {
        self.status >= 400
    }
}

/// Middleware function type.
pub type MiddlewareFn = Box<dyn Fn(&mut ServerRequest, &mut ServerResponse) -> MiddlewareResult + Send + Sync>;

/// Middleware execution result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MiddlewareResult {
    /// Continue to next middleware.
    Continue,
    /// Short-circuit — stop processing and return the response.
    Stop,
}

/// A middleware in the pipeline.
pub struct Middleware {
    /// Name for debugging.
    pub name: String,
    /// The middleware function.
    pub handler: MiddlewareFn,
}

/// Server middleware pipeline.
pub struct MiddlewarePipeline {
    /// Ordered middleware.
    middleware: Vec<Middleware>,
}

impl MiddlewarePipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { middleware: Vec::new() }
    }

    /// Add middleware to the end of the pipeline.
    pub fn add(&mut self, name: impl Into<String>, handler: MiddlewareFn) {
        self.middleware.push(Middleware {
            name: name.into(),
            handler,
        });
    }

    /// Add middleware to the beginning of the pipeline.
    pub fn prepend(&mut self, name: impl Into<String>, handler: MiddlewareFn) {
        self.middleware.insert(0, Middleware {
            name: name.into(),
            handler,
        });
    }

    /// Execute the pipeline on a request/response.
    pub fn execute(&self, request: &mut ServerRequest, response: &mut ServerResponse) {
        for mw in &self.middleware {
            let result = (mw.handler)(request, response);
            if result == MiddlewareResult::Stop {
                break;
            }
        }
    }

    /// Number of middleware in the pipeline.
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    /// Whether the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }
}

impl Default for MiddlewarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Common middleware: CORS headers.
pub fn cors_middleware() -> MiddlewareFn {
    Box::new(|_req: &mut ServerRequest, res: &mut ServerResponse| {
        res.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
        res.headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string());
        res.headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization".to_string());
        MiddlewareResult::Continue
    })
}

/// Common middleware: request logging.
pub fn logging_middleware() -> MiddlewareFn {
    Box::new(|req: &mut ServerRequest, _res: &mut ServerResponse| {
        // In a real implementation, this would log to a logger
        // Here we just continue
        let _ = &req.method;
        let _ = &req.path;
        MiddlewareResult::Continue
    })
}

/// Common middleware: compression check.
pub fn compression_middleware() -> MiddlewareFn {
    Box::new(|_req: &mut ServerRequest, res: &mut ServerResponse| {
        // Check if client accepts gzip
        res.headers.insert("Vary".to_string(), "Accept-Encoding".to_string());
        MiddlewareResult::Continue
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_request() {
        let req = ServerRequest::get("/users")
            .with_header("Accept", "application/json");
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/users");
        assert_eq!(req.headers.get("Accept"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_server_response() {
        let res = ServerResponse::ok(b"hello".to_vec())
            .with_header("Content-Type", "text/plain");
        assert_eq!(res.status, 200);
        assert_eq!(res.body, b"hello");
        assert!(!res.is_error());
    }

    #[test]
    fn test_server_response_redirect() {
        let res = ServerResponse::status(302, Vec::new());
        assert!(res.is_redirect());
        assert!(!res.is_error());
    }

    #[test]
    fn test_server_response_error() {
        let res = ServerResponse::status(404, b"Not Found".to_vec());
        assert!(res.is_error());
    }

    #[test]
    fn test_middleware_pipeline() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add("cors", cors_middleware());
        pipeline.add("logging", logging_middleware());

        assert_eq!(pipeline.len(), 2);

        let mut req = ServerRequest::get("/api/data");
        let mut res = ServerResponse::ok(Vec::new());
        pipeline.execute(&mut req, &mut res);

        assert_eq!(res.headers.get("Access-Control-Allow-Origin"), Some(&"*".to_string()));
    }

    #[test]
    fn test_middleware_short_circuit() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add("auth", Box::new(|_req, res| {
            res.status = 401;
            MiddlewareResult::Stop
        }));
        pipeline.add("handler", Box::new(|_req, res| {
            res.status = 200;
            MiddlewareResult::Continue
        }));

        let mut req = ServerRequest::get("/protected");
        let mut res = ServerResponse::ok(Vec::new());
        pipeline.execute(&mut req, &mut res);

        assert_eq!(res.status, 401);
    }

    #[test]
    fn test_middleware_prepend() {
        let mut pipeline = MiddlewarePipeline::new();
        pipeline.add("second", logging_middleware());
        pipeline.prepend("first", cors_middleware());

        assert_eq!(pipeline.len(), 2);
    }

    #[test]
    fn test_http_method() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
    }
}
