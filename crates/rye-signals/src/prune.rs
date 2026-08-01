//! Signal graph pruning — automatically detach signals with no subscribers.
//!
//! When a signal has no subscribers, it is pruned from the dependency graph
//! to save memory. It re-attaches on the next read. Useful for long-running
//! apps with many transient signals.

use crate::runtime;
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    /// Set of pruned signal IDs — signals whose subscriber lists are empty.
    static PRUNED: RefCell<HashSet<runtime::SignalId>> = RefCell::new(HashSet::new());

    /// Set of manually pinned signal IDs — signals that should never be pruned.
    static PINNED: RefCell<HashSet<runtime::SignalId>> = RefCell::new(HashSet::new());

    /// Pruning enabled flag.
    static PRUNING_ENABLED: RefCell<bool> = RefCell::new(true);
}

/// Check if pruning is enabled.
pub fn is_pruning_enabled() -> bool {
    PRUNING_ENABLED.with(|e| *e.borrow())
}

/// Enable or disable pruning globally.
pub fn set_pruning_enabled(enabled: bool) {
    PRUNING_ENABLED.with(|e| *e.borrow_mut() = enabled);
}

/// Pin a signal so it is never pruned.
pub fn pin(signal_id: runtime::SignalId) {
    PINNED.with(|p| {
        p.borrow_mut().insert(signal_id);
    });
}

/// Unpin a signal.
pub fn unpin(signal_id: runtime::SignalId) {
    PINNED.with(|p| {
        p.borrow_mut().remove(&signal_id);
    });
}

/// Check if a signal is pruned.
pub fn is_pruned(signal_id: runtime::SignalId) -> bool {
    PRUNED.with(|p| p.borrow().contains(&signal_id))
}

/// Check if a signal is pinned.
pub fn is_pinned(signal_id: runtime::SignalId) -> bool {
    PINNED.with(|p| p.borrow().contains(&signal_id))
}

/// Attempt to prune signals that have no subscribers.
/// Returns the number of signals pruned.
///
/// This scans the subscriber map and marks signals with empty subscriber lists.
/// Pruned signals are tracked so they can be re-attached on next read.
pub fn prune() -> usize {
    if !is_pruning_enabled() {
        return 0;
    }

    let pinned_set = PINNED.with(|p| p.borrow().clone());

    let to_prune: Vec<runtime::SignalId> = runtime::get_signals_with_no_subscribers()
        .into_iter()
        .filter(|id| !pinned_set.contains(id))
        .collect();

    let count = to_prune.len();
    PRUNED.with(|p| {
        let mut p = p.borrow_mut();
        for id in &to_prune {
            p.insert(*id);
        }
    });

    // Remove empty subscriber entries
    for id in &to_prune {
        runtime::remove_subscriber_entry(*id);
    }

    count
}

/// Mark a signal as re-attached (called when a signal is read again).
pub fn reattach(signal_id: runtime::SignalId) {
    PRUNED.with(|p| {
        p.borrow_mut().remove(&signal_id);
    });
}

/// Get the number of currently pruned signals.
pub fn pruned_count() -> usize {
    PRUNED.with(|p| p.borrow().len())
}

/// Get all pruned signal IDs.
pub fn pruned_ids() -> Vec<runtime::SignalId> {
    PRUNED.with(|p| p.borrow().iter().copied().collect())
}

/// Clear all pruning state.
pub fn reset() {
    PRUNED.with(|p| p.borrow_mut().clear());
    PINNED.with(|p| p.borrow_mut().clear());
    PRUNING_ENABLED.with(|e| *e.borrow_mut() = true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signal;

    #[test]
    fn test_pin_and_unpin() {
        let sig = Signal::new(42);
        let id = sig.id();
        pin(id);
        assert!(is_pinned(id));
        unpin(id);
        assert!(!is_pinned(id));
    }

    #[test]
    fn test_pruning_enabled_disabled() {
        set_pruning_enabled(false);
        assert!(!is_pruning_enabled());
        set_pruning_enabled(true);
        assert!(is_pruning_enabled());
    }

    #[test]
    fn test_pruned_tracking() {
        reset();
        let sig = Signal::new(10);
        let id = sig.id();
        assert!(!is_pruned(id));
        assert_eq!(pruned_count(), 0);
    }

    #[test]
    fn test_reattach() {
        let sig = Signal::new(10);
        let id = sig.id();
        PRUNED.with(|p| {
            p.borrow_mut().insert(id);
        });
        assert!(is_pruned(id));
        reattach(id);
        assert!(!is_pruned(id));
    }

    #[test]
    fn test_reset() {
        let sig = Signal::new(10);
        let id = sig.id();
        pin(id);
        PRUNED.with(|p| {
            p.borrow_mut().insert(id);
        });
        reset();
        assert!(!is_pinned(id));
        assert_eq!(pruned_count(), 0);
    }
}
