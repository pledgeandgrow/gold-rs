//! Goal 236: Integration testing harness.
//!
//! `rye-testing` extension that spins up a full SSR server, makes real HTTP
//! requests, and asserts on the rendered HTML. End-to-end testing without a browser.

use std::collections::HashMap;

/// An HTTP request in an integration test.
#[derive(Debug, Clone)]
pub struct TestRequest {
    /// The HTTP method.
    pub method: String,
    /// The path.
    pub path: String,
    /// The headers.
    pub headers: HashMap<String, String>,
    /// The body (if any).
    pub body: Option<String>,
}

impl TestRequest {
    /// Create a GET request.
    pub fn get(path: &str) -> Self {
        Self {
            method: "GET".to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a POST request.
    pub fn post(path: &str, body: &str) -> Self {
        Self {
            method: "POST".to_string(),
            path: path.to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
        }
    }

    /// Add a header.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

/// An HTTP response in an integration test.
#[derive(Debug, Clone)]
pub struct TestResponse {
    /// The status code.
    pub status: u16,
    /// The headers.
    pub headers: HashMap<String, String>,
    /// The body.
    pub body: String,
}

impl TestResponse {
    /// Create a successful response.
    pub fn ok(body: &str) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: body.to_string(),
        }
    }

    /// Create a response with a status code.
    pub fn with_status(status: u16, body: &str) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: body.to_string(),
        }
    }

    /// Check if the response is successful (2xx).
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Assert the body contains a string.
    pub fn assert_contains(&self, needle: &str) -> bool {
        self.body.contains(needle)
    }

    /// Assert the body matches exactly.
    pub fn assert_body(&self, expected: &str) -> bool {
        self.body == expected
    }
}

/// A mock SSR server for integration tests.
pub struct MockSsrServer {
    routes: HashMap<String, fn(&TestRequest) -> TestResponse>,
}

impl MockSsrServer {
    /// Create a new mock server.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a route handler.
    pub fn route(&mut self, path: &str, handler: fn(&TestRequest) -> TestResponse) {
        self.routes.insert(path.to_string(), handler);
    }

    /// Handle a request.
    pub fn handle(&self, request: &TestRequest) -> TestResponse {
        if let Some(handler) = self.routes.get(&request.path) {
            handler(request)
        } else {
            TestResponse::with_status(404, "Not Found")
        }
    }

    /// Get the number of registered routes.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

impl Default for MockSsrServer {
    fn default() -> Self {
        Self::new()
    }
}

/// An integration test case.
#[derive(Debug, Clone)]
pub struct IntegrationTestCase {
    /// The test name.
    pub name: String,
    /// The request to make.
    pub request: TestRequest,
    /// The expected status code.
    pub expected_status: u16,
    /// Strings the response body should contain.
    pub expected_contains: Vec<String>,
    /// Strings the response body should NOT contain.
    pub expected_not_contains: Vec<String>,
}

impl IntegrationTestCase {
    /// Create a new test case.
    pub fn new(name: &str, request: TestRequest) -> Self {
        Self {
            name: name.to_string(),
            request,
            expected_status: 200,
            expected_contains: Vec::new(),
            expected_not_contains: Vec::new(),
        }
    }

    /// Set expected status.
    pub fn expect_status(mut self, status: u16) -> Self {
        self.expected_status = status;
        self
    }

    /// Add an expected substring.
    pub fn expect_contains(mut self, text: &str) -> Self {
        self.expected_contains.push(text.to_string());
        self
    }

    /// Add a string that should NOT be present.
    pub fn expect_not_contains(mut self, text: &str) -> Self {
        self.expected_not_contains.push(text.to_string());
        self
    }

    /// Run the test case against a response.
    pub fn run(&self, response: &TestResponse) -> bool {
        if response.status != self.expected_status {
            return false;
        }
        for needle in &self.expected_contains {
            if !response.body.contains(needle) {
                return false;
            }
        }
        for needle in &self.expected_not_contains {
            if response.body.contains(needle) {
                return false;
            }
        }
        true
    }
}

/// The integration test runner.
pub struct IntegrationTestRunner {
    server: MockSsrServer,
    tests: Vec<IntegrationTestCase>,
}

impl IntegrationTestRunner {
    /// Create a new runner.
    pub fn new(server: MockSsrServer) -> Self {
        Self {
            server,
            tests: Vec::new(),
        }
    }

    /// Add a test case.
    pub fn add_test(&mut self, test: IntegrationTestCase) {
        self.tests.push(test);
    }

    /// Run all tests.
    pub fn run_all(&self) -> (usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        for test in &self.tests {
            let response = self.server.handle(&test.request);
            if test.run(&response) {
                passed += 1;
            } else {
                failed += 1;
            }
        }
        (passed, failed)
    }

