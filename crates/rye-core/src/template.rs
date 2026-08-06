//! Template — compile-time template representation.
//!
//! The `template!` macro generates `Template` instances at compile time.
//! Static parts are created once and reused. Dynamic parts are bound
//! to signal subscriptions at runtime via fine-grained `Effect`s.

use crate::renderer::EventHandler;
use std::cell::RefCell;
use std::rc::Rc;

/// A shared event handler that can be referenced from borrowed templates.
pub type SharedEventHandler = Rc<RefCell<EventHandler>>;

/// Create a shared event handler from a closure.
///
/// Convenience for `Rc::new(RefCell::new(Box::new(closure)))`.
pub fn shared_event_handler<F>(f: F) -> SharedEventHandler
where
    F: FnMut(&dyn std::any::Any) + 'static,
{
    Rc::new(RefCell::new(Box::new(f)))
}

/// A reactive computation that produces a string value.
///
/// The closure is called inside an `Effect` scope, so any signals
/// it reads will cause the effect to re-run and update the DOM.
pub type ReactiveFn = Rc<dyn Fn() -> String + 'static>;

/// A reactive list computation that produces keyed items for reconciliation.
///
/// Each item is a `(key, Template)` pair. When the closure re-runs inside
/// an `Effect`, the returned keys are compared with the previous run using
/// the `reconcile` algorithm. Items are inserted, removed, or moved in the
/// DOM as needed — no full re-render.
pub type ReactiveListFn = Rc<dyn Fn() -> Vec<(crate::reconcile::Key, Template)> + 'static>;

/// A value that can be either static or reactive (signal-backed).
///
/// Used by component props to allow either a plain value or a `Signal<T>`.
/// When `Reactive`, the value is read inside an `Effect` scope, so changes
/// to the signal automatically update the DOM.
///
/// # Example
/// ```ignore
/// use rye_core::ReactiveValue;
/// use rye_signals::Signal;
///
/// // Static — never changes
/// let label = ReactiveValue::static("Submit");
///
/// // Reactive — updates when signal changes
/// let count = Signal::new(0);
/// let label = ReactiveValue::reactive(count.clone());
/// ```
#[derive(Clone, Debug)]
pub enum ReactiveValue<T: Clone + 'static> {
    /// A static value — computed once, never changes.
    Static(T),
    /// A reactive value — backed by a signal, updates automatically.
    Reactive(rye_signals::Signal<T>),
}

impl<T: Clone + 'static> ReactiveValue<T> {
    /// Create a static value.
    pub fn static_(value: T) -> Self {
        Self::Static(value)
    }

    /// Create a reactive value from a signal.
    pub fn reactive(signal: rye_signals::Signal<T>) -> Self {
        Self::Reactive(signal)
    }

    /// Read the current value (tracks dependencies if inside an Effect).
    pub fn get(&self) -> T {
        match self {
            Self::Static(v) => v.clone(),
            Self::Reactive(s) => s.get(),
        }
    }

    /// Check if this value is reactive (signal-backed).
    pub fn is_reactive(&self) -> bool {
        matches!(self, Self::Reactive(_))
    }
}

impl<T: Clone + 'static> From<T> for ReactiveValue<T> {
    fn from(value: T) -> Self {
        Self::Static(value)
    }
}

impl<T: Clone + std::fmt::Display + 'static> ReactiveValue<T> {
    /// Convert to a `ReactiveFn` for use in `TemplateNode::Reactive` or `reactive_attrs`.
    ///
    /// If static, the closure returns the formatted value once.
    /// If reactive, the closure reads the signal (tracking dependencies).
    pub fn to_reactive_fn(&self) -> ReactiveFn {
        match self {
            Self::Static(v) => {
                let s = v.to_string();
                Rc::new(move || s.clone())
            }
            Self::Reactive(s) => {
                let s = s.clone();
                Rc::new(move || s.get().to_string())
            }
        }
    }
}

/// A node in a template tree.
pub enum TemplateNode {
    /// A static text node.
    Text(String),
    /// A dynamic value (Rust expression result) — static, computed once.
    Dynamic(Box<dyn std::any::Any + 'static>),
    /// A reactive text binding — the closure is called inside an `Effect`,
    /// and the text node is updated whenever any signal it reads changes.
    Reactive(ReactiveFn),
    /// A reactive list of children with keyed reconciliation.
    /// The closure returns a vec of `(key, Template)` pairs. When the closure
    /// re-runs (due to signal changes), the list is reconciled using the
    /// `reconcile` algorithm — items are inserted, removed, or moved as needed.
    ReactiveList {
        /// The reactive list computation — returns keyed items.
        items_fn: ReactiveListFn,
    },
    /// An element with tag, attributes, events, and children.
    Element {
        /// Tag name (e.g. "div", "span").
        tag: String,
        /// Static attributes.
        attrs: Vec<(String, String)>,
        /// Reactive attributes — each closure is called inside an `Effect`
        /// to update the attribute when signals change.
        reactive_attrs: Vec<(String, ReactiveFn)>,
        /// Event handlers (shared so they can be referenced from borrowed templates).
        events: Vec<(String, SharedEventHandler)>,
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

    /// Create an element template with attributes, reactive attrs, events, and children.
    pub fn new_element(
        tag: impl Into<String>,
        attrs: Vec<(String, String)>,
        events: Vec<(String, SharedEventHandler)>,
        children: Vec<Template>,
    ) -> Self {
        Self {
            nodes: vec![TemplateNode::Element {
                tag: tag.into(),
                attrs,
                reactive_attrs: Vec::new(),
                events,
                children,
            }],
        }
    }

    /// Create an element template with reactive attributes.
    pub fn new_element_reactive(
        tag: impl Into<String>,
        attrs: Vec<(String, String)>,
        reactive_attrs: Vec<(String, ReactiveFn)>,
        events: Vec<(String, SharedEventHandler)>,
        children: Vec<Template>,
    ) -> Self {
        Self {
            nodes: vec![TemplateNode::Element {
                tag: tag.into(),
                attrs,
                reactive_attrs,
                events,
                children,
            }],
        }
    }

    /// Create an empty template.
    pub fn empty() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Create a reactive list template with keyed reconciliation.
    ///
    /// The closure is called inside an `Effect`. When any signal it reads
    /// changes, the closure re-runs and the returned items are reconciled
    /// against the previous run using keyed diffing — items are inserted,
    /// removed, or moved in the DOM as needed.
    ///
    /// # Example
    /// ```ignore
    /// use rye_core::template::Template;
    /// use rye_signals::Signal;
    ///
    /// let items = Signal::new(vec!["a".to_string(), "b".to_string()]);
    /// let list = Template::new_reactive_list(move || {
    ///     items.get().iter().map(|item| {
    ///         (item.len(), Template::text(item.clone()))
    ///     }).collect()
    /// });
    /// ```
    pub fn new_reactive_list(items_fn: ReactiveListFn) -> Self {
        Self {
            nodes: vec![TemplateNode::ReactiveList { items_fn }],
        }
    }
}
