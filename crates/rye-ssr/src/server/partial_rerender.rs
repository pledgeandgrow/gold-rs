//! Goal 190: Partial SSR re-rendering.
//!
//! When server-side state changes (via WebSocket/SSE), re-render only the
//! affected component subtree on the server and stream the diff to the client.
//! No full page reload.

use std::collections::HashMap;

/// A subtree diff — the result of re-rendering a component subtree.
#[derive(Debug, Clone, PartialEq)]
pub struct SubtreeDiff {
    /// The component ID that was re-rendered.
    pub component_id: String,
    /// The new HTML content.
    pub new_html: String,
    /// The old HTML content (for diffing).
    pub old_html: String,
    /// Whether the content actually changed.
    pub changed: bool,
}

impl SubtreeDiff {
    /// Create a new subtree diff.
    pub fn new(component_id: &str, old_html: &str, new_html: &str) -> Self {
        let changed = old_html != new_html;
        Self {
            component_id: component_id.to_string(),
            new_html: new_html.to_string(),
            old_html: old_html.to_string(),
            changed,
        }
    }

    /// Get the diff as a JSON patch instruction for the client.
    pub fn to_patch_json(&self) -> String {
        format!(
            r#"{{"type":"replace","componentId":"{}","html":"{}"}}"#,
            self.component_id,
            escape_json(&self.new_html),
        )
    }

    /// Get the JavaScript to apply this patch on the client.
    pub fn to_patch_script(&self) -> String {
        format!(
            r#"(function(){{var el=document.querySelector('[data-rye-component="{}"]');if(el)el.outerHTML='{}';}})();"#,
            self.component_id,
            escape_json(&self.new_html),
        )
    }
}

/// The partial re-renderer — tracks rendered subtrees and re-renders them.
pub struct PartialRenderer {
    subtrees: HashMap<String, String>, // component_id -> last rendered HTML
}

impl PartialRenderer {
    /// Create a new partial renderer.
    pub fn new() -> Self {
        Self {
            subtrees: HashMap::new(),
        }
    }

    /// Register a rendered subtree.
    pub fn register(&mut self, component_id: &str, html: &str) {
        self.subtrees.insert(component_id.to_string(), html.to_string());
    }

    /// Re-render a subtree and produce a diff.
    pub fn rerender<F: FnOnce() -> String>(&mut self, component_id: &str, render_fn: F) -> SubtreeDiff {
        let old_html = self.subtrees.get(component_id).cloned().unwrap_or_default();
        let new_html = render_fn();
        let diff = SubtreeDiff::new(component_id, &old_html, &new_html);
        if diff.changed {
            self.subtrees.insert(component_id.to_string(), new_html);
        }
        diff
    }

    /// Re-render multiple subtrees and return all diffs.
    pub fn rerender_batch<F: Fn(&str) -> String>(&mut self, component_ids: &[&str], render_fn: F) -> Vec<SubtreeDiff> {
        component_ids
            .iter()
            .map(|id| self.rerender(id, || render_fn(id)))
            .collect()
    }

    /// Get the last rendered HTML for a component.
    pub fn get_html(&self, component_id: &str) -> Option<&str> {
        self.subtrees.get(component_id).map(|s| s.as_str())
    }

    /// Check if a component is registered.
    pub fn has(&self, component_id: &str) -> bool {
        self.subtrees.contains_key(component_id)
    }

    /// Get the number of registered subtrees.
    pub fn len(&self) -> usize {
        self.subtrees.len()
    }

    /// Check if the renderer is empty.
    pub fn is_empty(&self) -> bool {
        self.subtrees.is_empty()
    }

    /// Remove a subtree.
    pub fn remove(&mut self, component_id: &str) {
        self.subtrees.remove(component_id);
    }

    /// Clear all subtrees.
    pub fn clear(&mut self) {
        self.subtrees.clear();
    }

    /// Get all registered component IDs.
    pub fn component_ids(&self) -> Vec<String> {
        self.subtrees.keys().cloned().collect()
    }

    /// Generate a batch patch script for multiple diffs.
    pub fn batch_patch_script(diffs: &[SubtreeDiff]) -> String {
        let scripts: Vec<String> = diffs.iter().filter(|d| d.changed).map(|d| d.to_patch_script()).collect();
        if scripts.is_empty() {
            return String::new();
        }
        format!("(function(){{{}}})();", scripts.join(""))
    }

    /// Generate a batch patch JSON for multiple diffs.
    pub fn batch_patch_json(diffs: &[SubtreeDiff]) -> String {
        let patches: Vec<String> = diffs.iter().filter(|d| d.changed).map(|d| d.to_patch_json()).collect();
        if patches.is_empty() {
            return "[]".to_string();
        }
        format!("[{}]", patches.join(","))
    }
}

