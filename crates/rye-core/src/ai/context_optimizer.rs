//! AI context window optimization — Goal 160.
//!
//! Summarizes rye APIs for AI agent context windows.
//! Only includes relevant information, not full source code.

use crate::component_registry::{self, ComponentMeta};
use crate::error_codes;

/// Approximate token count (4 chars ≈ 1 token).
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Context budget for AI agents.
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// Maximum tokens to spend.
    pub max_tokens: usize,
    /// Tokens used so far.
    pub used: usize,
}

impl ContextBudget {
    /// Create a new budget with the given token limit.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            used: 0,
        }
    }

    /// Check if there's room for more tokens.
    pub fn has_room(&self, tokens: usize) -> bool {
        self.used + tokens <= self.max_tokens
    }

    /// Spend tokens from the budget.
    pub fn spend(&mut self, tokens: usize) -> bool {
        if self.has_room(tokens) {
            self.used += tokens;
            true
        } else {
            false
        }
    }

    /// Remaining tokens.
    pub fn remaining(&self) -> usize {
        self.max_tokens.saturating_sub(self.used)
    }
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self::new(4000)
    }
}

/// A compact component summary for AI context.
#[derive(Debug, Clone)]
pub struct ComponentSummary {
    pub name: String,
    pub props: Vec<(String, String, bool)>, // (name, type, required)
    pub category: String,
    pub description: String,
}

impl ComponentSummary {
    /// Create from full ComponentMeta.
    pub fn from_meta(meta: &ComponentMeta) -> Self {
        Self {
            name: meta.name.clone(),
            props: meta
                .props
                .iter()
                .map(|p| (p.name.clone(), p.type_name.clone(), p.required))
                .collect(),
            category: meta.category.clone(),
            description: meta.description.clone(),
        }
    }

    /// Format as a compact one-liner for context-constrained AI.
    pub fn format_compact(&self) -> String {
        let props_str = if self.props.is_empty() {
            String::new()
        } else {
            let parts: Vec<String> = self
                .props
                .iter()
                .map(|(n, t, req)| {
                    format!("{}: {}{}", n, t, if *req { "" } else { "?" })
                })
                .collect();
            format!("({})", parts.join(", "))
        };
        format!("{}{} — {}", self.name, props_str, self.description)
    }

    /// Format as a detailed but token-efficient entry.
    pub fn format_detailed(&self) -> String {
        let mut out = format!("## {}\n", self.name);
        out.push_str(&format!("Category: {}\n", self.category));
        out.push_str(&format!("{}\n", self.description));
        if !self.props.is_empty() {
            out.push_str("Props:\n");
            for (name, ty, req) in &self.props {
                let marker = if *req { "required" } else { "optional" };
                out.push_str(&format!("  - {} ({}): {}\n", name, marker, ty));
            }
        }
        out
    }
}

/// Optimize component list for AI context window.
/// Returns compact summaries that fit within the token budget.
pub fn optimize_components(budget: &mut ContextBudget) -> Vec<ComponentSummary> {
    let all = component_registry::list_all();
    optimize_component_list(&all, budget)
}

/// Optimize a specific set of components for AI context.
pub fn optimize_component_list(
    components: &[ComponentMeta],
    budget: &mut ContextBudget,
) -> Vec<ComponentSummary> {
    let mut summaries = Vec::new();

    for comp in components {
        let summary = ComponentSummary::from_meta(comp);
        let compact = summary.format_compact();
        let tokens = estimate_tokens(&compact);

        if budget.spend(tokens) {
            summaries.push(summary);
        } else {
            break;
        }
    }

    summaries
}

/// Generate a context-optimized summary of all rye error codes.
/// Only includes code + message, not full details.
pub fn optimize_error_codes(budget: &mut ContextBudget) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for code in error_codes::all_codes() {
        let entry = format!("{}: {}", code.code, code.message);
        let tokens = estimate_tokens(&entry);
        if budget.spend(tokens) {
            result.push((code.code.to_string(), code.message.to_string()));
        } else {
            break;
        }
    }
    result
}

/// Generate a complete AI context package within a token budget.
/// Includes: error codes summary, component summaries, and prompt template list.
pub fn generate_context_package(max_tokens: usize) -> String {
    let mut budget = ContextBudget::new(max_tokens);
    let mut out = String::new();

    // Error codes summary
    out.push_str("# rye Error Codes\n");
    let codes = optimize_error_codes(&mut budget);
    for (code, msg) in &codes {
        out.push_str(&format!("- {}: {}\n", code, msg));
    }

    // Component summaries
    if budget.remaining() > 100 {
        out.push_str("\n# Available Components\n");
        let components = optimize_components(&mut budget);
        for comp in &components {
            out.push_str(&format!("- {}\n", comp.format_compact()));
        }
    }

    // Prompt templates
    if budget.remaining() > 100 {
        out.push_str("\n# Prompt Templates\n");
        for t in crate::ai::prompt_templates::all_templates() {
            let entry = format!("- {}: {}", t.id, t.description);
            let tokens = estimate_tokens(&entry);
            if budget.spend(tokens) {
                out.push_str(&entry);
                out.push('\n');
            }
        }
    }

    out.push_str(&format!(
        "\n# Context Budget: {}/{} tokens used\n",
        budget.used, budget.max_tokens
    ));

    out
}

