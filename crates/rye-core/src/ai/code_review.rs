//! AI code review integration — Goal 164.
//!
//! Provides structured code review feedback for rye components.
//! AI agents can use this to self-review generated code before presenting to users.

use std::collections::HashMap;

/// Severity of a review finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Code will not compile or will panic.
    Error,
    /// Code works but has a correctness or performance issue.
    Warning,
    /// Code works but could be improved.
    Info,
    /// Code follows best practices.
    Praise,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Praise => "praise",
        }
    }
}

/// A single review finding.
#[derive(Debug, Clone)]
pub struct ReviewFinding {
    pub severity: Severity,
    pub line: usize,
    pub message: String,
    pub suggestion: Option<String>,
    pub error_code: Option<String>,
}

/// A complete code review result.
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub file: String,
    pub findings: Vec<ReviewFinding>,
    pub score: u8, // 0-100
    pub summary: String,
}

impl ReviewResult {
    /// Format as human-readable text.
    pub fn format_text(&self) -> String {
        let mut out = format!("Code Review: {} (score: {}/100)\n\n", self.file, self.score);
        out.push_str(&format!("{}\n\n", self.summary));

        if self.findings.is_empty() {
            out.push_str("No issues found.\n");
            return out;
        }

        for f in &self.findings {
            out.push_str(&format!(
                "  [{}] Line {}: {}\n",
                f.severity.as_str(),
                f.line,
                f.message
            ));
            if let Some(sug) = &f.suggestion {
                out.push_str(&format!("    Suggestion: {}\n", sug));
            }
            if let Some(code) = &f.error_code {
                out.push_str(&format!("    Error code: {} (run 'rpg explain {}')\n", code, code));
            }
        }
        out
    }

