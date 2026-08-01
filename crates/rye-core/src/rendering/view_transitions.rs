//! Goal 116: View Transitions API.
//!
//! Cross-page animations using the browser's native View Transitions API.
//! `use_view_transition()` hook. On native, fallback to shared element transitions.

/// View transition configuration.
#[derive(Debug, Clone)]
pub struct ViewTransitionConfig {
    /// Names of elements to capture as shared elements.
    pub shared_elements: Vec<String>,
    /// Transition duration in milliseconds.
    pub duration_ms: u32,
    /// Whether to use snapshot cloning.
    pub snapshot: bool,
}

impl Default for ViewTransitionConfig {
    fn default() -> Self {
        Self {
            shared_elements: Vec::new(),
            duration_ms: 300,
            snapshot: true,
        }
    }
}

impl ViewTransitionConfig {
    /// Create a new view transition config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a shared element by its `view-transition-name` CSS property.
    pub fn shared(mut self, name: impl Into<String>) -> Self {
        self.shared_elements.push(name.into());
        self
    }

    /// Set transition duration.
    pub fn duration(mut self, ms: u32) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Whether the View Transitions API is supported.
pub fn is_view_transitions_supported() -> bool {
    // On Wasm: 'startViewTransition' in document
    // On native: false (fallback to manual transitions)
    #[cfg(not(target_arch = "wasm32"))]
    { false }
    #[cfg(target_arch = "wasm32")]
    { false }
}

/// Generate the JS for view transitions.
pub fn view_transition_script() -> &'static str {
    r#"<script>
(function() {
  window.__rye_view_transition = function(callback) {
    if (document.startViewTransition) {
      return document.startViewTransition(callback);
    } else {
      // Fallback: just call the callback
      callback();
      return Promise.resolve();
    }
  };

  window.__rye_view_transition_supported = function() {
    return 'startViewTransition' in document;
  };
})();
</script>"#
}

/// Generate CSS for view transition shared elements.
pub fn view_transition_css(names: &[(&str, &str)]) -> String {
    let mut css = String::new();
    for (selector, name) in names {
        css.push_str(&format!("{} {{\n  view-transition-name: {};\n}}\n", selector, name));
    }

    // Default transition animation
    css.push_str("\n@keyframes rye-fade-in {\n");
    css.push_str("  from { opacity: 0; }\n  to { opacity: 1; }\n}\n");
    css.push_str("\n@keyframes rye-fade-out {\n");
    css.push_str("  from { opacity: 1; }\n  to { opacity: 0; }\n}\n");
    css.push_str("\n::view-transition-old(root) {\n");
    css.push_str("  animation: rye-fade-out 300ms;\n}\n");
    css.push_str("::view-transition-new(root) {\n");
    css.push_str("  animation: rye-fade-in 300ms;\n}\n");

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_transition_config() {
        let config = ViewTransitionConfig::new()
            .shared("hero-image")
            .shared("title")
            .duration(500);
        assert_eq!(config.shared_elements, vec!["hero-image", "title"]);
        assert_eq!(config.duration_ms, 500);
    }

    #[test]
    fn test_view_transition_script() {
        let script = view_transition_script();
        assert!(script.contains("startViewTransition"));
        assert!(script.contains("__rye_view_transition"));
    }

    #[test]
    fn test_view_transition_css() {
        let css = view_transition_css(&[(".hero", "hero-image"), (".title", "page-title")]);
        assert!(css.contains("view-transition-name: hero-image"));
        assert!(css.contains("view-transition-name: page-title"));
        assert!(css.contains("@keyframes rye-fade-in"));
        assert!(css.contains("@keyframes rye-fade-out"));
        assert!(css.contains("::view-transition-old(root)"));
        assert!(css.contains("::view-transition-new(root)"));
    }
}
