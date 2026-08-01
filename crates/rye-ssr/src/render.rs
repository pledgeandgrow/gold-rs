//! Render — server-side rendering to string.

use rye_core::renderer::{EventHandler, Renderer};
use std::cell::RefCell;
use std::rc::Rc;

/// SSR renderer — produces HTML strings with hydration markers.
pub struct SsrRenderer {
    /// Counter for unique hydration IDs.
    next_id: RefCell<usize>,
}

/// SSR node — either an element or text, stored as a string.
#[derive(Clone, Debug)]
pub struct SsrNode {
    html: String,
}

/// SSR element — an HTML element string.
#[derive(Clone, Debug)]
pub struct SsrElement {
    html: String,
}

/// SSR text — a text node string.
#[derive(Clone, Debug)]
pub struct SsrText {
    content: String,
}

impl SsrRenderer {
    /// Create a new SSR renderer.
    pub fn new() -> Self {
        Self {
            next_id: RefCell::new(0),
        }
    }

    fn next_hydration_id(&self) -> String {
        let id = *self.next_id.borrow();
        *self.next_id.borrow_mut() = id + 1;
        format!("r{}", id)
    }
}

impl Default for SsrRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SsrRenderer {
    type Node = SsrNode;
    type Text = SsrText;
    type Element = SsrElement;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        let hydration_id = self.next_hydration_id();
        SsrElement {
            html: format!("<{} data-rye-id=\"{}\">", tag, hydration_id),
        }
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        SsrText {
            content: html_escape(content),
        }
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        // SSR text is immutable in this simple impl; in a real impl we'd use Rc<RefCell>
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        // In a real impl, we'd modify the element's attribute map
        // For now, SSR elements are built at creation time
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        // Same as above
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        // In a real impl, we'd insert into a children vector
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        // Same as above
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        // Same as above
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        // Same as above
    }

    fn set_event_listener(&mut self, _el: &Self::Element, _event: &str, _handler: EventHandler) {
        // SSR doesn't attach event listeners — they're attached during hydration
    }

    fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {
        // No-op in SSR
    }

    fn root(&self) -> Self::Element {
        SsrElement {
            html: String::new(),
        }
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        SsrNode {
            html: text.content.clone(),
        }
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        SsrNode {
            html: el.html.clone(),
        }
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Render an SSR element tree to an HTML string.
pub fn render_to_string(_root: &rye_core::Element) -> String {
    // In a full implementation, this walks the element tree and
    // uses SsrRenderer to produce HTML with hydration markers.
    // For now, return a placeholder.
    String::new()
}
