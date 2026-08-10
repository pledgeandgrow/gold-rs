//! Component discovery API — Goal 153.
//!
//! Provides a registry for discovering components at runtime.
//! AI agents can query this API to understand what components exist,
//! their props, and how to use them before generating code.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Information about a component prop.
#[derive(Debug, Clone)]
pub struct PropInfo {
    /// Prop name.
    pub name: String,
    /// Type name (e.g. "String", "Signal<i32>").
    pub type_name: String,
    /// Whether the prop is required.
    pub required: bool,
    /// Default value if optional (as a string representation).
    pub default: Option<String>,
    /// Human-readable description.
    pub description: String,
}

impl PropInfo {
    /// Create a new required prop.
    pub fn required(name: &str, type_name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            required: true,
            default: None,
            description: description.to_string(),
        }
    }

    /// Create a new optional prop with a default value.
    pub fn optional(name: &str, type_name: &str, default: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            required: false,
            default: Some(default.to_string()),
            description: description.to_string(),
        }
    }
}

/// Information about a registered component.
#[derive(Debug, Clone)]
pub struct ComponentMeta {
    /// Component name (PascalCase).
    pub name: String,
    /// The props struct name (e.g. "ButtonProps").
    pub props_type: String,
    /// List of props.
    pub props: Vec<PropInfo>,
    /// Whether this component is an island (client-only hydrated).
    pub is_island: bool,
    /// Whether this component uses suspense (async data).
    pub uses_suspense: bool,
    /// Human-readable description of what the component does.
    pub description: String,
    /// Category (e.g. "form", "layout", "feedback", "data").
    pub category: String,
    /// Tags for searchability.
    pub tags: Vec<String>,
    /// Correct usage example.
    pub example: String,
}

impl ComponentMeta {
    /// Format as human-readable text.
    pub fn format_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Component: {}\n", self.name));
        out.push_str(&format!("Category: {}\n", self.category));
        out.push_str(&format!("Description: {}\n", self.description));

        let flags = match (self.is_island, self.uses_suspense) {
            (true, true) => " [island] [suspense]",
            (true, false) => " [island]",
            (false, true) => " [suspense]",
            (false, false) => "",
        };
        if !flags.is_empty() {
            out.push_str(&format!("Flags:{}\n", flags));
        }