    /// Format as JSON.
    pub fn format_json(&self) -> String {
        let findings: Vec<String> = self
            .findings
            .iter()
            .map(|f| {
                let suggestion = f
                    .suggestion
                    .as_ref()
                    .map(|s| format!(",\"suggestion\":\"{}\"", json_escape(s)))
                    .unwrap_or_default();
                let error_code = f
                    .error_code
                    .as_ref()
                    .map(|c| format!(",\"error_code\":\"{}\"", c))
                    .unwrap_or_default();
                format!(
                    r#"{{"severity":"{}","line":{},"message":"{}"{}{}}}"#,
                    f.severity.as_str(),
                    f.line,
                    json_escape(&f.message),
                    suggestion,
                    error_code
                )
            })
            .collect();

        format!(
            r#"{{"file":"{}","findings":[{}],"score":{},"summary":"{}"}}"#,
            json_escape(&self.file),
            findings.join(","),
            self.score,
            json_escape(&self.summary)
        )
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Review a source file for rye-specific issues.
pub fn review_source(file_path: &str, source: &str) -> ReviewResult {
    let mut findings = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Check: Missing #[component] on functions returning template!
        if trimmed.starts_with("fn ") && is_pascal_case_fn(trimmed) {
            // Check if previous non-empty line has #[component]
            let has_component_attr = (0..i)
                .rev()
                .take(5)
                .any(|j| lines[j].trim().starts_with("#[component]"));
            if !has_component_attr {
                findings.push(ReviewFinding {
                    severity: Severity::Error,
                    line: line_num,
                    message: "Component function missing #[component] attribute".to_string(),
                    suggestion: Some("Add #[component] above the function".to_string()),
                    error_code: Some("R805".to_string()),
                });
            }
        }

        // Check: Signal used without .get() in templates
        if trimmed.contains("{") && !trimmed.starts_with("//") {
            for word in trimmed.split_whitespace() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                // Heuristic: lowercase identifier followed by { or } without .get()
                if is_signal_like_name(cleaned) {
                    let context = &trimmed[trimmed.find(cleaned).unwrap_or(0)..];
                    if !context.contains(".get()") && !context.contains(".set(") && !context.contains("let ") {
                        // Only flag if it looks like it's in a template expression
                        if trimmed.contains("{") && trimmed.contains("}") {
                            findings.push(ReviewFinding {
                                severity: Severity::Warning,
                                line: line_num,
                                message: format!("Signal '{}' may need .get() to read its value", cleaned),
                                suggestion: Some(format!("Use {}.get() instead of {}", cleaned, cleaned)),
                                error_code: Some("R802".to_string()),
                            });
                        }
                    }
                }
            }
        }

        // Check: Closure without 'move' in event handlers
        if (trimmed.contains("onclick:") || trimmed.contains("oninput:") || trimmed.contains("onchange:"))
            && trimmed.contains("|_|")
            && !trimmed.contains("move |")
        {
            findings.push(ReviewFinding {
                severity: Severity::Error,
                line: line_num,
                message: "Event handler closure missing 'move' keyword".to_string(),
                suggestion: Some("Add 'move' before the closure: move |_|".to_string()),
                error_code: Some("R801".to_string()),
            });
        }

        // Check: Direct assignment to Signal (count = 5 instead of count.set(5))
        if !trimmed.starts_with("//") && !trimmed.starts_with("let ") {
            if let Some(eq_pos) = trimmed.find(" = ") {
                let lhs = trimmed[..eq_pos].trim();
                if is_signal_like_name(lhs) && !lhs.contains(".") && !lhs.contains("::") {
                    findings.push(ReviewFinding {
                        severity: Severity::Error,
                        line: line_num,
                        message: format!("Direct assignment to Signal '{}' — use .set() instead", lhs),
                        suggestion: Some(format!("Use {}.set(value) instead of {} = value", lhs, lhs)),
                        error_code: Some("R803".to_string()),
                    });
                }
            }
        }

        // Check: use_effect for derived state (should be Memo)
        if trimmed.contains("use_effect") && trimmed.contains(".set(") {
            findings.push(ReviewFinding {
                severity: Severity::Warning,
                line: line_num,
                message: "use_effect used to compute derived state — consider Memo instead".to_string(),
                suggestion: Some("Replace use_effect + .set() with Memo::new(move || ...)".to_string()),
                error_code: Some("R806".to_string()),
            });
        }

        // Check: Unnecessary .clone()
        if trimmed.contains(".clone()") && (trimmed.contains("props.") || trimmed.contains(".get().clone()")) {
            findings.push(ReviewFinding {
                severity: Severity::Info,
                line: line_num,
                message: "Unnecessary .clone() — rye props are borrowed, not moved".to_string(),
                suggestion: Some("Remove .clone() if the value is only read".to_string()),
                error_code: Some("R807".to_string()),
            });
        }

        // Check: Raw async instead of use_resource
        if (trimmed.contains("tokio::spawn") || trimmed.contains("spawn_local"))
            && trimmed.contains("async")
        {
            findings.push(ReviewFinding {
                severity: Severity::Warning,
                line: line_num,
                message: "Raw async spawn detected — use use_resource for reactive async data".to_string(),
                suggestion: Some("Replace with use_resource(move || async { ... })".to_string()),
                error_code: Some("R809".to_string()),
            });
        }

        // Check: snake_case component name
        if trimmed.starts_with("fn ") && is_component_candidate_in_range(&lines, i) {
            let name = extract_fn_name(trimmed);
            if let Some(ref n) = name {
                if !n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    findings.push(ReviewFinding {
                        severity: Severity::Error,
                        line: line_num,
                        message: format!("Component '{}' should be PascalCase", n),
                        suggestion: Some(format!("Rename to {}", to_pascal(n))),
                        error_code: Some("R804".to_string()),
                    });
                }
            }
        }

        // Praise: Using Memo correctly
        if trimmed.contains("Memo::new") {
            findings.push(ReviewFinding {
                severity: Severity::Praise,
                line: line_num,
                message: "Good use of Memo for derived state".to_string(),
                suggestion: None,
                error_code: None,
            });
        }

