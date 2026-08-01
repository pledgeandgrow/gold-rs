//! DOM renderer — implements the Renderer trait using web-sys.
//!
//! Supports batched mutations via the `BatchRenderer` trait. When batching
//! is active, DOM operations are queued and applied in a single JS function
//! call on `flush_batch`, minimizing Wasm→JS bridge crossings.

use rye_core::renderer::{BatchRenderer, EventHandler, Renderer};
use crate::batch::{DomMutation, apply_mutations, apply_mutation_direct};

/// Web/DOM renderer using web-sys for WASM target.
pub struct DomRenderer {
    /// Whether batching is currently active.
    is_batching: bool,
    /// Queued mutations applied on `flush_batch`.
    pending_mutations: Vec<DomMutation>,
}

impl DomRenderer {
    /// Create a new DOM renderer attached to the document body.
    pub fn new() -> Self {
        Self {
            is_batching: false,
            pending_mutations: Vec::new(),
        }
    }

    /// Queue a mutation or apply it directly depending on batching state.
    fn queue_or_apply(&mut self, mutation: DomMutation) {
        if self.is_batching {
            self.pending_mutations.push(mutation);
        } else {
            apply_mutation_direct(&mutation);
        }
    }
}

impl Default for DomRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for DomRenderer {
    type Node = web_sys::Node;
    type Text = web_sys::Text;
    type Element = web_sys::Element;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_element(tag).unwrap()
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(content)
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        self.queue_or_apply(DomMutation::SetText {
            node: node.clone(),
            content: content.to_string(),
        });
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        self.queue_or_apply(DomMutation::SetAttribute {
            el: el.clone(),
            name: name.to_string(),
            value: value.to_string(),
        });
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        self.queue_or_apply(DomMutation::RemoveAttribute {
            el: el.clone(),
            name: name.to_string(),
        });
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        self.queue_or_apply(DomMutation::InsertChild {
            parent: parent.clone(),
            child: child.clone(),
            index,
        });
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        self.queue_or_apply(DomMutation::RemoveChild {
            parent: parent.clone(),
            index,
        });
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        self.queue_or_apply(DomMutation::ReplaceChild {
            parent: parent.clone(),
            new: new.clone(),
            index,
        });
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        self.queue_or_apply(DomMutation::MoveChild {
            parent: parent.clone(),
            from,
            to,
        });
    }

    fn set_event_listener(&mut self, _el: &Self::Element, _event: &str, _handler: EventHandler) {
        // TODO: attach event listener via wasm-bindgen closure
    }

    fn remove_event_listener(&mut self, _el: &Self::Element, _event: &str) {
        // TODO: remove event listener
    }

    fn root(&self) -> Self::Element {
        let document = web_sys::window().unwrap().document().unwrap();
        document.body().unwrap().into()
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        text.clone().into()
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        el.clone().into()
    }
}

impl BatchRenderer for DomRenderer {
    fn begin_batch(&mut self) {
        self.is_batching = true;
    }

    fn flush_batch(&mut self) {
        self.is_batching = false;
        if !self.pending_mutations.is_empty() {
            apply_mutations(&self.pending_mutations);
            self.pending_mutations.clear();
        }
    }
}
