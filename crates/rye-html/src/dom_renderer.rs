//! DOM renderer — implements the Renderer trait using web-sys.
//!
//! Supports batched mutations via the `BatchRenderer` trait. When batching
//! is active, DOM operations are queued and applied in a single JS function
//! call on `flush_batch`, minimizing Wasm→JS bridge crossings.
//!
//! ## Event Delegation
//!
//! Instead of attaching individual event listeners to each element (which
//! leaks `wasm-bindgen` closures), the renderer uses event delegation:
//!
//! 1. A single root listener is attached per event type (click, input, etc.)
//! 2. Each element with an event handler gets a `data-rye-event-id` attribute
//! 3. When an event fires, the root listener walks up from `event.target`
//!    to find the nearest `data-rye-event-id` and dispatches to the registry
//! 4. Handlers are stored in an `EventDelegator` and removed by ID — no leaks

use rye_core::event_delegation::EventDelegator;
use rye_core::renderer::{BatchRenderer, EventHandler, Hydratable, Renderer};
use crate::batch::{DomMutation, apply_mutations, apply_mutation_direct};
use crate::events::dom_event_name;

use std::cell::RefCell;
use std::rc::Rc;

/// Attribute name for event handler IDs on DOM elements.
const EVENT_ID_ATTR: &str = "data-rye-event-id";

/// Attribute name for the event type on DOM elements.
const EVENT_TYPE_ATTR: &str = "data-rye-event";

/// Web/DOM renderer using web-sys for WASM target.
///
/// Uses event delegation — one root listener per event type, handlers
/// stored in a registry. No per-element closures, no memory leaks.
pub struct DomRenderer {
    /// Whether batching is currently active.
    is_batching: bool,
    /// Queued mutations applied on `flush_batch`.
    pending_mutations: Vec<DomMutation>,
    /// Event delegation registry — maps handler IDs to callbacks.
    delegator: Rc<RefCell<EventDelegator>>,
    /// Root-level closures kept alive to prevent GC (one per event type).
    root_closures: RefCell<Vec<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>>>,
    /// Next element ID for unique data-rye-event-id attributes.
    next_element_id: RefCell<usize>,
    /// Map of (element, event) → element_id string for cleanup.
    element_event_ids: RefCell<std::collections::HashMap<(web_sys::Element, String), String>>,
}

impl DomRenderer {
    /// Create a new DOM renderer attached to the document body.
    pub fn new() -> Self {
        Self {
            is_batching: false,
            pending_mutations: Vec::new(),
            delegator: Rc::new(RefCell::new(EventDelegator::new())),
            root_closures: RefCell::new(Vec::new()),
            next_element_id: RefCell::new(0),
            element_event_ids: RefCell::new(std::collections::HashMap::new()),
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

    /// Generate a unique element ID for event delegation.
    fn next_id(&self) -> String {
        let id = *self.next_element_id.borrow();
        *self.next_element_id.borrow_mut() = id + 1;
        format!("e{}", id)
    }

    /// Set up root-level delegated event listeners.
    ///
    /// Attaches one listener per event type at the document body. When
    /// an event fires, the listener walks up from `event.target` to find
    /// the nearest element with `data-rye-event-id` and dispatches to the
    /// `EventDelegator` registry.
    ///
    /// Call this once after `mount()` to activate event delegation.
    pub fn setup_delegation(&self) {
        use wasm_bindgen::JsCast;

        let root = self.root();
        let delegator = Rc::clone(&self.delegator);

        // Events to delegate at the root level
        let event_types = [
            "click", "input", "change", "submit",
            "keydown", "keyup", "keypress",
            "focus", "blur",
            "mouseenter", "mouseleave",
            "mousedown", "mouseup", "mousemove",
            "touchstart", "touchend", "touchmove",
            "scroll", "resize",
        ];

        for event_type in event_types {
            let delegator_clone = Rc::clone(&delegator);
            let event_type_owned = event_type.to_string();

            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                move |event: web_sys::Event| {
                    // Walk up from event target to find data-rye-event-id
                    let target = match event.target() {
                        Some(t) => t,
                        None => return,
                    };

                    let element = target.dyn_into::<web_sys::Element>();
                    let mut current = match element {
                        Ok(el) => Some(el),
                        Err(node) => {
                            // Target is a text node — get parent
                            node.parent_element()
                        }
                    };

                    while let Some(el) = current {
                        // Check if this element has a data-rye-event-id
                        if let Some(handler_id) = el.get_attribute(EVENT_ID_ATTR) {
                            // Check if the event type matches
                            if let Some(el_event) = el.get_attribute(EVENT_TYPE_ATTR) {
                                if el_event == event_type_owned {
                                    delegator_clone.dispatch(
                                        &handler_id,
                                        &event_type_owned,
                                        &event as &dyn std::any::Any,
                                    );
                                    return;
                                }
                            }
                        }
                        current = el.parent_element();
                    }
                },
            ) as Box<dyn FnMut(web_sys::Event)>);

            let js_fn = closure.as_ref().unchecked_ref();
            let _ = root.add_event_listener_with_callback(event_type, js_fn);

            // Keep the closure alive — stored in the renderer, not leaked
            self.root_closures.borrow_mut().push(closure);
        }
    }

