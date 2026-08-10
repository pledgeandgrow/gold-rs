//! Memo — derived/computed state with automatic dependency tracking.

use crate::runtime::{self, Callback, ScopeId, SignalId};
use std::cell::RefCell;
use std::rc::Rc;

/// A computed value that automatically re-computes when its
/// signal dependencies change.
///
/// # Example
/// ```
/// use rye_signals::{Signal, Memo};
///
/// let count = Signal::new(2);
/// let count_clone = count.clone();
/// let doubled = Memo::new(move || count_clone.get() * 2);
/// assert_eq!(doubled.get(), 4);
/// count.set(5);
/// assert_eq!(doubled.get(), 10);
/// ```
pub struct Memo<T: Clone + 'static> {
    inner: Rc<RefCell<MemoInner<T>>>,
    scope_id: ScopeId,
    memo_id: SignalId,
}

struct MemoInner<T> {
    value: Option<T>,
    compute: Box<dyn Fn() -> T>,
}

impl<T: Clone + 'static> Memo<T> {
    /// Create a new memo. The closure runs immediately and whenever
    /// any signal read inside it changes.
    pub fn new<F: Fn() -> T + 'static>(compute: F) -> Self {
        let memo_id = runtime::next_id();
        let inner = Rc::new(RefCell::new(MemoInner {
            value: None,
            compute: Box::new(compute),
        }));

        // Register a placeholder scope to get the scope_id.
        // The callback will be replaced immediately after with one that
        // captures the correct scope_id.
        let scope_id = runtime::register_scope(Rc::new(RefCell::new(|| {})));

        // Create the real callback that references scope_id.
        let inner_clone = Rc::clone(&inner);
        let callback: Callback = Rc::new(RefCell::new(move || {
            // Clear old subscriptions before re-running
            runtime::clear_scope_subscriptions(scope_id);
            // Re-compute inside a tracking scope
            runtime::push_scope(scope_id);
            let value = (inner_clone.borrow().compute)();
            runtime::pop_scope();
            inner_clone.borrow_mut().value = Some(value);
            // Notify downstream scopes that depend on this memo
            runtime::notify(memo_id);
        }));

        // Replace the placeholder callback with the real one.
        runtime::update_scope_callback(scope_id, callback);

        // Initial computation
        runtime::push_scope(scope_id);
        let value = (inner.borrow().compute)();
        runtime::pop_scope();
        inner.borrow_mut().value = Some(value);

        Self {
            inner,
            scope_id,
            memo_id,
        }
    }

    /// Read the current value. Registers a dependency if inside a tracking scope.
    pub fn get(&self) -> T {
        runtime::track(self.memo_id);
        self.inner
            .borrow()
            .value
            .clone()
            .expect("Memo was not computed")
    }

    /// Read the current value without tracking.
    pub fn get_untracked(&self) -> T {
        self.inner
            .borrow()
            .value
            .clone()
            .expect("Memo was not computed")
    }

    /// Convenience method — shorthand for `.get()`.
    pub fn call(&self) -> T {
        self.get()
    }
}

impl<T: Clone + 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            scope_id: self.scope_id,
            memo_id: self.memo_id,
        }
    }
}

impl<T: Clone + 'static> Drop for Memo<T> {
    fn drop(&mut self) {
        runtime::unregister_scope(self.scope_id);
    }
}
