//! Renderer trait — abstraction over rendering backends.

use std::any::Any;

/// A unique ID for a node in the renderer.
pub type NodeId = usize;

/// The Renderer trait abstracts the rendering surface.
///
/// Implementations include:
/// - `DomRenderer` (web/WASM via web-sys)
/// - `SsrRenderer` (server-side rendering to string)
/// - `NativeRenderer` (desktop/mobile via wgpu)
/// - `TestRenderer` (in-memory for testing)
///
/// The core framework never depends on a specific renderer.
/// All rendering goes through this trait.
pub trait Renderer: 'static {
    /// The node type for this renderer (e.g. `web_sys::Node`, `String`, `TestNode`).
    type Node: Clone;
    /// The text node type.
    type Text: Clone;
    /// The element node type.
    type Element: Clone;

    /// Create a new element node with the given tag name.
    fn create_element(&mut self, tag: &str) -> Self::Element;

    /// Create a new text node with the given content.
    fn create_text(&mut self, content: &str) -> Self::Text;

    /// Set the text content of a text node.
    fn set_text(&mut self, node: &Self::Text, content: &str);

    /// Set an attribute on an element.
    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str);

    /// Remove an attribute from an element.
    fn remove_attribute(&mut self, el: &Self::Element, name: &str);

    /// Insert a child node at a specific index in the parent's children.
    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize);

    /// Remove the child at the given index from the parent.
    fn remove_child(&mut self, parent: &Self::Element, index: usize);

    /// Replace the child at the given index with a new node.
    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize);

    /// Move a child from one index to another within the same parent.
    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize);

    /// Set an event listener on an element.
    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler);

    /// Remove an event listener from an element.
    fn remove_event_listener(&mut self, el: &Self::Element, event: &str);

    /// Get the root node of the renderer (for mounting).
    fn root(&self) -> Self::Element;

    /// Convert a text node to a generic node.
    fn text_to_node(&self, text: &Self::Text) -> Self::Node;

    /// Convert an element node to a generic node.
    fn element_to_node(&self, el: &Self::Element) -> Self::Node;
}

/// Optional trait for renderers that support batched operations.
/// Batched renderers collect mutations and apply them in a single flush,
/// minimizing reflows and repaints.
pub trait BatchRenderer: Renderer {
    /// Begin a batch — subsequent operations are queued.
    fn begin_batch(&mut self);

    /// Flush all queued operations to the backend.
    fn flush_batch(&mut self);
}

/// An event handler callback.
pub type EventHandler = Box<dyn FnMut(&dyn Any) + 'static>;
