//! Element — the output of a component render.

use crate::template::Template;

/// An Element is a node in the component tree.
///
/// It can be:
/// - A template instance (from `template!` macro)
/// - A component instance
/// - A fragment (multiple nodes)
/// - Nothing (empty, for conditional rendering)
pub enum Element {
    /// A single template node.
    Template(Template),
    /// A nested component.
    Component(Box<dyn std::any::Any>),
    /// Multiple elements (fragment).
    Fragment(Vec<Element>),
    /// No element (for conditionals that evaluate to false).
    None,
}

impl Element {
    /// Create an empty element.
    pub fn none() -> Self {
        Element::None
    }
}
