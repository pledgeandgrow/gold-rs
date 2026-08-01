//! Goal 192: Request-scoped context.
//!
//! `RequestContext` that carries per-request data (user, locale, theme, request ID)
//! through the component tree during SSR. Automatically injected into components
//! via context system. No manual prop drilling for request data.

use std::collections::HashMap;

/// Request-scoped context — carries per-request data through the component tree.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID (for logging/tracing).
    pub request_id: String,
    /// The authenticated user ID, if any.
    pub user_id: Option<String>,
    /// The user's locale (e.g. "en-US", "fr-FR").
    pub locale: String,
    /// The user's preferred theme.
    pub theme: Theme,
    /// The request's IP address.
    pub ip: Option<String>,
    /// The user agent string.
    pub user_agent: Option<String>,
    /// Custom request-scoped data.
    pub data: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// The request URL.
    pub url: String,
    /// The HTTP method.
    pub method: String,
}

/// The user's preferred theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Light theme.
    Light,
    /// Dark theme.
    Dark,
    /// System preference.
    System,
}

impl Theme {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    /// Parse from string.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        }
    }
}

impl RequestContext {
    /// Create a new request context.
    pub fn new(request_id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            user_id: None,
            locale: "en-US".to_string(),
            theme: Theme::System,
            ip: None,
            user_agent: None,
            data: HashMap::new(),
            headers: HashMap::new(),
            url: url.into(),
            method: "GET".to_string(),
        }
    }

    /// Set the user ID.
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the locale.
    pub fn with_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Set the theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the IP address.
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = Some(ip.into());
        self
    }

    /// Set the user agent.
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set the HTTP method.
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    /// Add a custom data field.
    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Add a header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Get a custom data field.
    pub fn get_data(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Get a header value.
    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    /// Check if the user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Extract request context from HTTP headers.
    pub fn from_headers(request_id: &str, url: &str, headers: &HashMap<String, String>) -> Self {
        let mut ctx = Self::new(request_id, url);

        // Extract locale from Accept-Language
        if let Some(accept_lang) = headers.get("accept-language") {
            let locale = accept_lang
                .split(',')
                .next()
                .unwrap_or("en-US")
                .split(';')
                .next()
                .unwrap_or("en-US")
                .trim();
            ctx = ctx.with_locale(locale);
        }

        // Extract theme from cookie or header
        if let Some(theme) = headers.get("x-theme") {
            ctx = ctx.with_theme(Theme::from_str(theme));
        }

        // Extract user agent
        if let Some(ua) = headers.get("user-agent") {
            ctx = ctx.with_user_agent(ua);
        }

        // Extract IP
        if let Some(ip) = headers.get("x-forwarded-for") {
            let ip = ip.split(',').next().unwrap_or("").trim();
            if !ip.is_empty() {
                ctx = ctx.with_ip(ip);
            }
        } else if let Some(ip) = headers.get("x-real-ip") {
            ctx = ctx.with_ip(ip);
        }

        ctx
    }

    /// Serialize to a script tag for client-side hydration.
    pub fn to_script_tag(&self) -> String {
        let user = self.user_id.as_deref().unwrap_or("");
        let ip = self.ip.as_deref().unwrap_or("");
        let ua = self.user_agent.as_deref().unwrap_or("");

        format!(
            r#"<script>window.__RYE_REQUEST__={{"requestId":"{}","userId":"{}","locale":"{}","theme":"{}","ip":"{}","userAgent":"{}","url":"{}","method":"{}"}};</script>"#,
            escape_js(&self.request_id),
            escape_js(user),
            escape_js(&self.locale),
            self.theme.as_str(),
            escape_js(ip),
            escape_js(ua),
            escape_js(&self.url),
            escape_js(&self.method),
        )
    }
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new("req-1", "/home");
        assert_eq!(ctx.request_id, "req-1");
        assert_eq!(ctx.url, "/home");
        assert_eq!(ctx.locale, "en-US");
        assert_eq!(ctx.theme, Theme::System);
        assert!(!ctx.is_authenticated());
    }

    #[test]
    fn test_request_context_builder() {
        let ctx = RequestContext::new("req-1", "/dashboard")
            .with_user("user-123")
            .with_locale("fr-FR")
            .with_theme(Theme::Dark)
            .with_ip("192.168.1.1")
            .with_user_agent("Mozilla/5.0")
            .with_method("POST")
            .with_data("feature_flag", "enabled")
            .with_header("x-custom", "value");

        assert_eq!(ctx.user_id, Some("user-123".to_string()));
        assert_eq!(ctx.locale, "fr-FR");
        assert_eq!(ctx.theme, Theme::Dark);
        assert_eq!(ctx.ip, Some("192.168.1.1".to_string()));
        assert_eq!(ctx.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(ctx.method, "POST");
        assert!(ctx.is_authenticated());
        assert_eq!(ctx.get_data("feature_flag"), Some("enabled"));
        assert_eq!(ctx.get_header("x-custom"), Some("value"));
    }

    #[test]
    fn test_theme_from_str() {
        assert_eq!(Theme::from_str("dark"), Theme::Dark);
        assert_eq!(Theme::from_str("DARK"), Theme::Dark);
        assert_eq!(Theme::from_str("light"), Theme::Light);
        assert_eq!(Theme::from_str("system"), Theme::System);
        assert_eq!(Theme::from_str("unknown"), Theme::Light);
    }

    #[test]
    fn test_theme_as_str() {
        assert_eq!(Theme::Light.as_str(), "light");
        assert_eq!(Theme::Dark.as_str(), "dark");
        assert_eq!(Theme::System.as_str(), "system");
    }

    #[test]
    fn test_from_headers() {
        let mut headers = HashMap::new();
        headers.insert("accept-language".to_string(), "fr-FR,fr;q=0.9,en;q=0.8".to_string());
        headers.insert("x-theme".to_string(), "dark".to_string());
        headers.insert("user-agent".to_string(), "Mozilla/5.0".to_string());
        headers.insert("x-forwarded-for".to_string(), "10.0.0.1, 10.0.0.2".to_string());

        let ctx = RequestContext::from_headers("req-1", "/page", &headers);
        assert_eq!(ctx.locale, "fr-FR");
        assert_eq!(ctx.theme, Theme::Dark);
        assert_eq!(ctx.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(ctx.ip, Some("10.0.0.1".to_string()));
    }

    #[test]
    fn test_from_headers_real_ip() {
        let mut headers = HashMap::new();
        headers.insert("x-real-ip".to_string(), "172.16.0.1".to_string());

        let ctx = RequestContext::from_headers("req-1", "/", &headers);
        assert_eq!(ctx.ip, Some("172.16.0.1".to_string()));
    }

    #[test]
    fn test_from_headers_defaults() {
        let headers = HashMap::new();
        let ctx = RequestContext::from_headers("req-1", "/", &headers);
        assert_eq!(ctx.locale, "en-US");
        assert_eq!(ctx.theme, Theme::System);
        assert!(!ctx.is_authenticated());
    }

    #[test]
    fn test_to_script_tag() {
        let ctx = RequestContext::new("req-1", "/home")
            .with_user("user-1")
            .with_locale("en-US")
            .with_theme(Theme::Dark);

        let tag = ctx.to_script_tag();
        assert!(tag.contains("__RYE_REQUEST__"));
        assert!(tag.contains("req-1"));
        assert!(tag.contains("user-1"));
        assert!(tag.contains("dark"));
    }

    #[test]
    fn test_is_authenticated() {
        let ctx_with = RequestContext::new("r", "/").with_user("u1");
        assert!(ctx_with.is_authenticated());

        let ctx_without = RequestContext::new("r", "/");
        assert!(!ctx_without.is_authenticated());
    }

    #[test]
    fn test_get_data_nonexistent() {
        let ctx = RequestContext::new("r", "/");
        assert!(ctx.get_data("nonexistent").is_none());
    }

    #[test]
    fn test_get_header_nonexistent() {
        let ctx = RequestContext::new("r", "/");
        assert!(ctx.get_header("nonexistent").is_none());
    }
}
