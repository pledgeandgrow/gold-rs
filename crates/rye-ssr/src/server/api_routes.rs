//! Goal 187: API routes with OpenAPI generation.
//!
//! `#[api_route]` macro that defines HTTP endpoints alongside rye components.
//! Auto-generates OpenAPI 3.1 spec from type signatures. Swagger UI at `/docs`.

use std::collections::HashMap;

/// HTTP method for API routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl ApiMethod {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiMethod::Get => "GET",
            ApiMethod::Post => "POST",
            ApiMethod::Put => "PUT",
            ApiMethod::Delete => "DELETE",
            ApiMethod::Patch => "PATCH",
        }
    }

    /// Convert to lowercase.
    pub fn to_lowercase(&self) -> &'static str {
        match self {
            ApiMethod::Get => "get",
            ApiMethod::Post => "post",
            ApiMethod::Put => "put",
            ApiMethod::Delete => "delete",
            ApiMethod::Patch => "patch",
        }
    }
}

/// A parameter location in an API route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamLocation {
    /// Path parameter (e.g. `/users/:id`).
    Path,
    /// Query parameter.
    Query,
    /// Header parameter.
    Header,
    /// Cookie parameter.
    Cookie,
}

impl ParamLocation {
    /// Convert to OpenAPI "in" string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamLocation::Path => "path",
            ParamLocation::Query => "query",
            ParamLocation::Header => "header",
            ParamLocation::Cookie => "cookie",
        }
    }
}

/// A parameter definition for an API route.
#[derive(Debug, Clone)]
pub struct ApiParam {
    /// Parameter name.
    pub name: String,
    /// Where the parameter is located.
    pub location: ParamLocation,
    /// Parameter description.
    pub description: String,
    /// Whether the parameter is required.
    pub required: bool,
    /// The type of the parameter.
    pub param_type: String,
}

/// An API route definition.
#[derive(Debug, Clone)]
pub struct ApiRoute {
    /// The route path (e.g. "/api/users/:id").
    pub path: String,
    /// The HTTP method.
    pub method: ApiMethod,
    /// Operation ID (unique identifier for the operation).
    pub operation_id: String,
    /// Summary of the operation.
    pub summary: String,
    /// Detailed description.
    pub description: String,
    /// Parameters for this route.
    pub params: Vec<ApiParam>,
    /// Request body type (if any).
    pub request_body: Option<ApiRequestBody>,
    /// Response definitions.
    pub responses: Vec<ApiResponse>,
    /// Tags for grouping operations.
    pub tags: Vec<String>,
}

/// A request body definition.
#[derive(Debug, Clone)]
pub struct ApiRequestBody {
    /// The content type (e.g. "application/json").
    pub content_type: String,
    /// The schema type name.
    pub schema_type: String,
    /// Whether the body is required.
    pub required: bool,
    /// Description of the request body.
    pub description: String,
}

/// A response definition.
#[derive(Debug, Clone)]
pub struct ApiResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response description.
    pub description: String,
    /// The content type (e.g. "application/json").
    pub content_type: String,
    /// The schema type name (if any).
    pub schema_type: Option<String>,
}

/// The OpenAPI document — generated from registered API routes.
#[derive(Debug, Clone)]
pub struct OpenApiDoc {
    /// OpenAPI version.
    pub openapi: String,
    /// API info.
    pub info: OpenApiInfo,
    /// API servers.
    pub servers: Vec<OpenApiServer>,
    /// Paths and operations.
    pub paths: HashMap<String, Vec<ApiRoute>>,
    /// Tags.
    pub tags: Vec<OpenApiTag>,
}

/// API info section.
#[derive(Debug, Clone)]
pub struct OpenApiInfo {
    /// API title.
    pub title: String,
    /// API version.
    pub version: String,
    /// API description.
    pub description: String,
}

/// Server definition.
#[derive(Debug, Clone)]
pub struct OpenApiServer {
    /// Server URL.
    pub url: String,
    /// Server description.
    pub description: String,
}

