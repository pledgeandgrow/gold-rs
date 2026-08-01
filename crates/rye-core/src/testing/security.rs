//! Goal 143: Security audit helpers.
//!
//! Automated security checks for rye apps: XSS prevention, CSP validation,
//! dependency audit, and secure coding pattern verification.

use std::collections::HashMap;

/// A security finding.
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    /// Finding severity.
    pub severity: SecuritySeverity,
    /// Rule that triggered the finding.
    pub rule: SecurityRule,
    /// Description.
    pub description: String,
    /// Location (file:line or element).
    pub location: String,
    /// Recommended fix.
    pub recommendation: String,
}

/// Security finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    /// Info level.
    Info,
    /// Low severity.
    Low,
    /// Medium severity.
    Medium,
    /// High severity.
    High,
    /// Critical severity.
    Critical,
}

impl SecuritySeverity {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SecuritySeverity::Info => "info",
            SecuritySeverity::Low => "low",
            SecuritySeverity::Medium => "medium",
            SecuritySeverity::High => "high",
            SecuritySeverity::Critical => "critical",
        }
    }
}

/// Security rule identifiers.
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityRule {
    /// XSS: unescaped user input in HTML.
    XssUnescaped,
    /// XSS: dangerous innerHTML usage.
    XssInnerHTML,
    /// XSS: eval() usage.
    XssEval,
    /// Missing Content-Security-Policy.
    MissingCsp,
    /// Weak CSP (unsafe-inline).
    WeakCsp,
    /// Insecure HTTP (not HTTPS).
    InsecureHttp,
    /// Missing integrity attribute on script.
    MissingIntegrity,
    /// Sensitive data in localStorage.
    SensitiveStorage,
    /// Missing CSRF protection.
    MissingCsrf,
    /// Open redirect vulnerability.
    OpenRedirect,
    /// Custom rule.
    Custom(String),
}

impl SecurityRule {
    /// Get the rule ID string.
    pub fn id(&self) -> String {
        match self {
            SecurityRule::XssUnescaped => "xss-unescaped".to_string(),
            SecurityRule::XssInnerHTML => "xss-innerhtml".to_string(),
            SecurityRule::XssEval => "xss-eval".to_string(),
            SecurityRule::MissingCsp => "missing-csp".to_string(),
            SecurityRule::WeakCsp => "weak-csp".to_string(),
            SecurityRule::InsecureHttp => "insecure-http".to_string(),
            SecurityRule::MissingIntegrity => "missing-integrity".to_string(),
            SecurityRule::SensitiveStorage => "sensitive-storage".to_string(),
            SecurityRule::MissingCsrf => "missing-csrf".to_string(),
            SecurityRule::OpenRedirect => "open-redirect".to_string(),
            SecurityRule::Custom(s) => s.clone(),
        }
    }
}

/// Security audit report.
#[derive(Debug, Clone)]
pub struct SecurityReport {
    /// Findings.
    pub findings: Vec<SecurityFinding>,
    /// Number of files checked.
    pub files_checked: usize,
}

impl SecurityReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            files_checked: 0,
        }
    }

    /// Add a finding.
    pub fn add(&mut self, finding: SecurityFinding) {
        self.findings.push(finding);
    }

    /// Whether the report has critical or high findings.
    pub fn has_critical_issues(&self) -> bool {
        self.findings.iter().any(|f| f.severity >= SecuritySeverity::High)
    }

    /// Count findings by severity.
    pub fn count_by_severity(&self, severity: SecuritySeverity) -> usize {
        self.findings.iter().filter(|f| f.severity == severity).count()
    }

    /// Generate a summary.
    pub fn summary(&self) -> String {
        format!(
            "Security Report: {} critical, {} high, {} medium, {} low, {} info ({} files checked)",
            self.count_by_severity(SecuritySeverity::Critical),
            self.count_by_severity(SecuritySeverity::High),
            self.count_by_severity(SecuritySeverity::Medium),
            self.count_by_severity(SecuritySeverity::Low),
            self.count_by_severity(SecuritySeverity::Info),
            self.files_checked,
        )
    }
}

