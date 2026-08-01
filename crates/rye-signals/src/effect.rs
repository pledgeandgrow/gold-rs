//! Effect — side effects with automatic dependency tracking and cleanup.

use crate::runtime::{self, ScopeId, Callback};
use std::cell::RefCell;
use std::rc::Rc;

/// An effect that re-runs when any signal it reads changes.
///
/// # Example
/// ```
/// use rye_signals::{Signal, Effect};
/// use std::cell::Cell;
/// use std::rc::Rc;
///
/// let count = Signal::new(0);
/// let count_clone = count.clone();
/// let logged = Rc::new(Cell::new(0));
/// let logged_clone = Rc::clone(&logged);
///
/// let _eff = Effect::new(move || {
///     let _ = count_clone.get(); // track count
///     logged_clone.set(logged_clone.get() + 1);
/// });
///
/// assert_eq!(logged.get(), 1); // initial run
/// count.set(1);
/// assert_eq!(logged.get(), 2); // re-ran
/// count.set(2);
/// assert_eq!(logged.get(), 3); // re-ran
/// ```
pub struct Effect {
    scope_id: ScopeId,
}

impl Effect {
    /// Create a new effect. Runs immediately, then re-runs whenever
    /// any signal read inside the closure changes.
    pub fn new<F: Fn() + 'static>(callback: F) -> Self {
        let cb_ref = Rc::new(RefCell::new(callback));
        let scope_id = runtime::register_scope(Rc::new(RefCell::new(|| {})));

        let cb_clone = Rc::clone(&cb_ref);
        let callback: Callback = Rc::new(RefCell::new(move || {
            // Run cleanup from previous run
            runtime::pop_cleanup_scope();
            runtime::push_cleanup_scope();

            // Clear old subscriptions
            runtime::clear_scope_subscriptions(scope_id);

            // Run the effect inside a tracking scope
            runtime::push_scope(scope_id);
            (cb_clone.borrow())();
            runtime::pop_scope();
        }));
        runtime::update_scope_callback(scope_id, callback);

        // Initial run
        runtime::push_cleanup_scope();
        runtime::push_scope(scope_id);
        (cb_ref.borrow())();
        runtime::pop_scope();
        // Keep the cleanup scope — it will be run on next execution or drop

        Self { scope_id }
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        // Run cleanup functions
        runtime::pop_cleanup_scope();
        // Unregister from runtime
        runtime::unregister_scope(self.scope_id);
    }
}

/// Register a cleanup function for the current scope.
///
/// Cleanup runs before the effect re-runs or when the effect is dropped.
pub fn on_cleanup<F: FnOnce() + 'static>(cleanup: F) {
    runtime::on_cleanup(cleanup);
}
