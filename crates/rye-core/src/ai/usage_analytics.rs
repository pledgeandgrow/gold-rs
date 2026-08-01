//! Component usage analytics — Goal 163.
//!
//! Tracks which components are used where in a project.
//! Helps AI agents understand common patterns and prop combinations.

use std::collections::HashMap;
use std::sync::Mutex;

/// A single usage instance of a component.
#[derive(Debug, Clone)]
pub struct UsageRecord {
    /// Component name.
    pub component: String,
    /// File path where used.
    pub file: String,
    /// Line number (1-indexed).
    pub line: usize,
    /// Props used in this instance (name=value pairs as strings).
    pub props_used: Vec<String>,
    /// Whether this is a definition site (#[component]) or usage site.
    pub is_definition: bool,
}

/// Aggregated usage stats for a component.
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Component name.
    pub component: String,
    /// Total number of usages (excluding definitions).
    pub usage_count: usize,
    /// Files where this component is used.
    pub files: Vec<String>,
    /// Most common prop combinations.
    pub common_props: Vec<(String, usize)>,
    /// Whether this component is defined in the project.
    pub is_defined: bool,
}

impl UsageStats {
    /// Format as text.
    pub fn format_text(&self) -> String {
        let mut out = format!("{} ({} usages)\n", self.component, self.usage_count);
        if self.is_defined {
            out.push_str("  Defined in project\n");
        }
        if !self.files.is_empty() {
            out.push_str(&format!("  Used in {} file(s)\n", self.files.len()));
            for f in &self.files {
                out.push_str(&format!("    - {}\n", f));
            }
        }
        if !self.common_props.is_empty() {
            out.push_str("  Common props:\n");
            for (prop, count) in &self.common_props {
                out.push_str(&format!("    - {} ({}x)\n", prop, count));
            }
        }
        out
    }

    /// Format as JSON.
    pub fn format_json(&self) -> String {
        let files: Vec<String> = self.files.iter().map(|f| format!("\"{}\"", f.replace('\\', "\\\\"))).collect();
        let props: Vec<String> = self
            .common_props
            .iter()
            .map(|(p, c)| format!(r#"{{"prop":"{}","count":{}}}"#, p, c))
            .collect();
        format!(
            r#"{{"component":"{}","usage_count":{},"files":[{}],"common_props":[{}],"is_defined":{}}}"#,
            self.component, self.usage_count, files.join(","), props.join(","), self.is_defined
        )
    }
}

/// Global usage tracker.
static TRACKER: std::sync::OnceLock<Mutex<Vec<UsageRecord>>> = std::sync::OnceLock::new();

fn tracker() -> &'static Mutex<Vec<UsageRecord>> {
    TRACKER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Record a component usage.
pub fn record(record: UsageRecord) {
    tracker().lock().unwrap().push(record);
}

/// Record a component definition.
pub fn record_definition(component: &str, file: &str, line: usize) {
    record(UsageRecord {
        component: component.to_string(),
        file: file.to_string(),
        line,
        props_used: vec![],
        is_definition: true,
    });
}

/// Record a component usage.
pub fn record_usage(component: &str, file: &str, line: usize, props: &[&str]) {
    record(UsageRecord {
        component: component.to_string(),
        file: file.to_string(),
        line,
        props_used: props.iter().map(|p| p.to_string()).collect(),
        is_definition: false,
    });
}

/// Get all usage records.
pub fn all_records() -> Vec<UsageRecord> {
    tracker().lock().unwrap().clone()
}

/// Compute stats from a slice of records (no locking).
fn stats_from_records(records: &[UsageRecord], component: &str) -> Option<UsageStats> {
    let component_records: Vec<&UsageRecord> = records.iter().filter(|r| r.component == component).collect();
    if component_records.is_empty() {
        return None;
    }

    let usage_count = component_records.iter().filter(|r| !r.is_definition).count();
    let is_defined = component_records.iter().any(|r| r.is_definition);

    let mut files: Vec<String> = component_records
        .iter()
        .filter(|r| !r.is_definition)
        .map(|r| r.file.clone())
        .collect();
    files.sort();
    files.dedup();

    let mut prop_counts: HashMap<String, usize> = HashMap::new();
    for r in &component_records {
        if !r.is_definition {
            for prop in &r.props_used {
                *prop_counts.entry(prop.clone()).or_insert(0) += 1;
            }
        }
    }
    let mut common_props: Vec<(String, usize)> = prop_counts.into_iter().collect();
    common_props.sort_by(|a, b| b.1.cmp(&a.1));
    common_props.truncate(10);

    Some(UsageStats {
        component: component.to_string(),
        usage_count,
        files,
        common_props,
        is_defined,
    })
}

/// Get aggregated stats for a specific component.
pub fn stats_for(component: &str) -> Option<UsageStats> {
    let records = tracker().lock().unwrap();
    stats_from_records(&records, component)
}

/// Get stats for all components.
pub fn all_stats() -> Vec<UsageStats> {
    let records = tracker().lock().unwrap();
    let mut components: Vec<String> = records.iter().map(|r| r.component.clone()).collect();
    components.sort();
    components.dedup();
    components.into_iter().filter_map(|c| stats_from_records(&records, &c)).collect()
}

/// Get the most used components.
pub fn most_used(limit: usize) -> Vec<UsageStats> {
    let mut stats = all_stats();
    stats.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
    stats.truncate(limit);
    stats
}

/// Get unused components (defined but never used).
pub fn unused_components() -> Vec<String> {
    all_stats()
        .into_iter()
        .filter(|s| s.is_defined && s.usage_count == 0)
        .map(|s| s.component)
        .collect()
}

/// Clear all records (for testing).
pub fn clear() {
    tracker().lock().unwrap().clear();
}

/// Scan source code and record component usages.
/// This is a simple text-based scanner that looks for PascalCase identifiers.
pub fn scan_source(file_path: &str, source: &str) {
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check for #[component] definitions
        if trimmed.starts_with("#[component]") {
            // Look at next line for fn name
            let lines: Vec<&str> = source.lines().collect();
            if i + 1 < lines.len() {
                let fn_line = lines[i + 1].trim();
                if let Some(name) = extract_pascal_name(fn_line) {
                    record_definition(&name, file_path, i + 1);
                }
            }
        }

        // Check for component usages in template! blocks
        // Look for PascalCase identifiers that aren't fn definitions
        if !trimmed.starts_with("fn ") && !trimmed.starts_with("//") {
            for word in trimmed.split_whitespace() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if is_component_name(cleaned) && !is_rust_keyword(cleaned) {
                    // Check if it's a usage (has { or ( after it on the same line)
                    if trimmed.contains(&format!("{} {{", cleaned)) || trimmed.contains(&format!("{}(", cleaned)) {
                        record_usage(cleaned, file_path, i + 1, &[]);
                    }
                }
            }
        }
    }
}

