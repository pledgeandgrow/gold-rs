//! Context system — type-safe dependency injection without prop drilling.

use rye_signals::Signal;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A stored context value — either a plain value or a signal.
enum ContextEntry {
    Value(Box<dyn Any>),
    Signal(Rc<dyn Any>),
}

thread_local! {
    /// Stack of context scopes — each scope is a map of TypeId → ContextEntry.
    /// provide_context pushes a new scope, use_context searches from top down.
    static CONTEXT_STACK: RefCell<Vec<HashMap<TypeId, ContextEntry>>> =
        RefCell::new(Vec::new());
}

/// Push a new context scope. Call this when entering a component subtree.
pub fn push_context_scope() {
    CONTEXT_STACK.with(|s| s.borrow_mut().push(HashMap::new()));
}

/// Pop the top context scope. Call this when leaving a component subtree.
pub fn pop_context_scope() {
    CONTEXT_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Provide a value to all child components in the current scope.
///
/// # Example
/// ```
/// use rye_core::context::provide_context;
///
/// provide_context(42i32);
/// ```
pub fn provide_context<T: Any + 'static>(value: T) {
    CONTEXT_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if let Some(scope) = stack.last_mut() {
            scope.insert(TypeId::of::<T>(), ContextEntry::Value(Box::new(value)));
        } else {
            // If no scope exists, create one
            let mut map = HashMap::new();
            map.insert(TypeId::of::<T>(), ContextEntry::Value(Box::new(value)));
            stack.push(map);
        }
    });
}

/// Provide a signal to all child components in the current scope.
///
/// # Example
/// ```
/// use rye_core::context::provide_context_signal;
/// use rye_signals::Signal;
///
/// provide_context_signal(Signal::new(42i32));
/// ```
pub fn provide_context_signal<T: Any + 'static>(signal: Signal<T>) {
    let rc_signal = Rc::new(signal);
    CONTEXT_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        if let Some(scope) = stack.last_mut() {
            scope.insert(TypeId::of::<T>(), ContextEntry::Signal(rc_signal));
        } else {
            let mut map = HashMap::new();
            map.insert(TypeId::of::<T>(), ContextEntry::Signal(rc_signal));
            stack.push(map);
        }
    });
}

/// Consume a context value of the given type.
///
/// Searches from the innermost scope outward. Returns `None` if not found.
///
/// # Example
/// ```
/// use rye_core::context::{provide_context, use_context};
///
/// provide_context(42i32);
/// let value: i32 = use_context().expect("context not provided");
/// assert_eq!(value, 42);
/// ```
pub fn use_context<T: Any + Clone + 'static>() -> Option<T> {
    CONTEXT_STACK.with(|s| {
        let stack = s.borrow();
        for scope in stack.iter().rev() {
            if let Some(entry) = scope.get(&TypeId::of::<T>()) {
                match entry {
                    ContextEntry::Value(val) => {
                        return val.downcast_ref::<T>().cloned();
                    }
                    ContextEntry::Signal(sig) => {
                        if let Some(signal) = sig.downcast_ref::<Signal<T>>() {
                            return Some(signal.get_untracked());
                        }
                    }
                }
            }
        }
        None
    })
}

/// Consume a context signal of the given type.
///
/// Returns the `Signal<T>` so you can reactively read it.
pub fn use_context_signal<T: Any + Clone + 'static>() -> Option<Signal<T>> {
    CONTEXT_STACK.with(|s| {
        let stack = s.borrow();
        for scope in stack.iter().rev() {
            if let Some(entry) = scope.get(&TypeId::of::<T>()) {
                match entry {
                    ContextEntry::Signal(sig) => {
                        if let Some(signal) = sig.downcast_ref::<Signal<T>>() {
                            return Some(signal.clone());
                        }
                    }
                    ContextEntry::Value(val) => {
                        if let Some(v) = val.downcast_ref::<T>() {
                            return Some(Signal::new(v.clone()));
                        }
                    }
                }
            }
        }
        None
    })
}
