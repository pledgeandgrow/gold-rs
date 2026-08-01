//! Event delegation — efficient event handling via a single root listener.
//!
//! Instead of attaching individual event listeners to each element,
//! a single listener is attached at the root for each event type.
//! Events bubble up and are dispatched to the correct handler based
//! on the event target's data-rye-event-id attribute.

use std::cell::RefCell;
use std::collections::HashMap;

/// Unique ID for an event handler.
type HandlerId = usize;

/// A delegated event handler.
type Handler = Box<dyn FnMut(&dyn std::any::Any) + 'static>;

/// The event delegation registry — maps handler IDs to callbacks.
pub struct EventDelegator {
    /// Map of handler ID → handler callback.
    handlers: RefCell<HashMap<HandlerId, Handler>>,
    /// Map of (element data-rye-id, event name) → handler ID.
    element_handlers: RefCell<HashMap<(String, String), HandlerId>>,
    /// Set of event types that have been registered at the root.
    registered_events: RefCell<std::collections::HashSet<String>>,
    /// Next handler ID.
    next_id: RefCell<usize>,
}

impl EventDelegator {
    /// Create a new event delegator.
    pub fn new() -> Self {
        Self {
            handlers: RefCell::new(HashMap::new()),
            element_handlers: RefCell::new(HashMap::new()),
            registered_events: RefCell::new(std::collections::HashSet::new()),
            next_id: RefCell::new(0),
        }
    }

    /// Register an event handler for a specific element and event type.
    ///
    /// Returns a handler ID that can be used to remove the handler later.
    /// The element should have a `data-rye-event-id` attribute set to
    /// the returned ID for the delegation to work.
    pub fn add_handler(
        &self,
        element_id: &str,
        event: &str,
        handler: Handler,
    ) -> HandlerId {
        // Remove existing handler for this element+event
        let key = (element_id.to_string(), event.to_string());
        if let Some(old_id) = self.element_handlers.borrow().get(&key).copied() {
            self.handlers.borrow_mut().remove(&old_id);
        }

        let id = {
            let mut next = self.next_id.borrow_mut();
            let id = *next;
            *next += 1;
            id
        };

        self.handlers.borrow_mut().insert(id, handler);
        self.element_handlers.borrow_mut().insert(key, id);
        self.registered_events.borrow_mut().insert(event.to_string());

        id
    }

    /// Remove a handler by its ID.
    pub fn remove_handler(&self, id: HandlerId) {
        self.handlers.borrow_mut().remove(&id);
        // Also clean up element_handlers mapping
        self.element_handlers.borrow_mut().retain(|_, v| *v != id);
    }

    /// Remove all handlers for a specific element and event type.
    pub fn remove_element_handler(&self, element_id: &str, event: &str) {
        let key = (element_id.to_string(), event.to_string());
        if let Some(id) = self.element_handlers.borrow().get(&key).copied() {
            self.handlers.borrow_mut().remove(&id);
            self.element_handlers.borrow_mut().remove(&key);
        }
    }

    /// Dispatch an event to the appropriate handler.
    ///
    /// `element_id` is the data-rye-event-id of the target element.
    /// `event` is the event type (e.g. "click", "input").
    /// `payload` is the event data.
    pub fn dispatch(&self, element_id: &str, event: &str, payload: &dyn std::any::Any) {
        let key = (element_id.to_string(), event.to_string());
        let id = match self.element_handlers.borrow().get(&key).copied() {
            Some(id) => id,
            None => return,
        };
        if let Some(handler) = self.handlers.borrow_mut().get_mut(&id) {
            handler(payload);
        }
    }

    /// Get all event types that have been registered.
    pub fn registered_event_types(&self) -> Vec<String> {
        self.registered_events.borrow().iter().cloned().collect()
    }

    /// Check if any handlers are registered for a given event type.
    pub fn has_handlers_for(&self, event: &str) -> bool {
        self.element_handlers
            .borrow()
            .keys()
            .any(|(_, e)| e == event)
    }

    /// Clear all handlers.
    pub fn clear(&self) {
        self.handlers.borrow_mut().clear();
        self.element_handlers.borrow_mut().clear();
        self.registered_events.borrow_mut().clear();
    }
}

impl Default for EventDelegator {
    fn default() -> Self {
        Self::new()
    }
}

/// Events that are commonly delegated.
pub mod events {
    /// Click event.
    pub const CLICK: &str = "click";
    /// Input event.
    pub const INPUT: &str = "input";
    /// Change event.
    pub const CHANGE: &str = "change";
    /// Keydown event.
    pub const KEYDOWN: &str = "keydown";
    /// Keyup event.
    pub const KEYUP: &str = "keyup";
    /// Keypress event.
    pub const KEYPRESS: &str = "keypress";
    /// Submit event.
    pub const SUBMIT: &str = "submit";
    /// Focus event.
    pub const FOCUS: &str = "focus";
    /// Blur event.
    pub const BLUR: &str = "blur";
    /// Mouseenter event.
    pub const MOUSEENTER: &str = "mouseenter";
    /// Mouseleave event.
    pub const MOUSELEAVE: &str = "mouseleave";
    /// Scroll event.
    pub const SCROLL: &str = "scroll";
    /// Resize event.
    pub const RESIZE: &str = "resize";

    /// All commonly delegated event types.
    pub const ALL: &[&str] = &[
        CLICK, INPUT, CHANGE, KEYDOWN, KEYUP, KEYPRESS,
        SUBMIT, FOCUS, BLUR, MOUSEENTER, MOUSELEAVE,
        SCROLL, RESIZE,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_add_and_dispatch() {
        let delegator = EventDelegator::new();
        let counter = Rc::new(Cell::new(0));
        let counter_clone = Rc::clone(&counter);

        let id = delegator.add_handler("elem1", "click", Box::new(move |_| {
            counter_clone.set(counter_clone.get() + 1);
        }));

        delegator.dispatch("elem1", "click", &());
        assert_eq!(counter.get(), 1);

        delegator.dispatch("elem1", "click", &());
        assert_eq!(counter.get(), 2);

        // Dispatch to wrong element should not trigger
        delegator.dispatch("elem2", "click", &());
        assert_eq!(counter.get(), 2);

        // Remove handler
        delegator.remove_handler(id);
        delegator.dispatch("elem1", "click", &());
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_replace_handler() {
        let delegator = EventDelegator::new();
        let result = Rc::new(Cell::new(0));

        let r1 = Rc::clone(&result);
        delegator.add_handler("elem1", "click", Box::new(move |_| {
            r1.set(1);
        }));

        let r2 = Rc::clone(&result);
        delegator.add_handler("elem1", "click", Box::new(move |_| {
            r2.set(2);
        }));

        delegator.dispatch("elem1", "click", &());
        assert_eq!(result.get(), 2); // Second handler replaced first
    }

    #[test]
    fn test_registered_events() {
        let delegator = EventDelegator::new();

        delegator.add_handler("elem1", "click", Box::new(|_| {}));
        delegator.add_handler("elem2", "input", Box::new(|_| {}));

        assert!(delegator.has_handlers_for("click"));
        assert!(delegator.has_handlers_for("input"));
        assert!(!delegator.has_handlers_for("submit"));

        let events = delegator.registered_event_types();
        assert!(events.contains(&"click".to_string()));
        assert!(events.contains(&"input".to_string()));
    }
}