fn extract_pascal_name(line: &str) -> Option<String> {
    let after_fn = line.find("fn ")?;
    let rest = &line[after_fn + 3..];
    let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next()?;
    if is_component_name(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn is_component_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_uppercase()) && s.len() > 1 && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "Self" | "String" | "Vec" | "Option" | "Result" | "Box" | "Some" | "None" | "Ok" | "Err"
            | "True" | "False"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mutex to serialize tests that share the global tracker.
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_record_and_stats() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_definition("Button", "src/components/button.rs", 5);
        record_usage("Button", "src/pages/home.rs", 20, &["label", "disabled"]);
        record_usage("Button", "src/pages/about.rs", 15, &["label"]);

        let stats = stats_for("Button").unwrap();
        assert_eq!(stats.usage_count, 2);
        assert!(stats.is_defined);
        assert_eq!(stats.files.len(), 2);
        assert!(stats.common_props.iter().any(|(p, c)| p == "label" && *c == 2));
    }

    #[test]
    fn test_stats_nonexistent() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        let stats = stats_for("Nonexistent");
        assert!(stats.is_none());
    }

    #[test]
    fn test_most_used() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_usage("A", "f.rs", 1, &[]);
        record_usage("A", "f.rs", 2, &[]);
        record_usage("A", "f.rs", 3, &[]);
        record_usage("B", "f.rs", 4, &[]);
        record_usage("B", "f.rs", 5, &[]);
        record_usage("C", "f.rs", 6, &[]);

        let top = most_used(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].component, "A");
        assert_eq!(top[0].usage_count, 3);
        assert_eq!(top[1].component, "B");
    }

    #[test]
    fn test_unused_components() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_definition("Unused", "src/components/unused.rs", 1);
        record_definition("Used", "src/components/used.rs", 1);
        record_usage("Used", "src/pages/home.rs", 1, &[]);

        let unused = unused_components();
        assert!(unused.contains(&"Unused".to_string()));
        assert!(!unused.contains(&"Used".to_string()));
    }

    #[test]
    fn test_usage_stats_format_text() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_usage("Button", "src/home.rs", 1, &["label"]);
        let stats = stats_for("Button").unwrap();
        let text = stats.format_text();
        assert!(text.contains("Button"));
        assert!(text.contains("1 usages"));
    }

    #[test]
    fn test_usage_stats_format_json() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_usage("Button", "src/home.rs", 1, &["label"]);
        let stats = stats_for("Button").unwrap();
        let json = stats.format_json();
        assert!(json.contains("\"component\":\"Button\""));
        assert!(json.contains("\"usage_count\":1"));
    }

    #[test]
    fn test_scan_source() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        let source = r#"
#[component]
fn MyButton(props: MyButtonProps) {
    div {
        MyButton { label: "OK" }
    }
}
"#;
        scan_source("test.rs", source);
        let stats = stats_for("MyButton");
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert!(stats.is_defined);
    }

    #[test]
    fn test_is_component_name() {
        assert!(is_component_name("Button"));
        assert!(is_component_name("MyComponent"));
        assert!(!is_component_name("button"));
        assert!(!is_component_name("fn"));
        assert!(!is_component_name(""));
    }

    #[test]
    fn test_all_stats() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear();
        record_usage("A", "f.rs", 1, &[]);
        record_usage("B", "f.rs", 2, &[]);
        let stats = all_stats();
        assert_eq!(stats.len(), 2);
    }
}