/// Tag definition.
#[derive(Debug, Clone)]
pub struct OpenApiTag {
    /// Tag name.
    pub name: String,
    /// Tag description.
    pub description: String,
}

/// The API route registry — stores routes and generates OpenAPI specs.
pub struct ApiRouteRegistry {
    routes: Vec<ApiRoute>,
    info: OpenApiInfo,
    servers: Vec<OpenApiServer>,
}

impl ApiRouteRegistry {
    /// Create a new registry with the given API info.
    pub fn new(title: &str, version: &str) -> Self {
        Self {
            routes: Vec::new(),
            info: OpenApiInfo {
                title: title.to_string(),
                version: version.to_string(),
                description: String::new(),
            },
            servers: Vec::new(),
        }
    }

    /// Set the API description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.info.description = desc.to_string();
        self
    }

    /// Add a server.
    pub fn add_server(&mut self, url: &str, description: &str) {
        self.servers.push(OpenApiServer {
            url: url.to_string(),
            description: description.to_string(),
        });
    }

    /// Register an API route.
    pub fn register(&mut self, route: ApiRoute) {
        self.routes.push(route);
    }

    /// Get all registered routes.
    pub fn routes(&self) -> &[ApiRoute] {
        &self.routes
    }

    /// Get the number of registered routes.
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// Generate the OpenAPI 3.1 document as JSON.
    pub fn to_json(&self) -> String {
        let mut paths_json = String::new();
        let mut grouped: HashMap<String, Vec<&ApiRoute>> = HashMap::new();

        for route in &self.routes {
            grouped.entry(route.path.clone()).or_default().push(route);
        }

        let mut first_path = true;
        for (path, routes) in &grouped {
            if !first_path {
                paths_json.push(',');
            }
            first_path = false;

            paths_json.push_str(&format!("\"{}\":{{", path));

            let mut first_method = true;
            for route in routes {
                if !first_method {
                    paths_json.push(',');
                }
                first_method = false;

                paths_json.push_str(&format!(
                    "\"{}\":{{\"operationId\":\"{}\",\"summary\":\"{}\",\"description\":\"{}\"",
                    route.method.to_lowercase(),
                    escape_json(&route.operation_id),
                    escape_json(&route.summary),
                    escape_json(&route.description),
                ));

                // Parameters
                if !route.params.is_empty() {
                    paths_json.push_str(",\"parameters\":[");
                    let mut first_param = true;
                    for param in &route.params {
                        if !first_param {
                            paths_json.push(',');
                        }
                        first_param = false;
                        paths_json.push_str(&format!(
                            "{{\"name\":\"{}\",\"in\":\"{}\",\"required\":{},\"description\":\"{}\",\"schema\":{{\"type\":\"{}\"}}}}",
                            escape_json(&param.name),
                            param.location.as_str(),
                            param.required,
                            escape_json(&param.description),
                            escape_json(&param.param_type),
                        ));
                    }
                    paths_json.push(']');
                }

                // Request body
                if let Some(body) = &route.request_body {
                    paths_json.push_str(&format!(
                        ",\"requestBody\":{{\"description\":\"{}\",\"required\":{},\"content\":{{\"{}\":{{\"schema\":{{\"type\":\"{}\"}}}}}}}}",
                        escape_json(&body.description),
                        body.required,
                        escape_json(&body.content_type),
                        escape_json(&body.schema_type),
                    ));
                }

                // Responses
                paths_json.push_str(",\"responses\":{");
                let mut first_resp = true;
                for resp in &route.responses {
                    if !first_resp {
                        paths_json.push(',');
                    }
                    first_resp = false;
                    paths_json.push_str(&format!(
                        "\"{}\":{{\"description\":\"{}\"",
                        resp.status,
                        escape_json(&resp.description),
                    ));
                    if let Some(schema) = &resp.schema_type {
                        paths_json.push_str(&format!(
                            ",\"content\":{{\"{}\":{{\"schema\":{{\"type\":\"{}\"}}}}}}",
                            escape_json(&resp.content_type),
                            escape_json(schema),
                        ));
                    }
                    paths_json.push('}');
                }
                paths_json.push('}'); // close responses

                // Tags
                if !route.tags.is_empty() {
                    let tags: Vec<String> = route.tags.iter().map(|t| format!("\"{}\"", escape_json(t))).collect();
                    paths_json.push_str(&format!(",\"tags\":[{}]", tags.join(",")));
                }

                paths_json.push('}'); // close method
            }

            paths_json.push('}'); // close path
        }

        let servers_json: Vec<String> = self
            .servers
            .iter()
            .map(|s| format!("{{\"url\":\"{}\",\"description\":\"{}\"}}", escape_json(&s.url), escape_json(&s.description)))
            .collect();

        format!(
            "{{\"openapi\":\"3.1.0\",\"info\":{{\"title\":\"{}\",\"version\":\"{}\",\"description\":\"{}\"}},\"servers\":[{}],\"paths\":{{{}}}}}",
            escape_json(&self.info.title),
            escape_json(&self.info.version),
            escape_json(&self.info.description),
            servers_json.join(","),
            paths_json,
        )
    }

    /// Generate the Swagger UI HTML page.
    pub fn swagger_ui_html(&self) -> String {
        let spec = self.to_json();
        format!(
            r#"<!DOCTYPE html><html><head><title>API Docs</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script><script>window.onload=function(){{SwaggerUIBundle({{url:'data:application/json;base64,{}',dom_id:'#swagger-ui'}});}};</script></body></html>"#,
            base64_encode(&spec),
        )
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 63) as usize] as char);
        result.push(CHARS[((triple >> 12) & 63) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 63) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 63) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Builder for creating API routes.
