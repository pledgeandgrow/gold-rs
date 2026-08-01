//! Static template extraction — extract fully static subtrees at compile time.
//!
//! At compile time, extract fully static template subtrees (no dynamic bindings)
//! into pre-rendered HTML strings. Skip the render pipeline entirely for these
//! subtrees. Reduces Wasm execution time for content-heavy pages.

use std::cell::RefCell;
use std::collections::HashMap;

/// A static template — pre-rendered HTML with no dynamic bindings.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticTemplate {
    /// Unique identifier for this template.
    pub id: String,
    /// The pre-rendered HTML string.
    pub html: String,
    /// Whether this template has been verified as fully static.
    pub verified: bool,
    /// The byte size of the HTML.
    pub byte_size: usize,
}

impl StaticTemplate {
    /// Create a new static template.
    pub fn new(id: &str, html: &str) -> Self {
        Self {
            id: id.to_string(),
            html: html.to_string(),
            verified: true,
            byte_size: html.len(),
        }
    }

    /// Get the HTML content.
    pub fn html(&self) -> &str {
        &self.html
    }

    /// Get the byte size.
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }
}

/// The static template registry — stores extracted static templates.
pub struct StaticTemplateRegistry {
    templates: RefCell<HashMap<String, StaticTemplate>>,
}

impl StaticTemplateRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            templates: RefCell::new(HashMap::new()),
        }
    }

    /// Register a static template.
    pub fn register(&self, template: StaticTemplate) {
        self.templates.borrow_mut().insert(template.id.clone(), template);
    }

    /// Get a static template by ID.
    pub fn get(&self, id: &str) -> Option<StaticTemplate> {
        self.templates.borrow().get(id).cloned()
    }

    /// Render a static template by ID.
    pub fn render(&self, id: &str) -> Option<String> {
        self.templates.borrow().get(id).map(|t| t.html.clone())
    }

    /// Check if a template is registered.
    pub fn has(&self, id: &str) -> bool {
        self.templates.borrow().contains_key(id)
    }

    /// Get the number of registered templates.
    pub fn len(&self) -> usize {
        self.templates.borrow().len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.templates.borrow().is_empty()
    }

    /// Get total byte size of all templates.
    pub fn total_byte_size(&self) -> usize {
        self.templates.borrow().values().map(|t| t.byte_size).sum()
    }

    /// Get all template IDs.
    pub fn ids(&self) -> Vec<String> {
        self.templates.borrow().keys().cloned().collect()
    }

    /// Clear all templates.
    pub fn clear(&self) {
        self.templates.borrow_mut().clear();
    }
}

impl Default for StaticTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a template node is fully static (no dynamic content).
pub fn is_static_node(node: &crate::template::TemplateNode) -> bool {
    match node {
        crate::template::TemplateNode::Text(_) => true,
        crate::template::TemplateNode::Dynamic(_) => false,
        crate::template::TemplateNode::Element { children, .. } => {
            children.iter().all(|child| {
                child.nodes.iter().all(is_static_node)
            })
        }
    }
}

/// Extract static HTML from a template.
/// Returns None if the template contains any dynamic content.
pub fn extract_static_html(template: &crate::template::Template) -> Option<String> {
    let mut html = String::new();
    for node in &template.nodes {
        if !is_static_node(node) {
            return None;
        }
        render_static_node(node, &mut html);
    }
    Some(html)
}

