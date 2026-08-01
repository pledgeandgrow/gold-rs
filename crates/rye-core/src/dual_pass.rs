//! Dual-pass rendering — skeleton first, then fill in dynamic content.
//!
//! First pass renders a skeleton with placeholders, second pass fills in
//! dynamic content. Enables faster first paint for complex layouts.
//! Works with existing streaming SSR to send skeleton immediately.

use std::cell::RefCell;
use std::collections::HashMap;

/// A placeholder in the skeleton — to be filled in the second pass.
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonPlaceholder {
    /// Unique placeholder ID.
    pub id: String,
    /// The placeholder HTML (e.g. a loading spinner or empty div).
    pub skeleton_html: String,
    /// Whether this placeholder has been filled.
    pub filled: bool,
}

/// The result of a dual-pass render.
#[derive(Debug, Clone)]
pub struct DualPassResult {
    /// The skeleton HTML (first pass).
    pub skeleton: String,
    /// The filled HTML (second pass), if completed.
    pub filled: Option<String>,
    /// Placeholders that were filled.
    pub placeholders: Vec<SkeletonPlaceholder>,
}

/// The dual-pass renderer — renders a skeleton first, then fills in content.
pub struct DualPassRenderer {
    placeholders: RefCell<HashMap<String, SkeletonPlaceholder>>,
    skeleton: RefCell<String>,
}

impl DualPassRenderer {
    /// Create a new dual-pass renderer.
    pub fn new() -> Self {
        Self {
            placeholders: RefCell::new(HashMap::new()),
            skeleton: RefCell::new(String::new()),
        }
    }

    /// First pass — render the skeleton with placeholders.
    pub fn render_skeleton<F: FnOnce(&mut SkeletonBuilder)>(&self, build: F) -> String {
        let mut builder = SkeletonBuilder::new();
        build(&mut builder);

        let html = builder.html.clone();
        *self.skeleton.borrow_mut() = html.clone();

        for placeholder in &builder.placeholders {
            self.placeholders.borrow_mut().insert(
                placeholder.id.clone(),
                SkeletonPlaceholder {
                    id: placeholder.id.clone(),
                    skeleton_html: placeholder.skeleton_html.clone(),
                    filled: false,
                },
            );
        }

        html
    }

    /// Second pass — fill a placeholder with content.
    pub fn fill_placeholder(&self, id: &str, content: &str) -> bool {
        let mut placeholders = self.placeholders.borrow_mut();
        if let Some(placeholder) = placeholders.get_mut(id) {
            placeholder.filled = true;

            // Replace the placeholder in the skeleton
            let mut skeleton = self.skeleton.borrow_mut();
            let marker = format!("<!--rye-placeholder:{}-->", id);
            *skeleton = skeleton.replace(&marker, content);

            return true;
        }
        false
    }

    /// Fill all remaining placeholders with their fallback content.
    pub fn fill_remaining(&self) {
        let ids: Vec<String> = self
            .placeholders
            .borrow()
            .iter()
            .filter(|(_, p)| !p.filled)
            .map(|(id, _)| id.clone())
            .collect();

        for id in ids {
            let fallback = self
                .placeholders
                .borrow()
                .get(&id)
                .map(|p| p.skeleton_html.clone())
                .unwrap_or_default();
            self.fill_placeholder(&id, &fallback);
        }
    }

    /// Get the current HTML (skeleton with any filled placeholders).
    pub fn current_html(&self) -> String {
        self.skeleton.borrow().clone()
    }

    /// Get the final HTML — fills remaining placeholders and returns the result.
    pub fn finalize(&self) -> String {
        self.fill_remaining();
        self.skeleton.borrow().clone()
    }

    /// Check if all placeholders have been filled.
    pub fn is_complete(&self) -> bool {
        self.placeholders.borrow().values().all(|p| p.filled)
    }

    /// Get the number of placeholders.
    pub fn placeholder_count(&self) -> usize {
        self.placeholders.borrow().len()
    }

    /// Get the number of filled placeholders.
    pub fn filled_count(&self) -> usize {
        self.placeholders.borrow().values().filter(|p| p.filled).count()
    }

    /// Get the number of unfilled placeholders.
    pub fn unfilled_count(&self) -> usize {
        self.placeholders.borrow().values().filter(|p| !p.filled).count()
    }

    /// Get all placeholder IDs.
    pub fn placeholder_ids(&self) -> Vec<String> {
        self.placeholders.borrow().keys().cloned().collect()
    }

    /// Get unfilled placeholder IDs.
    pub fn unfilled_ids(&self) -> Vec<String> {
        self.placeholders
            .borrow()
            .iter()
            .filter(|(_, p)| !p.filled)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Clear all state.
    pub fn clear(&self) {
        self.placeholders.borrow_mut().clear();
        self.skeleton.borrow_mut().clear();
    }
}

impl Default for DualPassRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for the skeleton pass — collects HTML and placeholders.
pub struct SkeletonBuilder {
    html: String,
    placeholders: Vec<SkeletonPlaceholder>,
}

impl SkeletonBuilder {
    /// Create a new skeleton builder.
    pub fn new() -> Self {
        Self {
            html: String::new(),
            placeholders: Vec::new(),
        }
    }

