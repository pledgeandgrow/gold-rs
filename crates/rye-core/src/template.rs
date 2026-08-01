//! Template — compile-time template representation.
//!
//! The `template!` macro generates `Template` instances at compile time.
//! Static parts are created once and reused. Dynamic parts are bound
//! to signal subscriptions at runtime.

use crate::renderer::EventHandler;

/// A node in a template tree.
pub enum TemplateNode {
    /// A static text node.
    Text(String),
    /// A dynamic value (Rust expression result).
    Dynamic(Box<dyn std::any::Any + 'static>),
    /// An element with tag, attributes, events, and children.
    Element {
        /// Tag name (e.g. "div", "span").
        tag: String,
        /// Static attributes.
        attrs: Vec<(String, String)>,
        /// Event handlers.
        events: Vec<(String, EventHandler)>,
        /// Child template nodes.
        children: Vec<Template>,
    },
}

/// A template — the output of the `template!` macro.
///
/// Contains a tree of template nodes that can be rendered by a Renderer.
pub struct Template {
    /// The nodes in this template.
    pub nodes: Vec<TemplateNode>,
}

impl Template {
    /// Create a new template from a list of nodes.
    pub fn new(nodes: Vec<TemplateNode>) -> Self {
        Self { nodes }
    }

    /// Create a text template.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            nodes: vec![TemplateNode::Text(text.into())],
        }
    }

    /// Create an element template with attributes, events, and children.
    pub fn new_element(
        tag: impl Into<String>,
        attrs: Vec<(String, String)>,
        events: Vec<(String, EventHandler)>,
        children: Vec<Template>,
    ) -> Self {
        Self {
            nodes: vec![TemplateNode::Element {
                tag: tag.into(),
                attrs,
                events,
                children,
            }],
        }
    }

    /// Create an empty template.
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }
}
