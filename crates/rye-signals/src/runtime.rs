//! Reactive runtime — dependency tracking and batch queue.
//!
//! Thread-local system that tracks which signals are read inside which scopes.
//! When a signal changes, all dependent scopes are re-run.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Unique ID for a signal.
pub type SignalId = usize;

/// Unique ID for a scope (Effect/Memo).
pub type ScopeId = usize;

/// A subscriber callback — stored as Rc so it can be referenced by multiple signals.
pub type Callback = Rc<RefCell<dyn Fn()>>;

thread_local! {
    /// Stack of currently-active scope IDs (for dependency tracking).
    static SCOPE_STACK: RefCell<Vec<ScopeId>> = const { RefCell::new(Vec::new()) };

    /// All registered scopes — maps scope ID to its callback.
    static SCOPES: RefCell<HashMap<ScopeId, Callback>> = RefCell::new(HashMap::new());

    /// All signal subscribers — maps signal ID to list of (scope ID, callback).
    static SUBSCRIBERS: RefCell<HashMap<SignalId, Vec<(ScopeId, Callback)>>> = RefCell::new(HashMap::new());

    /// Batch state.
    static BATCH_STATE: RefCell<BatchState> = const { RefCell::new(BatchState::new()) };

    /// Cleanup stack — each scope can have cleanup functions.
    static CLEANUP_STACK: RefCell<Vec<Vec<Box<dyn FnOnce()>>>> = const { RefCell::new(Vec::new()) };

    /// Next ID counter (shared by signals and scopes).
    static NEXT_ID: RefCell<usize> = const { RefCell::new(0) };
}

struct BatchState {
    batching: bool,
    dirty_signals: Vec<SignalId>,
}

impl BatchState {
    const fn new() -> Self {
        Self {
            batching: false,
            dirty_signals: Vec::new(),
        }
    }
}

/// Get a unique ID (used for both signals and scopes).
pub(crate) fn next_id() -> usize {
    NEXT_ID.with(|n| {
        let val = *n.borrow();
        *n.borrow_mut() = val + 1;
        val
    })
}

// ── Scope management ──────────────────────────────────────────

/// Register a new scope with its callback. Returns the scope ID.
pub(crate) fn register_scope(callback: Callback) -> ScopeId {
    let id = next_id();
    SCOPES.with(|s| s.borrow_mut().insert(id, callback));
    id
}

/// Unregister a scope (on destroy).
pub(crate) fn unregister_scope(id: ScopeId) {
    SCOPES.with(|s| {
        s.borrow_mut().remove(&id);
    });
    SUBSCRIBERS.with(|subs| {
        for (_, list) in subs.borrow_mut().iter_mut() {
            list.retain(|(scope_id, _)| *scope_id != id);
        }
    });
}

/// Update a scope's callback (used when the callback needs to reference the scope_id).
pub(crate) fn update_scope_callback(id: ScopeId, callback: Callback) {
    SCOPES.with(|s| {
        s.borrow_mut().insert(id, callback);
    });
}

/// Push a scope ID onto the tracking stack.
pub(crate) fn push_scope(id: ScopeId) {
    SCOPE_STACK.with(|s| s.borrow_mut().push(id));
}