/// Generate a focused context for a specific query.
/// Only includes components and error codes relevant to the query.
pub fn generate_focused_context(query: &str, max_tokens: usize) -> String {
    let mut budget = ContextBudget::new(max_tokens);
    let mut out = String::new();

    // Search for relevant components
    let components = component_registry::search(query);
    if !components.is_empty() {
        out.push_str("# Relevant Components\n");
        let summaries = optimize_component_list(&components, &mut budget);
        for comp in &summaries {
            out.push_str(&comp.format_detailed());
            out.push('\n');
        }
    }

    // Search for relevant error codes
    let codes = error_codes::search(query);
    if !codes.is_empty() && budget.remaining() > 100 {
        out.push_str("\n# Relevant Error Codes\n");
        for code in &codes {
            let entry = format!("## {} — {}\n{}\n", code.code, code.message, code.suggestion);
            let tokens = estimate_tokens(&entry);
            if budget.spend(tokens) {
                out.push_str(&entry);
            }
        }
    }

    // Relevant prompt templates
    let templates = crate::ai::prompt_templates::all_templates();
    let relevant: Vec<_> = templates
        .iter()
        .filter(|t| {
            t.description.to_lowercase().contains(&query.to_lowercase())
                || t.id.contains(&query.to_lowercase())
        })
        .collect();
    if !relevant.is_empty() && budget.remaining() > 100 {
        out.push_str("\n# Relevant Prompt Templates\n");
        for t in &relevant {
            let entry = format!("- {}: {}\n", t.id, t.description);
            let tokens = estimate_tokens(&entry);
            if budget.spend(tokens) {
                out.push_str(&entry);
            }
        }
    }

    out.push_str(&format!(
        "\n# Context: {}/{} tokens\n",
        budget.used, budget.max_tokens
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_registry;

    fn setup_test_components() {
        component_registry::clear();
        component_registry::register(ComponentMeta {
            name: "Button".to_string(),
            props_type: "ButtonProps".to_string(),
            props: vec![
                crate::component_registry::PropInfo::required("label", "String", "Button text"),
                crate::component_registry::PropInfo::optional("disabled", "bool", "false", "Disabled state"),
            ],
            is_island: false,
            uses_suspense: false,
            description: "A clickable button".to_string(),
            category: "form".to_string(),
            tags: vec!["interactive".to_string()],
            example: "Button { label: \"OK\" }".to_string(),
        });
        component_registry::register(ComponentMeta {
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
    }

    #[test]
    fn test_estimate_tokens() {
        assert!(estimate_tokens("hello world") > 0);
    }

    #[test]
    fn test_context_budget() {
        let mut budget = ContextBudget::new(100);
        assert!(budget.has_room(50));
        assert!(budget.spend(50));
        assert_eq!(budget.used, 50);
        assert!(budget.has_room(50));
        assert!(budget.spend(50));
        assert!(!budget.has_room(1));
    }

    #[test]
    fn test_context_budget_default() {
        let budget = ContextBudget::default();
        assert_eq!(budget.max_tokens, 4000);
    }

    #[test]
    fn test_component_summary_compact() {
        setup_test_components();
        let comp = component_registry::find("Button").unwrap();
        let summary = ComponentSummary::from_meta(&comp);
        let compact = summary.format_compact();
        assert!(compact.contains("Button"));
        assert!(compact.contains("label"));
        assert!(compact.contains("String"));
    }

    #[test]
    fn test_optimize_components() {
        setup_test_components();
        let mut budget = ContextBudget::new(10000);
        let summaries = optimize_components(&mut budget);
        assert!(summaries.len() >= 2);
        assert!(budget.used > 0);
    }

    #[test]
    fn test_optimize_components_budget_limit() {
        setup_test_components();
        let mut budget = ContextBudget::new(1); // Very small budget
        let summaries = optimize_components(&mut budget);
        assert!(summaries.is_empty());
    }

    #[test]
    fn test_generate_context_package() {
        setup_test_components();
        let pkg = generate_context_package(8000);
        assert!(pkg.contains("Error Codes"));
        assert!(pkg.contains("Available Components"));
        assert!(pkg.contains("Prompt Templates"));
        assert!(pkg.contains("tokens"));
    }

    #[test]
    fn test_generate_focused_context() {
        setup_test_components();
        let ctx = generate_focused_context("button", 4000);
        assert!(ctx.contains("Relevant Components"));
        assert!(ctx.contains("Button"));
    }

    #[test]
    fn test_optimize_error_codes() {
        let mut budget = ContextBudget::new(10000);
        let codes = optimize_error_codes(&mut budget);
        assert!(!codes.is_empty());
        assert!(codes.iter().any(|(c, _)| c == "R001"));
    }
}
