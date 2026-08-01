//! Natural language component search — Goal 165.
//!
//! Allows AI agents and users to search for components using natural language
//! queries like "a button that submits a form" or "something to display a list".
//! Matches against component names, descriptions, tags, categories, and props.

use crate::component_registry::{self, ComponentMeta};

/// Search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matched component.
    pub component: ComponentMeta,
    /// Relevance score (0-100).
    pub score: u32,
    /// Which fields matched.
    pub matched_fields: Vec<String>,
}

impl SearchResult {
    /// Format as text.
    pub fn format_text(&self) -> String {
        let mut out = format!(
            "{} (score: {}) — {}\n",
            self.component.name, self.score, self.component.description
        );
        if !self.matched_fields.is_empty() {
            out.push_str(&format!("  Matched: {}\n", self.matched_fields.join(", ")));
        }
        out.push_str(&format!("  Category: {}\n", self.component.category));
        if !self.component.tags.is_empty() {
            out.push_str(&format!("  Tags: {}\n", self.component.tags.join(", ")));
        }
        out
    }

    /// Format as JSON.
    pub fn format_json(&self) -> String {
        let matched: Vec<String> = self.matched_fields.iter().map(|f| format!("\"{}\"", f)).collect();
        format!(
            r#"{{"component":{},"score":{},"matched_fields":[{}]}}"#,
            self.component.format_json(),
            self.score,
            matched.join(",")
        )
    }
}

/// Search for components using natural language.
pub fn search_nl(query: &str) -> Vec<SearchResult> {
    let components = component_registry::list_all();
    let query_lower = query.to_lowercase();
    let query_tokens = tokenize(&query_lower);

    let mut results: Vec<SearchResult> = components
        .iter()
        .map(|comp| score_component(comp, &query_lower, &query_tokens))
        .filter(|r| r.score > 0)
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}

/// Tokenize a query string.
fn tokenize(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty() && w.len() > 1)
        .collect()
}

/// Score a component against the query.
fn score_component(comp: &ComponentMeta, query_lower: &str, query_tokens: &[String]) -> SearchResult {
    let mut score: u32 = 0;
    let mut matched_fields = Vec::new();

    let name_lower = comp.name.to_lowercase();
    let desc_lower = comp.description.to_lowercase();
    let cat_lower = comp.category.to_lowercase();

    // Exact name match (highest score)
    if name_lower == query_lower {
        score += 100;
        matched_fields.push("name (exact)".to_string());
    } else if name_lower.contains(query_lower) || query_lower.contains(&name_lower) {
        score += 60;
        matched_fields.push("name".to_string());
    }

    // Name token matches
    for token in query_tokens {
        if name_lower.contains(token) {
            score += 20;
            if !matched_fields.contains(&"name".to_string()) {
                matched_fields.push("name".to_string());
            }
        }
    }

    // Description match
    if desc_lower.contains(query_lower) {
        score += 40;
        matched_fields.push("description".to_string());
    } else {
        for token in query_tokens {
            if desc_lower.contains(token) {
                score += 10;
                if !matched_fields.contains(&"description".to_string()) {
                    matched_fields.push("description".to_string());
                }
            }
        }
    }

    // Category match
    if cat_lower.contains(query_lower) || query_lower.contains(&cat_lower) {
        score += 30;
        matched_fields.push("category".to_string());
    } else {
        for token in query_tokens {
            if cat_lower.contains(token) {
                score += 15;
                if !matched_fields.contains(&"category".to_string()) {
                    matched_fields.push("category".to_string());
                }
            }
        }
    }

    // Tag matches
    for tag in &comp.tags {
        let tag_lower = tag.to_lowercase();
        if tag_lower == query_lower {
            score += 35;
            matched_fields.push("tags".to_string());
        } else if query_lower.contains(&tag_lower) || tag_lower.contains(query_lower) {
            score += 20;
            if !matched_fields.contains(&"tags".to_string()) {
                matched_fields.push("tags".to_string());
            }
        } else {
            for token in query_tokens {
                if tag_lower.contains(token) {
                    score += 8;
                    if !matched_fields.contains(&"tags".to_string()) {
                        matched_fields.push("tags".to_string());
                    }
                }
            }
        }
    }

    // Prop name matches
    for prop in &comp.props {
        let prop_lower = prop.name.to_lowercase();
        for token in query_tokens {
            if prop_lower.contains(token) {
                score += 12;
                if !matched_fields.contains(&"props".to_string()) {
                    matched_fields.push("props".to_string());
                }
            }
        }
    }

    // Semantic synonyms
    score += score_synonyms(query_lower, query_tokens, comp, &mut matched_fields);

    SearchResult {
        component: comp.clone(),
        score,
        matched_fields,
    }
}

/// Score based on common synonyms and related terms.
fn score_synonyms(
    query_lower: &str,
    query_tokens: &[String],
    comp: &ComponentMeta,
    matched: &mut Vec<String>,
) -> u32 {
    let mut score = 0u32;

    let synonyms: &[(&str, &[&str])] = &[
        ("button", &["click", "submit", "press", "action"]),
        ("input", &["text", "field", "form", "enter"]),
        ("list", &["items", "collection", "data", "rows"]),
        ("card", &["container", "box", "panel", "surface"]),
        ("modal", &["dialog", "popup", "overlay", "window"]),
        ("form", &["input", "submit", "validation", "field"]),
        ("table", &["grid", "data", "rows", "columns"]),
        ("nav", &["menu", "navigation", "links", "sidebar"]),
        ("image", &["photo", "picture", "img", "media"]),
        ("loading", &["spinner", "skeleton", "progress", "pending"]),
        ("error", &["error", "alert", "warning", "danger"]),
        ("layout", &["grid", "flex", "container", "wrapper"]),
    ];

    for (keyword, related) in synonyms {
        let kw_lower = keyword.to_lowercase();
        let comp_relevant = comp.name.to_lowercase().contains(&kw_lower)
            || comp.category.to_lowercase().contains(&kw_lower)
            || comp.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower));

        if comp_relevant {
            for related_word in related.iter() {
                if query_lower.contains(related_word) {
                    score += 15;
                    if !matched.contains(&"semantic".to_string()) {
                        matched.push("semantic".to_string());
                    }
                }
                for token in query_tokens {
                    if token == *related_word {
                        score += 10;
                    }
                }
            }
        }
    }

    score
}