impl Default for SecurityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Check HTML content for XSS vulnerabilities.
pub fn check_xss(html: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Check for innerHTML usage
    if html.contains("innerHTML") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::High,
            rule: SecurityRule::XssInnerHTML,
            description: "Use of innerHTML detected — potential XSS vector".to_string(),
            location: "innerHTML".to_string(),
            recommendation: "Use textContent or a safe DOM API instead".to_string(),
        });
    }

    // Check for eval() usage
    if html.contains("eval(") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Critical,
            rule: SecurityRule::XssEval,
            description: "Use of eval() detected — arbitrary code execution".to_string(),
            location: "eval()".to_string(),
            recommendation: "Avoid eval() — use JSON.parse() or Function() with caution".to_string(),
        });
    }

    // Check for document.write
    if html.contains("document.write") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::High,
            rule: SecurityRule::XssUnescaped,
            description: "Use of document.write detected — potential XSS".to_string(),
            location: "document.write".to_string(),
            recommendation: "Use safe DOM manipulation methods".to_string(),
        });
    }

    // Check for unescaped template expressions ({{ }}) in script context
    if html.contains("<script>") && html.contains("{{") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Medium,
            rule: SecurityRule::XssUnescaped,
            description: "Template expression inside <script> tag — potential XSS".to_string(),
            location: "<script>".to_string(),
            recommendation: "Escape dynamic content in script contexts".to_string(),
        });
    }

    findings
}

/// Validate Content-Security-Policy header value.
pub fn validate_csp(csp: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if csp.contains("'unsafe-inline'") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Medium,
            rule: SecurityRule::WeakCsp,
            description: "CSP contains 'unsafe-inline' — weakens XSS protection".to_string(),
            location: "Content-Security-Policy".to_string(),
            recommendation: "Use nonces or hashes instead of 'unsafe-inline'".to_string(),
        });
    }

    if csp.contains("'unsafe-eval'") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::High,
            rule: SecurityRule::WeakCsp,
            description: "CSP contains 'unsafe-eval' — allows eval()".to_string(),
            location: "Content-Security-Policy".to_string(),
            recommendation: "Remove 'unsafe-eval' and refactor code to avoid eval()".to_string(),
        });
    }

    if csp.contains("*") && !csp.contains("'self'") {
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Medium,
            rule: SecurityRule::WeakCsp,
            description: "CSP uses wildcard (*) — overly permissive".to_string(),
            location: "Content-Security-Policy".to_string(),
            recommendation: "Specify exact domains instead of wildcards".to_string(),
        });
    }

    findings
}

/// Check for insecure URLs.
pub fn check_insecure_urls(html: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Check for http:// URLs in src/href attributes
    let http_pattern = "http://";
    let mut start = 0;
    while let Some(pos) = html[start..].find(http_pattern) {
        let abs_pos = start + pos;
        findings.push(SecurityFinding {
            severity: SecuritySeverity::Low,
            rule: SecurityRule::InsecureHttp,
            description: "Insecure HTTP URL detected".to_string(),
            location: format!("offset:{}", abs_pos),
            recommendation: "Use HTTPS instead of HTTP".to_string(),
        });
        start = abs_pos + http_pattern.len();
    }

    findings
}

/// Check script tags for integrity attributes.
pub fn check_script_integrity(html: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Simple check: find <script src=...> without integrity=
    let mut start = 0;
    while let Some(pos) = html[start..].find("<script") {
        let abs_pos = start + pos;
        // Find the end of the script tag
        if let Some(end) = html[abs_pos..].find('>') {
            let tag = &html[abs_pos..abs_pos + end + 1];
            if tag.contains("src=") && !tag.contains("integrity=") {
                findings.push(SecurityFinding {
                    severity: SecuritySeverity::Low,
                    rule: SecurityRule::MissingIntegrity,
                    description: "External script missing integrity attribute".to_string(),
                    location: tag.to_string(),
                    recommendation: "Add integrity attribute with SRI hash".to_string(),
                });
            }
        }
        start = abs_pos + 7;
    }

    findings
}