/// Render a static template node to HTML.
fn render_static_node(node: &crate::template::TemplateNode, out: &mut String) {
    match node {
        crate::template::TemplateNode::Text(text) => {
            out.push_str(text);
        }
        crate::template::TemplateNode::Dynamic(_) => {
            // Should not reach here if is_static_node was checked
        }
        crate::template::TemplateNode::Element { tag, attrs, children, .. } => {
            out.push('<');
            out.push_str(tag);
            for (name, value) in attrs {
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                out.push_str(value);
                out.push('"');
            }
            out.push('>');
            for child in children {
                for child_node in &child.nodes {
                    render_static_node(child_node, out);
                }
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
    }
}

/// Analysis result for a template.
#[derive(Debug, Clone)]
pub struct TemplateAnalysis {
    /// Whether the template is fully static.
    pub is_static: bool,
    /// The number of static nodes.
    pub static_node_count: usize,
    /// The number of dynamic nodes.
    pub dynamic_node_count: usize,
    /// The estimated byte size if extracted as static HTML.
    pub estimated_byte_size: usize,
    /// The depth of the template tree.
    pub depth: usize,
}

/// Analyze a template to determine if it can be statically extracted.
pub fn analyze_template(template: &crate::template::Template) -> TemplateAnalysis {
    let mut static_count = 0;
    let mut dynamic_count = 0;
    let mut max_depth = 0;

    for node in &template.nodes {
        let (s, d, depth) = analyze_node(node, 1);
        static_count += s;
        dynamic_count += d;
        max_depth = max_depth.max(depth);
    }

    let is_static = dynamic_count == 0;
    let estimated_byte_size = if is_static {
        extract_static_html(template).map(|h| h.len()).unwrap_or(0)
    } else {
        0
    };

    TemplateAnalysis {
        is_static,
        static_node_count: static_count,
        dynamic_node_count: dynamic_count,
        estimated_byte_size,
        depth: max_depth,
    }
}

fn analyze_node(node: &crate::template::TemplateNode, current_depth: usize) -> (usize, usize, usize) {
    match node {
        crate::template::TemplateNode::Text(_) => (1, 0, current_depth),
        crate::template::TemplateNode::Dynamic(_) => (0, 1, current_depth),
        crate::template::TemplateNode::Element { children, .. } => {
            let mut static_count = 1;
            let mut dynamic_count = 0;
            let mut max_depth = current_depth;

            for child in children {
                for child_node in &child.nodes {
                    let (s, d, depth) = analyze_node(child_node, current_depth + 1);
                    static_count += s;
                    dynamic_count += d;
                    max_depth = max_depth.max(depth);
                }
            }

            (static_count, dynamic_count, max_depth)
        }
    }
}

// Global static template registry.
thread_local! {
    static GLOBAL_REGISTRY: RefCell<Option<StaticTemplateRegistry>> = const { RefCell::new(None) };
}

/// Initialize the global static template registry.
pub fn init_global_registry() {
    GLOBAL_REGISTRY.with(|r| {
        *r.borrow_mut() = Some(StaticTemplateRegistry::new());
    });
}

/// Get the global static template registry.
pub fn global_registry() -> Option<&'static StaticTemplateRegistry> {
    // SAFETY: thread_local storage is per-thread, returning a reference is unsafe
    // Instead, we return a clone of the registry contents
    None // Use global_registry_clone() instead
}

/// Get a clone of the global static template registry.
pub fn global_registry_clone() -> Option<StaticTemplateRegistry> {
    GLOBAL_REGISTRY.with(|r| {
        r.borrow().as_ref().map(|reg| StaticTemplateRegistry {
            templates: RefCell::new(reg.templates.borrow().clone()),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{Template, TemplateNode};

    #[test]
    fn test_static_template_basic() {
        let template = StaticTemplate::new("header", "<header><h1>Title</h1></header>");
        assert_eq!(template.html(), "<header><h1>Title</h1></header>");
        assert_eq!(template.byte_size, 31);
        assert!(template.verified);
    }

    #[test]
    fn test_registry_register_get() {
        let registry = StaticTemplateRegistry::new();
        registry.register(StaticTemplate::new("nav", "<nav>Home</nav>"));
        assert!(registry.has("nav"));
        assert!(!registry.has("footer"));
        let template = registry.get("nav").unwrap();
        assert_eq!(template.html(), "<nav>Home</nav>");
    }

    #[test]
    fn test_registry_render() {
        let registry = StaticTemplateRegistry::new();
        registry.register(StaticTemplate::new("footer", "<footer>Copyright</footer>"));
        assert_eq!(registry.render("footer"), Some("<footer>Copyright</footer>".to_string()));
        assert_eq!(registry.render("nonexistent"), None);
    }

    #[test]
    fn test_registry_total_byte_size() {
        let registry = StaticTemplateRegistry::new();
        registry.register(StaticTemplate::new("a", "hello"));
        registry.register(StaticTemplate::new("b", "world"));
        assert_eq!(registry.total_byte_size(), 10);
    }

    #[test]
    fn test_registry_ids() {
        let registry = StaticTemplateRegistry::new();
        registry.register(StaticTemplate::new("x", "a"));
        registry.register(StaticTemplate::new("y", "b"));
        let ids = registry.ids();
        assert!(ids.contains(&"x".to_string()));
        assert!(ids.contains(&"y".to_string()));
    }

    #[test]
    fn test_is_static_node_text() {
        let node = TemplateNode::Text("hello".to_string());
        assert!(is_static_node(&node));
    }

    #[test]
    fn test_is_static_node_dynamic() {
        let node = TemplateNode::Dynamic(Box::new(42i32));
        assert!(!is_static_node(&node));
    }

    #[test]
    fn test_is_static_node_element_with_static_children() {
        let node = TemplateNode::Element {
            tag: "div".to_string(),
            attrs: vec![("class".to_string(), "container".to_string())],
            events: vec![],
            children: vec![Template::text("hello")],
        };
        assert!(is_static_node(&node));
    }

    #[test]
    fn test_is_static_node_element_with_dynamic_children() {
        let node = TemplateNode::Element {
            tag: "div".to_string(),
            attrs: vec![],
            events: vec![],
            children: vec![Template {
                nodes: vec![TemplateNode::Dynamic(Box::new(42i32))],
            }],
        };
        assert!(!is_static_node(&node));
    }

    #[test]
    fn test_extract_static_html_static() {
        let template = Template {
            nodes: vec![TemplateNode::Element {
                tag: "p".to_string(),
                attrs: vec![],
                events: vec![],
                children: vec![Template::text("Hello")],
            }],
        };
        let html = extract_static_html(&template).unwrap();
        assert_eq!(html, "<p>Hello</p>");
    }

    #[test]
    fn test_extract_static_html_dynamic() {
        let template = Template {
            nodes: vec![TemplateNode::Dynamic(Box::new(42i32))],
        };
        assert!(extract_static_html(&template).is_none());
    }

    #[test]
    fn test_analyze_template_static() {
        let template = Template {
            nodes: vec![
                TemplateNode::Text("Hello".to_string()),
                TemplateNode::Element {
                    tag: "span".to_string(),
                    attrs: vec![],
                    events: vec![],
                    children: vec![Template::text("World")],
                },
            ],
        };
        let analysis = analyze_template(&template);
        assert!(analysis.is_static);
        assert_eq!(analysis.dynamic_node_count, 0);
        assert!(analysis.static_node_count >= 2);
    }

    #[test]
    fn test_analyze_template_dynamic() {
        let template = Template {
            nodes: vec![
                TemplateNode::Text("Hello".to_string()),
                TemplateNode::Dynamic(Box::new(42i32)),
            ],
        };
        let analysis = analyze_template(&template);
        assert!(!analysis.is_static);
        assert_eq!(analysis.dynamic_node_count, 1);
    }

    #[test]
    fn test_analyze_template_depth() {
        let template = Template {
            nodes: vec![TemplateNode::Element {
                tag: "div".to_string(),
                attrs: vec![],
                events: vec![],
                children: vec![Template {
                    nodes: vec![TemplateNode::Element {
                        tag: "span".to_string(),
                        attrs: vec![],
                        events: vec![],
                        children: vec![Template::text("deep")],
                    }],
                }],
            }],
        };
        let analysis = analyze_template(&template);
        assert_eq!(analysis.depth, 3);
    }
}