    /// Append raw HTML to the skeleton.
    pub fn html(&mut self, html: &str) -> &mut Self {
        self.html.push_str(html);
        self
    }

    /// Add a placeholder to the skeleton.
    pub fn placeholder(&mut self, id: &str, fallback_html: &str) -> &mut Self {
        self.html.push_str(&format!("<!--rye-placeholder:{}-->", id));
        self.placeholders.push(SkeletonPlaceholder {
            id: id.to_string(),
            skeleton_html: fallback_html.to_string(),
            filled: false,
        });
        self
    }

    /// Add a loading placeholder (spinner or skeleton box).
    pub fn loading(&mut self, id: &str) -> &mut Self {
        self.placeholder(id, "<div class=\"rye-loading\">Loading...</div>")
    }
}

impl Default for SkeletonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_builder_basic() {
        let renderer = DualPassRenderer::new();
        let skeleton = renderer.render_skeleton(|builder| {
            builder.html("<div>").html("Hello").html("</div>");
        });
        assert_eq!(skeleton, "<div>Hello</div>");
    }

    #[test]
    fn test_skeleton_with_placeholder() {
        let renderer = DualPassRenderer::new();
        let skeleton = renderer.render_skeleton(|builder| {
            builder.html("<div>");
            builder.placeholder("content", "<p>Loading...</p>");
            builder.html("</div>");
        });
        assert!(skeleton.contains("<!--rye-placeholder:content-->"));
        assert_eq!(renderer.placeholder_count(), 1);
    }

    #[test]
    fn test_fill_placeholder() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.html("<div>");
            builder.placeholder("content", "Loading");
            builder.html("</div>");
        });

        assert!(!renderer.is_complete());
        assert!(renderer.fill_placeholder("content", "<p>Real content</p>"));
        assert!(renderer.is_complete());
        assert!(renderer.current_html().contains("<p>Real content</p>"));
        assert!(!renderer.current_html().contains("rye-placeholder"));
    }

    #[test]
    fn test_fill_nonexistent_placeholder() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.html("hello");
        });
        assert!(!renderer.fill_placeholder("nonexistent", "content"));
    }

    #[test]
    fn test_fill_remaining() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.placeholder("a", "fallback-a");
            builder.placeholder("b", "fallback-b");
        });
        assert_eq!(renderer.unfilled_count(), 2);
        renderer.fill_remaining();
        assert_eq!(renderer.filled_count(), 2);
        assert!(renderer.is_complete());
    }

    #[test]
    fn test_finalize() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.html("<ul>");
            builder.placeholder("item1", "<li>Loading</li>");
            builder.html("</ul>");
        });
        renderer.fill_placeholder("item1", "<li>Item 1</li>");
        let final_html = renderer.finalize();
        assert!(final_html.contains("<li>Item 1</li>"));
    }

    #[test]
    fn test_loading_placeholder() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.loading("data");
        });
        assert_eq!(renderer.placeholder_count(), 1);
        renderer.fill_remaining();
        assert!(renderer.current_html().contains("rye-loading"));
    }

    #[test]
    fn test_placeholder_counts() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.placeholder("a", "a");
            builder.placeholder("b", "b");
            builder.placeholder("c", "c");
        });
        assert_eq!(renderer.placeholder_count(), 3);
        assert_eq!(renderer.filled_count(), 0);
        assert_eq!(renderer.unfilled_count(), 3);
        renderer.fill_placeholder("a", "filled-a");
        assert_eq!(renderer.filled_count(), 1);
        assert_eq!(renderer.unfilled_count(), 2);
    }

    #[test]
    fn test_placeholder_ids() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.placeholder("x", "x");
            builder.placeholder("y", "y");
        });
        let ids = renderer.placeholder_ids();
        assert!(ids.contains(&"x".to_string()));
        assert!(ids.contains(&"y".to_string()));
    }

    #[test]
    fn test_unfilled_ids() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.placeholder("x", "x");
            builder.placeholder("y", "y");
        });
        renderer.fill_placeholder("x", "filled");
        let unfilled = renderer.unfilled_ids();
        assert_eq!(unfilled, vec!["y".to_string()]);
    }

    #[test]
    fn test_clear() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.placeholder("a", "a");
        });
        renderer.clear();
        assert_eq!(renderer.placeholder_count(), 0);
        assert!(renderer.current_html().is_empty());
    }

    #[test]
    fn test_dual_pass_result() {
        let renderer = DualPassRenderer::new();
        renderer.render_skeleton(|builder| {
            builder.html("<div>");
            builder.placeholder("content", "Loading");
            builder.html("</div>");
        });
        renderer.fill_placeholder("content", "Hello");
        let html = renderer.finalize();
        assert!(html.contains("Hello"));
        assert!(!html.contains("rye-placeholder"));
    }
}