/// Run a full security audit on HTML content.
pub fn audit_html(html: &str) -> SecurityReport {
    let mut report = SecurityReport::new();
    report.files_checked = 1;

    for finding in check_xss(html) {
        report.add(finding);
    }
    for finding in check_insecure_urls(html) {
        report.add(finding);
    }
    for finding in check_script_integrity(html) {
        report.add(finding);
    }

    report
}

/// HTML-escape user content to prevent XSS.
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Check if a redirect URL is safe (no open redirect).
pub fn is_safe_redirect(url: &str) -> bool {
    // Allow only relative URLs or same-origin
    if url.starts_with('/') && !url.starts_with("//") {
        return true;
    }
    if url.starts_with("https://") || url.starts_with("http://") {
        // Would need to check same-origin in a real implementation
        return false; // Conservative: block absolute URLs
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_xss_innerhtml() {
        let findings = check_xss("element.innerHTML = userInput;");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, SecurityRule::XssInnerHTML);
        assert_eq!(findings[0].severity, SecuritySeverity::High);
    }

    #[test]
    fn test_check_xss_eval() {
        let findings = check_xss("eval(userInput)");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, SecurityRule::XssEval);
        assert_eq!(findings[0].severity, SecuritySeverity::Critical);
    }

    #[test]
    fn test_check_xss_clean() {
        let findings = check_xss("<div>Hello World</div>");
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_validate_csp_unsafe_inline() {
        let findings = validate_csp("script-src 'unsafe-inline'");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, SecurityRule::WeakCsp);
    }

    #[test]
    fn test_validate_csp_unsafe_eval() {
        let findings = validate_csp("script-src 'unsafe-eval'");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, SecuritySeverity::High);
    }

    #[test]
    fn test_validate_csp_strict() {
        let findings = validate_csp("script-src 'self' 'nonce-abc123'");
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_check_insecure_urls() {
        let findings = check_insecure_urls(r#"<a href="http://example.com">Link</a>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, SecurityRule::InsecureHttp);
    }

    #[test]
    fn test_check_insecure_urls_https() {
        let findings = check_insecure_urls(r#"<a href="https://example.com">Link</a>"#);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_check_script_integrity_missing() {
        let findings = check_script_integrity(r#"<script src="https://cdn.example.com/lib.js"></script>"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, SecurityRule::MissingIntegrity);
    }

    #[test]
    fn test_check_script_integrity_present() {
        let findings = check_script_integrity(r#"<script src="lib.js" integrity="sha384-abc"></script>"#);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_audit_html() {
        let html = r#"<div id="app"></div><script>el.innerHTML = data;</script>"#;
        let report = audit_html(html);
        assert!(report.findings.iter().any(|f| f.rule == SecurityRule::XssInnerHTML));
    }

    #[test]
    fn test_escape_html() {
        let escaped = escape_html("<script>alert('xss')</script>");
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(!escaped.contains("<script>"));
    }

    #[test]
    fn test_escape_html_quotes() {
        let escaped = escape_html(r#"Hello "world" and 'universe'"#);
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&#x27;"));
    }

    #[test]
    fn test_is_safe_redirect() {
        assert!(is_safe_redirect("/dashboard"));
        assert!(is_safe_redirect("/users/123"));
        assert!(!is_safe_redirect("//evil.com"));
        assert!(!is_safe_redirect("https://evil.com"));
        assert!(!is_safe_redirect("http://evil.com"));
    }

    #[test]
    fn test_security_report_summary() {
        let mut report = SecurityReport::new();
        report.add(SecurityFinding {
            severity: SecuritySeverity::Critical,
            rule: SecurityRule::XssEval,
            description: "eval() found".to_string(),
            location: "test".to_string(),
            recommendation: "Remove eval".to_string(),
        });
        report.files_checked = 5;
        let summary = report.summary();
        assert!(summary.contains("1 critical"));
        assert!(summary.contains("5 files checked"));
    }
}