/// Pop the top scope from the tracking stack.
pub(crate) fn pop_scope() {
    SCOPE_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

// ── Dependency tracking ───────────────────────────────────────

/// Register that the current scope depends on a signal.
/// Called when a signal is read inside a tracking scope.
pub(crate) fn track(signal_id: SignalId) {
    SCOPE_STACK.with(|stack| {
        if let Some(&scope_id) = stack.borrow().last() {
            SCOPES.with(|scopes| {
                if let Some(callback) = scopes.borrow().get(&scope_id) {
                    let callback = Rc::clone(callback);
                    SUBSCRIBERS.with(|subs| {
                        let mut subs = subs.borrow_mut();
                        let list = subs.entry(signal_id).or_default();
                        if !list.iter().any(|(sid, _)| *sid == scope_id) {
                            list.push((scope_id, callback));
                        }
                    });
                }
            });
        }
    });
}

/// Notify all subscribers of a signal that it changed.
/// If batching, queues the signal ID. Otherwise, runs callbacks immediately.
pub(crate) fn notify(signal_id: SignalId) {
    if BATCH_STATE.with(|b| b.borrow().batching) {
        BATCH_STATE.with(|b| {
            let mut state = b.borrow_mut();
            if !state.dirty_signals.contains(&signal_id) {
                state.dirty_signals.push(signal_id);
            }
        });
    } else {
        run_subscribers(signal_id);
    }
}

/// Run all subscriber callbacks for a signal.
fn run_subscribers(signal_id: SignalId) {
    let callbacks: Vec<Callback> = SUBSCRIBERS.with(|subs| {
        subs.borrow()
            .get(&signal_id)
            .map(|list| list.iter().map(|(_, cb)| Rc::clone(cb)).collect())
            .unwrap_or_default()
    });

    for cb in callbacks {
        let cb_ref = cb.borrow();
        cb_ref();
    }
}

// ── Batch ─────────────────────────────────────────────────────

/// Start batching — signal notifications will be queued.
pub(crate) fn start_batch() {
    BATCH_STATE.with(|b| {
        b.borrow_mut().batching = true;
    });
}

/// End batching — flush all pending notifications.
pub(crate) fn end_batch() {
    let dirty: Vec<SignalId> = BATCH_STATE.with(|b| {
        let mut state = b.borrow_mut();
        state.batching = false;
        std::mem::take(&mut state.dirty_signals)
    });

    for signal_id in dirty {
        run_subscribers(signal_id);
    }
}

/// Check if currently batching.
pub(crate) fn is_batching() -> bool {
    BATCH_STATE.with(|b| b.borrow().batching)
}

// ── Cleanup ───────────────────────────────────────────────────

/// Push a new cleanup scope.
pub(crate) fn push_cleanup_scope() {
    CLEANUP_STACK.with(|s| s.borrow_mut().push(Vec::new()));
}

/// Pop and run all cleanup functions for the current scope.
pub(crate) fn pop_cleanup_scope() {
    let cleanups = CLEANUP_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
    for cleanup in cleanups {
        cleanup();
    }
}

/// Register a cleanup function for the current scope.
pub(crate) fn on_cleanup<F: FnOnce() + 'static>(cleanup: F) {
    CLEANUP_STACK.with(|s| {
        if let Some(scope) = s.borrow_mut().last_mut() {
            scope.push(Box::new(cleanup));
        }
    });
}

/// Clear all subscriptions for a scope (before re-running, so it can re-subscribe).
pub(crate) fn clear_scope_subscriptions(scope_id: ScopeId) {
    SUBSCRIBERS.with(|subs| {
        for (_, list) in subs.borrow_mut().iter_mut() {
            list.retain(|(sid, _)| *sid != scope_id);
        }
    });
}

/// Get all signal IDs that have no subscribers.
pub fn get_signals_with_no_subscribers() -> Vec<SignalId> {
    SUBSCRIBERS.with(|subs| {
        subs.borrow()
            .iter()
            .filter(|(_, list)| list.is_empty())
            .map(|(id, _)| *id)
            .collect()
    })
}

/// Remove a signal's subscriber entry entirely (for pruning).
pub fn remove_subscriber_entry(signal_id: SignalId) {
    SUBSCRIBERS.with(|subs| {
        subs.borrow_mut().remove(&signal_id);
    });
}

/// Get the subscriber count for a signal.
pub fn subscriber_count(signal_id: SignalId) -> usize {
    SUBSCRIBERS.with(|subs| {
        subs.borrow().get(&signal_id).map(|l| l.len()).unwrap_or(0)
    })
}