impl Default for PartialRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtree_diff_changed() {
        let diff = SubtreeDiff::new("counter", "<span>0</span>", "<span>1</span>");
        assert!(diff.changed);
    }

    #[test]
    fn test_subtree_diff_unchanged() {
        let diff = SubtreeDiff::new("counter", "<span>0</span>", "<span>0</span>");
        assert!(!diff.changed);
    }

    #[test]
    fn test_subtree_diff_to_patch_json() {
        let diff = SubtreeDiff::new("counter", "old", "<span>1</span>");
        let json = diff.to_patch_json();
        assert!(json.contains("\"type\":\"replace\""));
        assert!(json.contains("\"componentId\":\"counter\""));
        assert!(json.contains("<span>1</span>"));
    }

    #[test]
    fn test_subtree_diff_to_patch_script() {
        let diff = SubtreeDiff::new("header", "old", "<h1>New</h1>");
        let script = diff.to_patch_script();
        assert!(script.contains("data-rye-component"));
        assert!(script.contains("header"));
    }

    #[test]
    fn test_partial_renderer_register_rerender() {
        let mut renderer = PartialRenderer::new();
        renderer.register("counter", "<span>0</span>");
        assert!(renderer.has("counter"));

        let diff = renderer.rerender("counter", || "<span>1</span>".to_string());
        assert!(diff.changed);
        assert_eq!(renderer.get_html("counter"), Some("<span>1</span>"));
    }

    #[test]
    fn test_partial_renderer_rerender_no_change() {
        let mut renderer = PartialRenderer::new();
        renderer.register("static", "<p>Hello</p>");

        let diff = renderer.rerender("static", || "<p>Hello</p>".to_string());
        assert!(!diff.changed);
    }

    #[test]
    fn test_partial_renderer_rerender_unregistered() {
        let mut renderer = PartialRenderer::new();
        let diff = renderer.rerender("new-component", || "<div>New</div>".to_string());
        assert!(diff.changed);
        assert_eq!(diff.old_html, "");
    }

    #[test]
    fn test_partial_renderer_rerender_batch() {
        let mut renderer = PartialRenderer::new();
        renderer.register("a", "1");
        renderer.register("b", "2");
        renderer.register("c", "3");

        let diffs = renderer.rerender_batch(&["a", "b", "c"], |id| {
            format!("<span>updated-{}</span>", id)
        });

        assert_eq!(diffs.len(), 3);
        assert!(diffs.iter().all(|d| d.changed));
    }

    #[test]
    fn test_partial_renderer_remove() {
        let mut renderer = PartialRenderer::new();
        renderer.register("temp", "html");
        renderer.remove("temp");
        assert!(!renderer.has("temp"));
    }

    #[test]
    fn test_partial_renderer_clear() {
        let mut renderer = PartialRenderer::new();
        renderer.register("a", "1");
        renderer.register("b", "2");
        renderer.clear();
        assert!(renderer.is_empty());
    }

    #[test]
    fn test_partial_renderer_component_ids() {
        let mut renderer = PartialRenderer::new();
        renderer.register("x", "1");
        renderer.register("y", "2");
        let ids = renderer.component_ids();
        assert!(ids.contains(&"x".to_string()));
        assert!(ids.contains(&"y".to_string()));
    }

    #[test]
    fn test_batch_patch_script() {
        let diffs = vec![
            SubtreeDiff::new("a", "old", "<span>A</span>"),
            SubtreeDiff::new("b", "same", "same"), // unchanged
            SubtreeDiff::new("c", "old", "<span>C</span>"),
        ];
        let script = PartialRenderer::batch_patch_script(&diffs);
        assert!(script.contains("data-rye-component=\"a\""));
        assert!(script.contains("data-rye-component=\"c\""));
        assert!(!script.contains("data-rye-component=\"b\""));
    }

    #[test]
    fn test_batch_patch_json() {
        let diffs = vec![
            SubtreeDiff::new("a", "old", "<span>A</span>"),
            SubtreeDiff::new("b", "same", "same"),
        ];
        let json = PartialRenderer::batch_patch_json(&diffs);
        assert!(json.starts_with("["));
        assert!(json.contains("\"componentId\":\"a\""));
        assert!(!json.contains("\"componentId\":\"b\""));
    }

    #[test]
    fn test_batch_patch_json_empty() {
        let diffs: Vec<SubtreeDiff> = vec![];
        let json = PartialRenderer::batch_patch_json(&diffs);
        assert_eq!(json, "[]");
    }
}
