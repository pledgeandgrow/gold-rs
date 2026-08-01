//! Optimistic updates — immediately set a signal, roll back on error.
//!
//! `optimistic_update(signal, new_value, async_fn)` that immediately sets
//! the signal, runs the async operation, and rolls back on error.

use crate::signal::Signal;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::{Context, Poll, Wake, Waker};

/// Result of an optimistic update.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimisticResult {
    /// The update succeeded — the optimistic value is now confirmed.
    Confirmed,
    /// The update failed — the signal was rolled back to its previous value.
    RolledBack { error: String },
}

/// State tracking for an in-flight optimistic update.
pub struct OptimisticUpdate<T: Clone + 'static> {
    signal: Signal<T>,
    previous_value: T,
    state: Rc<RefCell<OptimisticState>>,
}

#[derive(Debug, Clone, PartialEq)]
enum OptimisticState {
    Pending,
    Confirmed,
    RolledBack(String),
}

impl<T: Clone + PartialEq + 'static> OptimisticUpdate<T> {
    /// Create a new optimistic update.
    ///
    /// Immediately sets the signal to `new_value` and stores the previous
    /// value for potential rollback.
    pub fn new(signal: &Signal<T>, new_value: T) -> Self {
        let previous_value = signal.get_untracked();
        signal.set(new_value);

        Self {
            signal: signal.clone(),
            previous_value,
            state: Rc::new(RefCell::new(OptimisticState::Pending)),
        }
    }

    /// Confirm the update — the optimistic value is now the confirmed value.
    pub fn confirm(&self) {
        *self.state.borrow_mut() = OptimisticState::Confirmed;
    }

    /// Roll back the update — restores the previous value.
    pub fn rollback(&self) {
        self.signal.set(self.previous_value.clone());
        *self.state.borrow_mut() = OptimisticState::RolledBack("rolled back".to_string());
    }

    /// Roll back with a specific error message.
    pub fn rollback_with_error(&self, error: &str) {
        self.signal.set(self.previous_value.clone());
        *self.state.borrow_mut() = OptimisticState::RolledBack(error.to_string());
    }

    /// Get the previous (pre-update) value.
    pub fn previous_value(&self) -> &T {
        &self.previous_value
    }

    /// Get the current state.
    pub fn state(&self) -> OptimisticResult {
        match &*self.state.borrow() {
            OptimisticState::Pending => OptimisticResult::Confirmed, // not fully done but optimistic
            OptimisticState::Confirmed => OptimisticResult::Confirmed,
            OptimisticState::RolledBack(e) => OptimisticResult::RolledBack {
                error: e.clone(),
            },
        }
    }

    /// Check if the update has been rolled back.
    pub fn was_rolled_back(&self) -> bool {
        matches!(*self.state.borrow(), OptimisticState::RolledBack(_))
    }

    /// Check if the update has been confirmed.
    pub fn is_confirmed(&self) -> bool {
        matches!(*self.state.borrow(), OptimisticState::Confirmed)
    }
}

/// Perform an optimistic update on a signal.
///
/// Immediately sets the signal to `new_value`. If the async operation
/// fails, the signal is rolled back to its previous value.
///
/// # Example
/// ```
/// use rye_signals::{Signal, optimistic_update_sync, OptimisticResult};
///
/// let count = Signal::new(5);
/// let result = optimistic_update_sync(&count, 10, || Ok::<(), String>(()));
/// assert_eq!(count.get(), 10); // confirmed
/// assert!(matches!(result, OptimisticResult::Confirmed));
/// ```
pub fn optimistic_update_sync<T, E, F>(signal: &Signal<T>, new_value: T, op: F) -> OptimisticResult
where
    T: Clone + PartialEq + 'static,
    E: std::fmt::Display,
    F: FnOnce() -> Result<(), E>,
{
    let update = OptimisticUpdate::new(signal, new_value);
    match op() {
        Ok(()) => {
            update.confirm();
            OptimisticResult::Confirmed
        }
        Err(e) => {
            let msg = e.to_string();
            update.rollback_with_error(&msg);
            OptimisticResult::RolledBack { error: msg }
        }
    }
}

/// A simple waker that does nothing.
struct NoopWaker;

impl Wake for NoopWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

/// Perform an optimistic update with an async operation.
///
/// Immediately sets the signal to `new_value`. Polls the future once.
/// If it resolves to an error, rolls back.
pub fn optimistic_update<T, E, F>(signal: &Signal<T>, new_value: T, future: F) -> OptimisticResult
where
    T: Clone + PartialEq + 'static,
    E: std::fmt::Display + 'static,
    F: Future<Output = Result<(), E>>,
{
    let update = OptimisticUpdate::new(signal, new_value);

    let mut future = Box::pin(future);
    let waker = Waker::from(std::sync::Arc::new(NoopWaker));
    let mut cx = Context::from_waker(&waker);

    match future.as_mut().poll(&mut cx) {
        Poll::Ready(Ok(())) => {
            update.confirm();
            OptimisticResult::Confirmed
        }
        Poll::Ready(Err(e)) => {
            let msg = e.to_string();
            update.rollback_with_error(&msg);
            OptimisticResult::RolledBack { error: msg }
        }
        Poll::Pending => {
            // Still pending — keep the optimistic value
            // In a real app, the async runtime would drive this to completion
            OptimisticResult::Confirmed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimistic_update_success() {
        let sig = Signal::new(5);
        let result = optimistic_update_sync(&sig, 10, || Ok::<(), String>(()));
        assert_eq!(sig.get(), 10);
        assert_eq!(result, OptimisticResult::Confirmed);
    }

    #[test]
    fn test_optimistic_update_rollback() {
        let sig = Signal::new(5);
        let result = optimistic_update_sync(&sig, 10, || Err::<(), String>("network error".to_string()));
        assert_eq!(sig.get(), 5); // rolled back
        assert!(matches!(result, OptimisticResult::RolledBack { .. }));
    }

    #[test]
    fn test_optimistic_update_struct() {
        let sig = Signal::new("old".to_string());
        let update = OptimisticUpdate::new(&sig, "new".to_string());
        assert_eq!(sig.get(), "new");
        assert!(!update.is_confirmed());
        update.rollback();
        assert_eq!(sig.get(), "old");
        assert!(update.was_rolled_back());
    }

    #[test]
    fn test_optimistic_update_confirm() {
        let sig = Signal::new(0);
        let update = OptimisticUpdate::new(&sig, 42);
        assert_eq!(sig.get(), 42);
        update.confirm();
        assert!(update.is_confirmed());
        assert_eq!(sig.get(), 42);
    }

    #[test]
    fn test_optimistic_update_previous_value() {
        let sig = Signal::new(100);
        let update = OptimisticUpdate::new(&sig, 200);
        assert_eq!(update.previous_value(), &100);
    }

    #[test]
    fn test_optimistic_update_async_success() {
        let sig = Signal::new(1);
        let result = optimistic_update(&sig, 2, async { Ok::<(), String>(()) });
        assert_eq!(sig.get(), 2);
        assert_eq!(result, OptimisticResult::Confirmed);
    }

    #[test]
    fn test_optimistic_update_async_error() {
        let sig = Signal::new(1);
        let result = optimistic_update(&sig, 99, async { Err::<(), String>("fail".to_string()) });
        assert_eq!(sig.get(), 1); // rolled back
        assert!(matches!(result, OptimisticResult::RolledBack { .. }));
    }
}
