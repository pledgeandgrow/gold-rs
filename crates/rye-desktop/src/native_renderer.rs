//! Native GPU renderer — implements Renderer trait using wgpu.

use rye_core::renderer::Renderer;

/// Native GPU renderer using wgpu for desktop platforms.
pub struct NativeRenderer {
    // TODO: wgpu surface, device, queue, render pipeline
}

impl NativeRenderer {
    /// Create a new native renderer for the given window.
    pub fn new() -> Self {
        // TODO: initialize wgpu surface, device, queue
        Self {}
    }
}

impl Renderer for NativeRenderer {
    type Node = RenderNode;
    type Text = RenderText;
    type Element = RenderElement;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        RenderElement {
            tag: tag.to_string(),
            ..Default::default()
        }
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        RenderText {
            content: content.to_string(),
        }
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        // Text nodes are immutable in the render tree; create new
        // TODO: implement proper text update via glyph atlas
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        // TODO: update element attributes, trigger re-layout if needed
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        // TODO: remove attribute
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        // TODO: insert child in render tree, trigger re-layout
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        // TODO: remove child from render tree
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        // TODO: replace child
    }

    fn set_event_listener(&mut self, _el: &Self::Element, _event: &str, _handler: rye_core::renderer::EventHandler) {
        // TODO: register with hit testing system
    }

    fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {
        // TODO: unregister from hit testing system
    }

    fn root(&self) -> Self::Element {
        RenderElement::default()
    }

    fn move_child(&mut self, _parent: &Self::Element, _from: usize, _to: usize) {
        // TODO: move child in render tree
    }

    fn text_to_node(&self, _text: &Self::Text) -> Self::Node {
        RenderNode::default()
    }

    fn element_to_node(&self, _el: &Self::Element) -> Self::Node {
        RenderNode::default()
    }
}

/// A render node in the native render tree.
#[derive(Clone, Default)]
pub struct RenderNode {
    // TODO: node data
}

/// A text node in the native render tree.
#[derive(Clone)]
pub struct RenderText {
    /// The text content.
    pub content: String,
}

/// An element node in the native render tree.
#[derive(Clone, Default)]
pub struct RenderElement {
    /// The tag name.
    pub tag: String,
    // TODO: attributes, children, layout node, style
}
