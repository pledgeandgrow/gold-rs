//! Hydration — attach reactive event handlers to server-rendered HTML.
//!
//! After SSR produces HTML with hydration markers, the client-side
//! runtime "hydrates" the DOM by attaching event listeners and
//! binding signals to existing nodes — without re-rendering.

use std::collections::HashMap;

/// A hydration marker embedded in SSR HTML.
///
/// Markers are HTML comments like `<!--rye-0-->` that map DOM nodes
/// to reactive scopes and event handlers.
#[derive(Debug, Clone)]
pub struct HydrationMarker {
    /// Unique hydration ID.
    pub id: usize,
    /// The type of node this marker represents.
    pub kind: HydrationKind,
}

/// The kind of node a hydration marker represents.
#[derive(Debug, Clone, PartialEq)]
pub enum HydrationKind {
    /// An element node.
    Element,
    /// A text node.
    Text,
    /// A component boundary.
    Component,
    /// A dynamic expression boundary.
    Dynamic,
}

/// A hydration plan — maps marker IDs to their reactive data.
#[derive(Debug, Default)]
pub struct HydrationPlan {
    /// Map of marker ID → element tag name.
    pub elements: HashMap<usize, String>,
    /// Map of marker ID → text content.
    pub texts: HashMap<usize, String>,
    /// Map of marker ID → event handlers (event name → handler ID).
    pub events: HashMap<usize, Vec<(String, usize)>>,
    /// Map of marker ID → child marker IDs.
    pub children: HashMap<usize, Vec<usize>>,
    /// The root marker ID.
    pub root: Option<usize>,
}

impl HydrationPlan {
    /// Create a new empty hydration plan.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an element in the plan.
    pub fn add_element(&mut self, id: usize, tag: impl Into<String>) {
        self.elements.insert(id, tag.into());
    }

    /// Register a text node in the plan.
    pub fn add_text(&mut self, id: usize, content: impl Into<String>) {
        self.texts.insert(id, content.into());
    }

    /// Register an event handler for an element.
    pub fn add_event(&mut self, id: usize, event: impl Into<String>, handler_id: usize) {
        self.events
            .entry(id)
            .or_default()
            .push((event.into(), handler_id));
    }

    /// Register a parent-child relationship.
    pub fn add_child(&mut self, parent: usize, child: usize) {
        self.children.entry(parent).or_default().push(child);
    }

    /// Set the root marker ID.
    pub fn set_root(&mut self, id: usize) {
        self.root = Some(id);
    }

    /// Get the element tag for a marker ID.
    pub fn get_element(&self, id: usize) -> Option<&str> {
        self.elements.get(&id).map(|s| s.as_str())
    }

    /// Get the text content for a marker ID.
    pub fn get_text(&self, id: usize) -> Option<&str> {
        self.texts.get(&id).map(|s| s.as_str())
    }

    /// Get the events for a marker ID.
    pub fn get_events(&self, id: usize) -> Option<&[(String, usize)]> {
        self.events.get(&id).map(|v| v.as_slice())
    }

    /// Get the children for a marker ID.
    pub fn get_children(&self, id: usize) -> Option<&[usize]> {
        self.children.get(&id).map(|v| v.as_slice())
    }
}

/// Parse hydration markers from SSR HTML.
///
/// Returns a list of hydration markers found in the HTML.
/// Markers are HTML comments in the format `<!--rye-{id}-{kind}-->`.
pub fn parse_markers(html: &str) -> Vec<HydrationMarker> {
    let mut markers = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find("<!--rye-") {
        let start = pos + start;
        let content_start = start + "<!--rye-".len();

        if let Some(end) = html[content_start..].find("-->") {
            let content = &html[content_start..content_start + end];
            if let Some(marker) = parse_marker_content(content) {
                markers.push(marker);
            }
            pos = content_start + end + 3;
        } else {
            break;
        }
    }

    markers
}

/// Parse a single marker content string (e.g. "0-e", "1-t", "2-c").
fn parse_marker_content(content: &str) -> Option<HydrationMarker> {
    let parts: Vec<&str> = content.splitn(2, '-').collect();
    if parts.len() != 2 {
        return None;
    }

    let id: usize = parts[0].parse().ok()?;
    let kind = match parts[1] {
        "e" => HydrationKind::Element,
        "t" => HydrationKind::Text,
        "c" => HydrationKind::Component,
        "d" => HydrationKind::Dynamic,
        _ => return None,
    };

    Some(HydrationMarker { id, kind })
}