    /// Get a reference to the event delegator (for testing/inspection).
    pub fn delegator(&self) -> &Rc<RefCell<EventDelegator>> {
        &self.delegator
    }

    /// Clear all event handlers and root listeners.
    pub fn clear_events(&self) {
        self.delegator.borrow().clear();
        self.root_closures.borrow_mut().clear();
        self.element_event_ids.borrow_mut().clear();
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

    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler) {
        let dom_event = dom_event_name(event).to_string();

        // Generate a unique element ID for this element+event pair
        let element_id = self.next_id();

        // Register handler in the delegator
        self.delegator.borrow().add_handler(&element_id, &dom_event, handler);

        // Set data attributes on the element for delegation dispatch
        let _ = el.set_attribute(EVENT_ID_ATTR, &element_id);
        let _ = el.set_attribute(EVENT_TYPE_ATTR, &dom_event);

        // Track for cleanup
        self.element_event_ids
            .borrow_mut()
            .insert((el.clone(), dom_event), element_id);
    }

    fn remove_event_listener(&mut self, el: &Self::Element, event: &str) {
        let dom_event = dom_event_name(event).to_string();
        let key = (el.clone(), dom_event.clone());

        if let Some(element_id) = self.element_event_ids.borrow_mut().remove(&key) {
            self.delegator.borrow().remove_element_handler(&element_id, &dom_event);
            let _ = el.remove_attribute(EVENT_ID_ATTR);
            let _ = el.remove_attribute(EVENT_TYPE_ATTR);
        }
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

impl Hydratable for DomRenderer {
    fn get_child_node(&self, parent: &Self::Element, index: usize) -> Option<Self::Node> {
        parent.child_nodes().item(index as u32)
    }

    fn child_node_count(&self, parent: &Self::Element) -> usize {
        parent.child_nodes().length() as usize
    }

    fn node_is_element(&self, node: &Self::Node) -> bool {
        node.node_type() == 1 // Node.ELEMENT_NODE
    }

    fn node_is_text(&self, node: &Self::Node) -> bool {
        node.node_type() == 3 // Node.TEXT_NODE
    }

    fn node_as_element(&self, node: &Self::Node) -> Option<Self::Element> {
        use wasm_bindgen::JsCast;
        node.clone().dyn_into::<web_sys::Element>().ok()
    }

    fn node_as_text(&self, node: &Self::Node) -> Option<Self::Text> {
        use wasm_bindgen::JsCast;
        node.clone().dyn_into::<web_sys::Text>().ok()
    }

    fn get_text_content(&self, text: &Self::Text) -> String {
        text.text_content().unwrap_or_default()
    }

    fn get_tag_name(&self, el: &Self::Element) -> String {
        el.tag_name().to_lowercase()
    }
}