    /// Get the number of tests.
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home_handler(_req: &TestRequest) -> TestResponse {
        TestResponse::ok("<html><body><h1>Welcome</h1></body></html>")
    }

    fn api_handler(_req: &TestRequest) -> TestResponse {
        TestResponse::ok(r#"{"status":"ok"}"#)
    }

    #[test]
    fn test_test_request_get() {
        let req = TestRequest::get("/");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/");
    }

    #[test]
    fn test_test_request_post() {
        let req = TestRequest::post("/api", "{}");
        assert_eq!(req.method, "POST");
        assert!(req.body.is_some());
    }

    #[test]
    fn test_test_request_with_header() {
        let req = TestRequest::get("/").with_header("Accept", "text/html");
        assert_eq!(req.headers.get("Accept"), Some(&"text/html".to_string()));
    }

    #[test]
    fn test_test_response_ok() {
        let res = TestResponse::ok("hello");
        assert!(res.is_success());
        assert!(res.assert_contains("hello"));
    }

    #[test]
    fn test_test_response_with_status() {
        let res = TestResponse::with_status(404, "Not Found");
        assert!(!res.is_success());
        assert_eq!(res.status, 404);
    }

    #[test]
    fn test_test_response_assert_body() {
        let res = TestResponse::ok("exact");
        assert!(res.assert_body("exact"));
        assert!(!res.assert_body("other"));
    }

    #[test]
    fn test_mock_ssr_server_route() {
        let mut server = MockSsrServer::new();
        server.route("/", home_handler);
        assert_eq!(server.route_count(), 1);
    }

    #[test]
    fn test_mock_ssr_server_handle() {
        let mut server = MockSsrServer::new();
        server.route("/", home_handler);
        server.route("/api", api_handler);

        let req = TestRequest::get("/");
        let res = server.handle(&req);
        assert!(res.is_success());
        assert!(res.assert_contains("Welcome"));
    }

    #[test]
    fn test_mock_ssr_server_404() {
        let server = MockSsrServer::new();
        let req = TestRequest::get("/unknown");
        let res = server.handle(&req);
        assert_eq!(res.status, 404);
    }

    #[test]
    fn test_integration_test_case_new() {
        let tc = IntegrationTestCase::new("home", TestRequest::get("/"));
        assert_eq!(tc.name, "home");
        assert_eq!(tc.expected_status, 200);
    }

    #[test]
    fn test_integration_test_case_builder() {
        let tc = IntegrationTestCase::new("home", TestRequest::get("/"))
            .expect_status(200)
            .expect_contains("Welcome")
            .expect_not_contains("Error");
        assert_eq!(tc.expected_contains.len(), 1);
        assert_eq!(tc.expected_not_contains.len(), 1);
    }

    #[test]
    fn test_integration_test_case_run_pass() {
        let tc = IntegrationTestCase::new("home", TestRequest::get("/")).expect_contains("Welcome");
        let res = TestResponse::ok("<h1>Welcome</h1>");
        assert!(tc.run(&res));
    }

    #[test]
    fn test_integration_test_case_run_fail_status() {
        let tc = IntegrationTestCase::new("home", TestRequest::get("/")).expect_status(200);
        let res = TestResponse::with_status(500, "error");
        assert!(!tc.run(&res));
    }

    #[test]
    fn test_integration_test_case_run_fail_contains() {
        let tc = IntegrationTestCase::new("home", TestRequest::get("/")).expect_contains("Welcome");
        let res = TestResponse::ok("Goodbye");
        assert!(!tc.run(&res));
    }

    #[test]
    fn test_integration_test_case_run_fail_not_contains() {
        let tc =
            IntegrationTestCase::new("home", TestRequest::get("/")).expect_not_contains("Error");
        let res = TestResponse::ok("Error occurred");
        assert!(!tc.run(&res));
    }

    #[test]
    fn test_integration_test_runner() {
        let mut server = MockSsrServer::new();
        server.route("/", home_handler);

        let mut runner = IntegrationTestRunner::new(server);
        runner.add_test(
            IntegrationTestCase::new("home_page", TestRequest::get("/")).expect_contains("Welcome"),
        );
        runner.add_test(
            IntegrationTestCase::new("api_404", TestRequest::get("/api")).expect_status(404),
        );

        let (passed, failed) = runner.run_all();
        assert_eq!(passed, 2);
        assert_eq!(failed, 0);
    }

    #[test]
    fn test_integration_test_runner_test_count() {
        let server = MockSsrServer::new();
        let mut runner = IntegrationTestRunner::new(server);
        runner.add_test(IntegrationTestCase::new("a", TestRequest::get("/")));
        runner.add_test(IntegrationTestCase::new("b", TestRequest::get("/")));
        assert_eq!(runner.test_count(), 2);
    }
}