        // Praise: Using provide_context
        if trimmed.contains("provide_context") {
            findings.push(ReviewFinding {
                severity: Severity::Praise,
                line: line_num,
                message: "Good use of provide_context for state sharing".to_string(),
                suggestion: None,
                error_code: None,
            });
        }

        // Praise: Using Suspense
        if trimmed.contains("Suspense") {
            findings.push(ReviewFinding {
                severity: Severity::Praise,
                line: line_num,
                message: "Good use of Suspense for async loading states".to_string(),
                suggestion: None,
                error_code: None,
            });
        }
    }

    // Calculate score
    let errors = findings.iter().filter(|f| f.severity == Severity::Error).count();
    let warnings = findings.iter().filter(|f| f.severity == Severity::Warning).count();
    let praises = findings.iter().filter(|f| f.severity == Severity::Praise).count();
    let score = calculate_score(errors, warnings, praises);

    let summary = generate_summary(errors, warnings, findings.len() - errors - warnings - praises, praises);

    ReviewResult {
        file: file_path.to_string(),
        findings,
        score,
        summary,
    }
}

fn calculate_score(errors: usize, warnings: usize, praises: usize) -> u8 {
    let mut score: i32 = 100;
    score -= (errors * 20) as i32;
    score -= (warnings * 10) as i32;
    score += (praises * 5) as i32;
    score.clamp(0, 100) as u8
}

fn generate_summary(errors: usize, warnings: usize, info: usize, praises: usize) -> String {
    let mut parts = Vec::new();
    if errors > 0 {
        parts.push(format!("{} error(s)", errors));
    }
    if warnings > 0 {
        parts.push(format!("{} warning(s)", warnings));
    }
    if info > 0 {
        parts.push(format!("{} info(s)", info));
    }
    if praises > 0 {
        parts.push(format!("{} praise(s)", praises));
    }
    if parts.is_empty() {
        "Clean code, no issues found.".to_string()
    } else {
        parts.join(", ")
    }
}

fn is_pascal_case_fn(line: &str) -> bool {
    if let Some(name) = extract_fn_name(line) {
        name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
    } else {
        false
    }
}

fn is_component_candidate(line: &str) -> bool {
    // A function that uses template! or returns an Element
    line.contains("template!") || line.contains("-> Element") || line.contains("-> impl")
}

fn is_component_candidate_in_range(lines: &[&str], idx: usize) -> bool {
    // Check current line plus next 10 lines for template! or Element return type
    let end = (idx + 10).min(lines.len());
    for j in idx..end {
        let l = lines[j].trim();
        if l.contains("template!") || l.contains("-> Element") || l.contains("-> impl") {
            return true;
        }
        // Stop at next fn definition
        if j > idx && l.starts_with("fn ") {
            break;
        }
    }
    // Also check if #[component] is above this fn
    if idx > 0 {
        for j in (0..idx).rev().take(5) {
            let l = lines[j].trim();
            if l.starts_with("#[component]") {
                return true;
            }
            if !l.starts_with("#[") && !l.is_empty() {
                break;
            }
        }
    }
    false
}

fn extract_fn_name(line: &str) -> Option<String> {
    let after_fn = line.find("fn ")?;
    let rest = &line[after_fn + 3..];
    let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn is_signal_like_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map(|c| c.is_lowercase()).unwrap_or(false)
        && !is_rust_keyword(s)
        && s.len() > 1
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "let" | "mut" | "if" | "else" | "for" | "while" | "loop" | "match" | "return"
            | "fn" | "struct" | "enum" | "impl" | "trait" | "use" | "mod" | "pub" | "self"
            | "super" | "crate" | "move" | "async" | "await" | "dyn" | "ref" | "static"
            | "const" | "true" | "false" | "div" | "span" | "p" | "h1" | "h2" | "button"
            | "input" | "form" | "img" | "a" | "ul" | "li" | "table" | "tr" | "td"
    )
}