        if !self.props.is_empty() {
            out.push_str("\nProps:\n");
            for prop in &self.props {
                let req = if prop.required {
                    "required"
                } else {
                    "optional"
                };
                let default = prop
                    .default
                    .as_ref()
                    .map(|d| format!(", default: {}", d))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "  {} ({}): {} — {}{}\n",
                    prop.name, req, prop.type_name, prop.description, default
                ));
            }
        }

        if !self.tags.is_empty() {
            out.push_str(&format!("\nTags: {}\n", self.tags.join(", ")));
        }

        out.push_str(&format!("\nExample:\n{}\n", self.example));

        out
    }

    /// Format as JSON for AI agent consumption.
    pub fn format_json(&self) -> String {
        let props: Vec<String> = self
            .props
            .iter()
            .map(|p| {
                let default = p
                    .default
                    .as_ref()
                    .map(|d| format!(",\"default\":\"{}\"", json_escape(d)))
                    .unwrap_or_default();
                format!(
                    r#"{{"name":"{}","type":"{}","required":{},"description":"{}"{} }}"#,
                    json_escape(&p.name),
                    json_escape(&p.type_name),
                    p.required,
                    json_escape(&p.description),
                    default
                )
            })
            .collect();

        let tags: Vec<String> = self
            .tags
            .iter()
            .map(|t| format!("\"{}\"", json_escape(t)))
            .collect();

        format!(
            r#"{{"name":"{}","props_type":"{}","props":[{}],"is_island":{},"uses_suspense":{},"description":"{}","category":"{}","tags":[{}],"example":"{}"}}"#,
            json_escape(&self.name),
            json_escape(&self.props_type),
            props.join(","),
            self.is_island,
            self.uses_suspense,
            json_escape(&self.description),
            json_escape(&self.category),
            tags.join(","),
            json_escape(&self.example)
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

/// Global component registry.
static REGISTRY: OnceLock<Mutex<HashMap<String, ComponentMeta>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, ComponentMeta>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a component in the global registry.
pub fn register(meta: ComponentMeta) {
    let name = meta.name.clone();
    registry().lock().unwrap().insert(name, meta);
}

/// Look up a component by name.
pub fn find(name: &str) -> Option<ComponentMeta> {
    registry().lock().unwrap().get(name).cloned()
}

/// List all registered components.
pub fn list_all() -> Vec<ComponentMeta> {
    registry().lock().unwrap().values().cloned().collect()
}

/// Search components by name, tag, or category.
pub fn search(query: &str) -> Vec<ComponentMeta> {
    let q = query.to_lowercase();
    registry()
        .lock()
        .unwrap()
        .values()
        .filter(|c| {
            c.name.to_lowercase().contains(&q)
                || c.category.to_lowercase().contains(&q)
                || c.description.to_lowercase().contains(&q)
                || c.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .cloned()
        .collect()
}

/// List components by category.
pub fn list_by_category(category: &str) -> Vec<ComponentMeta> {
    registry()
        .lock()
        .unwrap()
        .values()
        .filter(|c| c.category == category)
        .cloned()
        .collect()
}

/// Get all categories that have registered components.
pub fn categories() -> Vec<String> {
    let reg = registry().lock().unwrap();
    let mut cats: Vec<String> = reg.values().map(|c| c.category.clone()).collect();
    cats.sort();
    cats.dedup();
    cats
}

/// Format all components as JSON array (for AI agent discovery).
pub fn format_all_json() -> String {
    let all = list_all();
    let entries: Vec<String> = all.iter().map(|c| c.format_json()).collect();
    format!("[{}]", entries.join(","))
}

/// Clear the registry (useful for testing).
pub fn clear() {
    registry().lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_component() -> ComponentMeta {
        ComponentMeta {
            name: "Button".to_string(),
            props_type: "ButtonProps".to_string(),
            props: vec![
                PropInfo::required("label", "String", "Button text content"),
                PropInfo::optional(
                    "disabled",
                    "bool",
                    "false",
                    "Whether the button is disabled",
                ),
                PropInfo::optional("variant", "String", "\"primary\"", "Visual variant"),
            ],
            is_island: false,
            uses_suspense: false,
            description: "A clickable button component".to_string(),
            category: "form".to_string(),
            tags: vec!["interactive".to_string(), "form".to_string()],
            example: "Button { label: \"Submit\", disabled: false }".to_string(),
        }
    }

    #[test]
    fn test_register_and_find() {
        clear();
        register(sample_component());
        let found = find("Button");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Button");
    }

    #[test]
    fn test_find_nonexistent() {
        clear();
        let found = find("Nonexistent");
        assert!(found.is_none());
    }

    #[test]
    fn test_search() {
        clear();
        register(sample_component());
        let results = search("button");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Button");
    }

    #[test]
    fn test_search_by_tag() {
        clear();
        register(sample_component());
        let results = search("interactive");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_list_by_category() {
        clear();
        register(sample_component());
        let results = list_by_category("form");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_categories() {
        clear();
        register(sample_component());
        register(ComponentMeta {
            name: "Card".to_string(),
            props_type: "CardProps".to_string(),
            props: vec![],
            is_island: false,
            uses_suspense: false,
            description: "A layout card".to_string(),
            category: "layout".to_string(),
            tags: vec![],
            example: "Card { }".to_string(),
        });
        let cats = categories();
        assert!(cats.contains(&"form".to_string()));
        assert!(cats.contains(&"layout".to_string()));
    }

    #[test]
    fn test_format_text() {
        let comp = sample_component();
        let text = comp.format_text();
        assert!(text.contains("Button"));
        assert!(text.contains("form"));
        assert!(text.contains("Props:"));
        assert!(text.contains("label"));
        assert!(text.contains("required"));
        assert!(text.contains("disabled"));
        assert!(text.contains("optional"));
        assert!(text.contains("Example:"));
    }

    #[test]
    fn test_format_json() {
        let comp = sample_component();
        let json = comp.format_json();
        assert!(json.contains("\"name\":\"Button\""));
        assert!(json.contains("\"category\":\"form\""));
        assert!(json.contains("\"is_island\":false"));
        assert!(json.contains("\"props\""));
        assert!(json.contains("\"required\":true"));
    }

    #[test]
    fn test_format_all_json() {
        clear();
        register(sample_component());
        let json = format_all_json();
        assert!(json.starts_with("["));
        assert!(json.ends_with("]"));
        assert!(json.contains("\"name\":\"Button\""));
    }

    #[test]
    fn test_prop_info_required() {
        let prop = PropInfo::required("title", "String", "The title");
        assert!(prop.required);
        assert!(prop.default.is_none());
        assert_eq!(prop.name, "title");
    }

    #[test]
    fn test_prop_info_optional() {
        let prop = PropInfo::optional("count", "i32", "0", "Counter value");
        assert!(!prop.required);
        assert_eq!(prop.default.as_deref(), Some("0"));
    }
}
