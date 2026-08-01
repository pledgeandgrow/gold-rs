//! Goal 105: CSS-based reactive updates.
//!
//! For state-driven UI changes (show/hide, enable/disable, color themes),
//! set a single `data-state` attribute and let CSS handle the rest via
//! attribute selectors. This avoids multiple DOM calls per state change.
//!
//! Instead of:
//! ```ignore
//! // 3 DOM calls per state change
//! el.set_class("active", is_active);
//! el.set_style("display", if visible { "block" } else { "none" });
//! el.set_attribute("disabled", if !enabled { "true" } else { "false" });
//! ```
//!
//! Do:
//! ```ignore
//! // 1 DOM call — CSS handles the rest
//! el.set_attribute("data-state", &state_attr(is_active, visible, enabled));
//! ```

use std::collections::HashMap;

/// A CSS rule scoped to a `data-state` attribute value.
#[derive(Debug, Clone)]
pub struct CssStateRule {
    /// The `data-state` value this rule matches (e.g. "active-visible").
    pub state: String,
    /// CSS declarations (property, value pairs).
    pub declarations: Vec<(String, String)>,
}

/// A collection of CSS state rules for a component.
#[derive(Debug, Clone, Default)]
pub struct CssStateStylesheet {
    /// Rules keyed by state value.
    rules: Vec<CssStateRule>,
    /// The selector prefix (e.g. ".my-component").
    selector: String,
}

impl CssStateStylesheet {
    /// Create a new stylesheet for the given selector.
    pub fn new(selector: impl Into<String>) -> Self {
        Self {
            rules: Vec::new(),
            selector: selector.into(),
        }
    }

    /// Add a rule for a specific state value.
    pub fn rule(mut self, state: impl Into<String>, declarations: Vec<(&str, &str)>) -> Self {
        self.rules.push(CssStateRule {
            state: state.into(),
            declarations: declarations
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        });
        self
    }

    /// Generate the CSS text for this stylesheet.
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        for rule in &self.rules {
            css.push_str(&format!(
                "{}[data-state=\"{}\"] {{\n",
                self.selector, rule.state
            ));
            for (prop, value) in &rule.declarations {
                css.push_str(&format!("  {}: {};\n", prop, value));
            }
            css.push_str("}\n");
        }

        css
    }
}

/// Build a `data-state` attribute value from boolean flags.
///
/// Each flag contributes a segment to the state string. For example:
/// `state_attr(&[("active", true), ("visible", false), ("enabled", true)])`
/// → `"active--enabled"` (present flags included, absent flags skipped with `--` separator).
pub fn state_attr(flags: &[(&str, bool)]) -> String {
    let parts: Vec<&str> = flags
        .iter()
        .filter(|(_, v)| *v)
        .map(|(k, _)| *k)
        .collect();
    if parts.is_empty() {
        "default".to_string()
    } else {
        parts.join("-")
    }
}

/// A reactive state attribute manager.
///
/// Tracks which state flags are active and computes the `data-state`
/// attribute value. Only triggers a DOM update when the attribute
/// value actually changes.
pub struct StateAttribute {
    /// Current flag values.
    flags: HashMap<String, bool>,
    /// Cached attribute value (to avoid redundant DOM calls).
    cached_value: Option<String>,
}

impl StateAttribute {
    /// Create a new state attribute manager.
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
            cached_value: None,
        }
    }

    /// Set a flag value.
    ///
    /// Returns `Some(new_value)` if the `data-state` attribute changed
    /// (and DOM needs updating), or `None` if it stayed the same.
    pub fn set(&mut self, flag: impl Into<String>, value: bool) -> Option<String> {
        self.flags.insert(flag.into(), value);
        let new_value = self.compute();
        if self.cached_value.as_deref() != Some(new_value.as_str()) {
            self.cached_value = Some(new_value.clone());
            Some(new_value)
        } else {
            None
        }
    }

    /// Get a flag value.
    pub fn get(&self, flag: &str) -> bool {
        self.flags.get(flag).copied().unwrap_or(false)
    }

    /// Compute the current `data-state` attribute value.
    fn compute(&self) -> String {
        let active: Vec<&str> = self
            .flags
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.as_str())
            .collect();
        if active.is_empty() {
            "default".to_string()
        } else {
            active.join("-")
        }
    }

    /// Get the current attribute value (without recomputing).
    pub fn current(&self) -> &str {
        self.cached_value.as_deref().unwrap_or("default")
    }
}

impl Default for StateAttribute {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_attr() {
        let attr = state_attr(&[("active", true), ("visible", false), ("enabled", true)]);
        assert_eq!(attr, "active-enabled");
    }

    #[test]
    fn test_state_attr_empty() {
        let attr = state_attr(&[("active", false), ("visible", false)]);
        assert_eq!(attr, "default");
    }

    #[test]
    fn test_state_attr_all_active() {
        let attr = state_attr(&[("a", true), ("b", true), ("c", true)]);
        assert_eq!(attr, "a-b-c");
    }

    #[test]
    fn test_css_stylesheet() {
        let sheet = CssStateStylesheet::new(".btn")
            .rule("active", vec![("background", "blue"), ("color", "white")])
            .rule("disabled", vec![("opacity", "0.5"), ("cursor", "not-allowed")]);

        let css = sheet.to_css();
        assert!(css.contains(".btn[data-state=\"active\"]"));
        assert!(css.contains("background: blue;"));
        assert!(css.contains(".btn[data-state=\"disabled\"]"));
        assert!(css.contains("opacity: 0.5;"));
    }

    #[test]
    fn test_state_attribute_set() {
        let mut sa = StateAttribute::new();
        let change = sa.set("active", true);
        assert_eq!(change, Some("active".to_string()));
        assert_eq!(sa.current(), "active");
    }

    #[test]
    fn test_state_attribute_no_change() {
        let mut sa = StateAttribute::new();
        sa.set("active", true);
        // Setting the same value again should not trigger a change
        let change = sa.set("active", true);
        assert_eq!(change, None);
    }

    #[test]
    fn test_state_attribute_multiple_flags() {
        let mut sa = StateAttribute::new();
        sa.set("active", true);
        sa.set("visible", true);
        let current = sa.current();
        // Order depends on HashMap iteration, so check both parts
        assert!(current.contains("active"));
        assert!(current.contains("visible"));
        assert_eq!(current.len(), "active-visible".len());

        sa.set("active", false);
        assert_eq!(sa.current(), "visible");
    }

    #[test]
    fn test_state_attribute_all_false() {
        let mut sa = StateAttribute::new();
        sa.set("active", true);
        sa.set("active", false);
        assert_eq!(sa.current(), "default");
    }
}