/// Generate a hydration marker comment for the given ID and kind.
pub fn generate_marker(id: usize, kind: &HydrationKind) -> String {
    let kind_str = match kind {
        HydrationKind::Element => "e",
        HydrationKind::Text => "t",
        HydrationKind::Component => "c",
        HydrationKind::Dynamic => "d",
    };
    format!("<!--rye-{}-{}-->", id, kind_str)
}

/// The result of hydration.
#[derive(Debug, Default)]
pub struct HydrationResult {
    /// Number of elements hydrated.
    pub elements_hydrated: usize,
    /// Number of text nodes hydrated.
    pub texts_hydrated: usize,
    /// Number of event handlers attached.
    pub events_attached: usize,
    /// Any errors encountered during hydration.
    pub errors: Vec<String>,
}

impl HydrationResult {
    /// Check if hydration was successful (no errors).
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if hydration had errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Hydrate a hydration plan against a DOM-like structure.
///
/// This is a framework-agnostic function that walks the hydration plan
/// and verifies that the server-rendered structure matches.
/// In a real app, this would attach event listeners to DOM nodes.
pub fn hydrate(plan: &HydrationPlan) -> HydrationResult {
    let mut result = HydrationResult::default();

    let root = match plan.root {
        Some(root) => root,
        None => {
            result
                .errors
                .push("No root marker in hydration plan".to_string());
            return result;
        }
    };

    hydrate_node(plan, root, &mut result);

    result
}

/// Recursively hydrate a node and its children.
fn hydrate_node(plan: &HydrationPlan, id: usize, result: &mut HydrationResult) {
    // Check if this is an element or text
    if plan.elements.contains_key(&id) {
        result.elements_hydrated += 1;

        // Attach event handlers
        if let Some(events) = plan.get_events(id) {
            for (_event, _handler_id) in events {
                result.events_attached += 1;
            }
        }
    } else if plan.texts.contains_key(&id) {
        result.texts_hydrated += 1;
    } else {
        result.errors.push(format!("Unknown node ID: {}", id));
    }

    // Hydrate children
    if let Some(children) = plan.get_children(id) {
        for child in children {
            hydrate_node(plan, *child, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markers() {
        let html = r#"<div><!--rye-0-e--><span><!--rye-1-e-->Hello<!--rye-2-t--></span></div>"#;
        let markers = parse_markers(html);

        assert_eq!(markers.len(), 3);
        assert_eq!(markers[0].id, 0);
        assert_eq!(markers[0].kind, HydrationKind::Element);
        assert_eq!(markers[1].id, 1);
        assert_eq!(markers[1].kind, HydrationKind::Element);
        assert_eq!(markers[2].id, 2);
        assert_eq!(markers[2].kind, HydrationKind::Text);
    }

    #[test]
    fn test_generate_marker() {
        let marker = generate_marker(42, &HydrationKind::Element);
        assert_eq!(marker, "<!--rye-42-e-->");

        let marker = generate_marker(0, &HydrationKind::Text);
        assert_eq!(marker, "<!--rye-0-t-->");
    }

    #[test]
    fn test_hydrate_simple() {
        let mut plan = HydrationPlan::new();
        plan.set_root(0);
        plan.add_element(0, "div");
        plan.add_event(0, "click", 100);
        plan.add_child(0, 1);
        plan.add_text(1, "Hello");

        let result = hydrate(&plan);

        assert!(result.is_ok());
        assert_eq!(result.elements_hydrated, 1);
        assert_eq!(result.texts_hydrated, 1);
        assert_eq!(result.events_attached, 1);
    }

    #[test]
    fn test_hydrate_no_root() {
        let plan = HydrationPlan::new();
        let result = hydrate(&plan);

        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_hydrate_nested() {
        let mut plan = HydrationPlan::new();
        plan.set_root(0);
        plan.add_element(0, "div");
        plan.add_child(0, 1);
        plan.add_child(0, 2);
        plan.add_element(1, "span");
        plan.add_child(1, 3);
        plan.add_text(3, "Hello");
        plan.add_element(2, "button");
        plan.add_event(2, "click", 200);

        let result = hydrate(&plan);

        assert!(result.is_ok());
        assert_eq!(result.elements_hydrated, 3);
        assert_eq!(result.texts_hydrated, 1);
        assert_eq!(result.events_attached, 1);
    }
}