pub struct ApiRouteBuilder {
    route: ApiRoute,
}

impl ApiRouteBuilder {
    /// Create a new route builder.
    pub fn new(method: ApiMethod, path: &str, operation_id: &str) -> Self {
        Self {
            route: ApiRoute {
                path: path.to_string(),
                method,
                operation_id: operation_id.to_string(),
                summary: String::new(),
                description: String::new(),
                params: Vec::new(),
                request_body: None,
                responses: Vec::new(),
                tags: Vec::new(),
            },
        }
    }

    /// Set the summary.
    pub fn summary(mut self, summary: &str) -> Self {
        self.route.summary = summary.to_string();
        self
    }

    /// Set the description.
    pub fn description(mut self, desc: &str) -> Self {
        self.route.description = desc.to_string();
        self
    }

    /// Add a parameter.
    pub fn param(mut self, name: &str, location: ParamLocation, required: bool, param_type: &str) -> Self {
        self.route.params.push(ApiParam {
            name: name.to_string(),
            location,
            description: String::new(),
            required,
            param_type: param_type.to_string(),
        });
        self
    }

    /// Set the request body.
    pub fn request_body(mut self, content_type: &str, schema_type: &str, required: bool) -> Self {
        self.route.request_body = Some(ApiRequestBody {
            content_type: content_type.to_string(),
            schema_type: schema_type.to_string(),
            required,
            description: String::new(),
        });
        self
    }

    /// Add a response.
    pub fn response(mut self, status: u16, description: &str, content_type: &str, schema_type: Option<&str>) -> Self {
        self.route.responses.push(ApiResponse {
            status,
            description: description.to_string(),
            content_type: content_type.to_string(),
            schema_type: schema_type.map(|s| s.to_string()),
        });
        self
    }

    /// Add a tag.
    pub fn tag(mut self, tag: &str) -> Self {
        self.route.tags.push(tag.to_string());
        self
    }