fn to_pascal(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Review multiple files and return combined results.
pub fn review_files(files: &[(String, String)]) -> Vec<ReviewResult> {
    files
        .iter()
        .map(|(path, source)| review_source(path, source))
        .collect()
}

/// Get an overall score across multiple files.
pub fn overall_score(results: &[ReviewResult]) -> u8 {
    if results.is_empty() {
        return 100;
    }
    let total: u32 = results.iter().map(|r| r.score as u32).sum();
    (total / results.len() as u32) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_missing_component_attr() {
        let source = r#"
fn MyComponent() {
    template! { div { "Hello" } }
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Error && f.error_code == Some("R805".to_string())));
    }

    #[test]
    fn test_review_with_component_attr() {
        let source = r#"
#[component]
fn MyComponent() {
    template! { div { "Hello" } }
}
"#;
        let result = review_source("test.rs", source);
        assert!(!result.findings.iter().any(|f| f.error_code == Some("R805".to_string())));
    }

    #[test]
    fn test_review_closure_missing_move() {
        let source = r#"
#[component]
fn MyComponent() {
    button { onclick: |_| count.set(1) }
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.findings.iter().any(|f| f.error_code == Some("R801".to_string())));
    }

    #[test]
    fn test_review_closure_with_move() {
        let source = r#"
#[component]
fn MyComponent() {
    button { onclick: move |_| count.set(1) }
}
"#;
        let result = review_source("test.rs", source);
        assert!(!result.findings.iter().any(|f| f.error_code == Some("R801".to_string())));
    }

    #[test]
    fn test_review_use_effect_for_derived() {
        let source = r#"
#[component]
fn MyComponent() {
    use_effect(move || { derived.set(a.get() + b.get()); });
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.findings.iter().any(|f| f.error_code == Some("R806".to_string())));
    }

    #[test]
    fn test_review_praise_for_memo() {
        let source = r#"
#[component]
fn MyComponent() {
    let derived = Memo::new(move || a.get() + b.get());
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Praise));
    }

    #[test]
    fn test_review_score() {
        let source = r#"
#[component]
fn MyComponent() {
    let derived = Memo::new(move || a.get() + b.get());
    provide_context(derived);
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.score > 90); // Should be high with only praises
    }

    #[test]
    fn test_review_score_with_errors() {
        let source = r#"
fn MyComponent() {
    button { onclick: |_| count.set(1) }
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.score < 80); // Should be low with errors
    }

    #[test]
    fn test_review_format_text() {
        let source = "fn BadName() { }";
        let result = review_source("test.rs", source);
        let text = result.format_text();
        assert!(text.contains("Code Review"));
        assert!(text.contains("score"));
    }

    #[test]
    fn test_review_format_json() {
        let source = "#[component]\nfn Good() { }";
        let result = review_source("test.rs", source);
        let json = result.format_json();
        assert!(json.contains("\"file\":\"test.rs\""));
        assert!(json.contains("\"score\""));
    }

    #[test]
    fn test_review_files_multiple() {
        let files = vec![
            ("a.rs".to_string(), "#[component]\nfn Good() {}".to_string()),
            ("b.rs".to_string(), "fn Bad() {}".to_string()),
        ];
        let results = review_files(&files);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_overall_score() {
        let results = vec![
            ReviewResult { file: "a".into(), findings: vec![], score: 80, summary: "ok".into() },
            ReviewResult { file: "b".into(), findings: vec![], score: 60, summary: "ok".into() },
        ];
        assert_eq!(overall_score(&results), 70);
    }

    #[test]
    fn test_overall_score_empty() {
        assert_eq!(overall_score(&[]), 100);
    }

    #[test]
    fn test_snake_case_component_name() {
        let source = r#"
#[component]
fn my_button() {
    template! { div { "Hi" } }
}
"#;
        let result = review_source("test.rs", source);
        assert!(result.findings.iter().any(|f| f.error_code == Some("R804".to_string())));
    }
}