/// Format search results as a text list.
pub fn format_results_text(results: &[SearchResult]) -> String {
    if results.is_empty() {
        return "No components found.".to_string();
    }
    let mut out = format!("Found {} component(s):\n\n", results.len());
    for r in results {
        out.push_str(&r.format_text());
        out.push('\n');
    }
    out
}

/// Format search results as JSON.
pub fn format_results_json(results: &[SearchResult]) -> String {
    let entries: Vec<String> = results.iter().map(|r| r.format_json()).collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_registry::{self, ComponentMeta, PropInfo};

    fn setup_test_components() {
        component_registry::clear();
        component_registry::register(ComponentMeta {
            name: "Button".to_string(),
            props_type: "ButtonProps".to_string(),
            props: vec![
                PropInfo::required("label", "String", "Button text"),
                PropInfo::optional("disabled", "bool", "false", "Disabled state"),
            ],
            is_island: false,
            uses_suspense: false,
            description: "A clickable button for actions".to_string(),
            category: "form".to_string(),
            tags: vec!["interactive".to_string(), "click".to_string()],
            example: "Button { label: \"OK\" }".to_string(),
        });
        component_registry::register(ComponentMeta {
            name: "TextInput".to_string(),
            props_type: "TextInputProps".to_string(),
            props: vec![
                PropInfo::required("value", "String", "Current value"),
                PropInfo::optional("placeholder", "String", "\"\"", "Placeholder text"),
            ],
            is_island: false,
            uses_suspense: false,
            description: "A text input field for forms".to_string(),
            category: "form".to_string(),
            tags: vec!["input".to_string(), "field".to_string()],
            example: "TextInput { value: \"\" }".to_string(),
        });
        component_registry::register(ComponentMeta {
            name: "UserList".to_string(),
            props_type: "UserListProps".to_string(),
            props: vec![],
            is_island: false,
            uses_suspense: false,
            description: "A list component for displaying data items".to_string(),
            category: "data".to_string(),
            tags: vec!["list".to_string(), "items".to_string()],
            example: "UserList { }".to_string(),
        });
        component_registry::register(ComponentMeta {
            name: "Modal".to_string(),
            props_type: "ModalProps".to_string(),
            props: vec![],
            is_island: false,
            uses_suspense: false,
            description: "A dialog popup overlay".to_string(),
            category: "feedback".to_string(),
            tags: vec!["modal".to_string(), "dialog".to_string()],
            example: "Modal { }".to_string(),
        });
    }

    #[test]
    fn test_search_exact_name() {
        setup_test_components();
        let results = search_nl("Button");
        assert!(!results.is_empty());
        assert_eq!(results[0].component.name, "Button");
        assert!(results[0].score >= 60);
    }

    #[test]
    fn test_search_synonym() {
        setup_test_components();
        let results = search_nl("click");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.component.name == "Button"));
    }

    #[test]
    fn test_search_description() {
        setup_test_components();
        let results = search_nl("displaying data items");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.component.name == "UserList"));
    }

    #[test]
    fn test_search_category() {
        setup_test_components();
        let results = search_nl("form");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.component.name == "Button"));
        assert!(results.iter().any(|r| r.component.name == "TextInput"));
    }

    #[test]
    fn test_search_tag() {
        setup_test_components();
        let results = search_nl("dialog");
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.component.name == "Modal"));
    }

    #[test]
    fn test_search_no_results() {
        setup_test_components();
        let results = search_nl("xyzabc123");
        assert!(results.is_empty());
    }

    #[test]
    fn test_results_sorted_by_score() {
        setup_test_components();
        let results = search_nl("form input");
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    fn test_format_results_text() {
        setup_test_components();
        let results = search_nl("button");
        let text = format_results_text(&results);
        assert!(text.contains("Button"));
        assert!(text.contains("score"));
    }

    #[test]
    fn test_format_results_json() {
        setup_test_components();
        let results = search_nl("button");
        let json = format_results_json(&results);
        assert!(json.starts_with("["));
        assert!(json.contains("\"component\""));
    }

    #[test]
    fn test_format_results_empty() {
        let text = format_results_text(&[]);
        assert!(text.contains("No components"));
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("a button for forms");
        assert!(tokens.contains(&"button".to_string()));
        assert!(tokens.contains(&"forms".to_string()));
        assert!(!tokens.contains(&"a".to_string())); // length <= 1 filtered
    }

    #[test]
    fn test_semantic_match_popup() {
        setup_test_components();
        let results = search_nl("popup");
        assert!(results.iter().any(|r| r.component.name == "Modal"));
    }

    #[test]
    fn test_semantic_match_submit() {
        setup_test_components();
        let results = search_nl("submit");
        assert!(results.iter().any(|r| r.component.name == "Button"));
    }
}