    /// Build the route.
    pub fn build(self) -> ApiRoute {
        self.route
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_method_as_str() {
        assert_eq!(ApiMethod::Get.as_str(), "GET");
        assert_eq!(ApiMethod::Post.as_str(), "POST");
        assert_eq!(ApiMethod::Delete.as_str(), "DELETE");
    }

    #[test]
    fn test_api_method_to_lowercase() {
        assert_eq!(ApiMethod::Get.to_lowercase(), "get");
        assert_eq!(ApiMethod::Post.to_lowercase(), "post");
    }

    #[test]
    fn test_param_location_as_str() {
        assert_eq!(ParamLocation::Path.as_str(), "path");
        assert_eq!(ParamLocation::Query.as_str(), "query");
        assert_eq!(ParamLocation::Header.as_str(), "header");
        assert_eq!(ParamLocation::Cookie.as_str(), "cookie");
    }

    #[test]
    fn test_api_route_builder() {
        let route = ApiRouteBuilder::new(ApiMethod::Get, "/api/users/:id", "getUserById")
            .summary("Get a user by ID")
            .description("Returns a single user by their unique ID")
            .param("id", ParamLocation::Path, true, "string")
            .response(200, "User found", "application/json", Some("User"))
            .response(404, "User not found", "application/json", Some("Error"))
            .tag("users")
            .build();

        assert_eq!(route.path, "/api/users/:id");
        assert_eq!(route.method, ApiMethod::Get);
        assert_eq!(route.operation_id, "getUserById");
        assert_eq!(route.params.len(), 1);
        assert_eq!(route.responses.len(), 2);
        assert_eq!(route.tags, vec!["users".to_string()]);
    }

    #[test]
    fn test_api_route_builder_with_body() {
        let route = ApiRouteBuilder::new(ApiMethod::Post, "/api/users", "createUser")
            .summary("Create a user")
            .request_body("application/json", "CreateUserInput", true)
            .response(201, "User created", "application/json", Some("User"))
            .build();

        assert!(route.request_body.is_some());
        let body = route.request_body.unwrap();
        assert_eq!(body.schema_type, "CreateUserInput");
        assert!(body.required);
    }

    #[test]
    fn test_registry_to_json() {
        let mut registry = ApiRouteRegistry::new("Test API", "1.0.0");
        registry.register(
            ApiRouteBuilder::new(ApiMethod::Get, "/api/users", "listUsers")
                .summary("List all users")
                .response(200, "Success", "application/json", Some("UserList"))
                .build(),
        );

        let json = registry.to_json();
        assert!(json.contains("\"openapi\":\"3.1.0\""));
        assert!(json.contains("\"title\":\"Test API\""));
        assert!(json.contains("\"/api/users\""));
        assert!(json.contains("\"get\""));
        assert!(json.contains("\"operationId\":\"listUsers\""));
    }

    #[test]
    fn test_registry_to_json_with_params() {
        let mut registry = ApiRouteRegistry::new("Test API", "1.0.0");
        registry.register(
            ApiRouteBuilder::new(ApiMethod::Get, "/api/users/:id", "getUser")
                .param("id", ParamLocation::Path, true, "string")
                .response(200, "OK", "application/json", Some("User"))
                .build(),
        );

        let json = registry.to_json();
        assert!(json.contains("\"parameters\""));
        assert!(json.contains("\"name\":\"id\""));
        assert!(json.contains("\"in\":\"path\""));
    }

    #[test]
    fn test_registry_swagger_ui() {
        let registry = ApiRouteRegistry::new("Test API", "1.0.0");
        let html = registry.swagger_ui_html();
        assert!(html.contains("swagger-ui"));
        assert!(html.contains("SwaggerUIBundle"));
    }

    #[test]
    fn test_registry_servers() {
        let mut registry = ApiRouteRegistry::new("Test API", "1.0.0");
        registry.add_server("https://api.example.com", "Production");
        let json = registry.to_json();
        assert!(json.contains("api.example.com"));
    }

    #[test]
    fn test_registry_len() {
        let mut registry = ApiRouteRegistry::new("Test", "1.0");
        assert_eq!(registry.len(), 0);
        registry.register(
            ApiRouteBuilder::new(ApiMethod::Get, "/test", "test")
                .response(200, "OK", "application/json", None)
                .build(),
        );
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_json("a\\b"), "a\\\\b");
    }

    #[test]
    fn test_base64_encode() {
        let encoded = base64_encode("Hello");
        assert!(!encoded.is_empty());
    }
}
