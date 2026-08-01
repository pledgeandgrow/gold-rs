//! Priority batching — extend `batch()` with priority levels.
//!
//! High-priority updates flush first. Useful for keeping critical UI
//! responsive while processing background data.

use crate::runtime::{self, SignalId};
use std::cell::RefCell;
use std::collections::HashMap;

/// Priority level for batched updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Low priority — background data, non-critical updates.
    Low = 0,
    /// Normal priority — default for most updates.
    Normal = 1,
    /// High priority — critical UI updates that must flush first.
    High = 2,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::High => write!(f, "high"),
            Priority::Normal => write!(f, "normal"),
            Priority::Low => write!(f, "low"),
        }
    }
}

thread_local! {
    /// Priority batch state — tracks dirty signals with their priorities.
    static PRIORITY_BATCH: RefCell<PriorityBatchState> = RefCell::new(PriorityBatchState::new());

    /// Signal priority registry — maps signal ID to its priority.
    static SIGNAL_PRIORITIES: RefCell<HashMap<SignalId, Priority>> = RefCell::new(HashMap::new());
}

struct PriorityBatchState {
    batching: bool,
    dirty: Vec<(SignalId, Priority)>,
}

impl PriorityBatchState {
    const fn new() -> Self {
        Self {
            batching: false,
            dirty: Vec::new(),
        }
    }
}

/// Set the priority for a signal.
pub fn set_signal_priority(signal_id: SignalId, priority: Priority) {
    SIGNAL_PRIORITIES.with(|p| {
        p.borrow_mut().insert(signal_id, priority);
    });
}

/// Get the priority for a signal (defaults to Normal).
pub fn get_signal_priority(signal_id: SignalId) -> Priority {
    SIGNAL_PRIORITIES.with(|p| {
        p.borrow()
            .get(&signal_id)
            .copied()
            .unwrap_or(Priority::Normal)
    })
}

/// Clear all signal priority registrations.
pub fn clear_priorities() {
    SIGNAL_PRIORITIES.with(|p| p.borrow_mut().clear());
}

/// Start a priority batch.
pub fn start_priority_batch() {
    PRIORITY_BATCH.with(|b| {
        b.borrow_mut().batching = true;
    });
}

/// End a priority batch — flushes dirty signals in priority order
/// (High first, then Normal, then Low).
pub fn end_priority_batch() {
    let mut dirty = PRIORITY_BATCH.with(|b| {
        let mut state = b.borrow_mut();
        state.batching = false;
        std::mem::take(&mut state.dirty)
    });

    // Sort by priority descending (High first)
    dirty.sort_by(|a, b| b.1.cmp(&a.1));

    for (signal_id, _) in dirty {
        run_subscribers(signal_id);
    }
}

/// Notify a signal with priority. If batching, queues with priority.
/// If not batching, runs immediately.
pub fn notify_with_priority(signal_id: SignalId) {
    let priority = get_signal_priority(signal_id);

    if PRIORITY_BATCH.with(|b| b.borrow().batching) {
        PRIORITY_BATCH.with(|b| {
            let mut state = b.borrow_mut();
            // Update priority if already queued (take the higher one)
            if let Some(entry) = state.dirty.iter_mut().find(|(id, _)| *id == signal_id) {
                if priority > entry.1 {
                    entry.1 = priority;
                }
            } else {
                state.dirty.push((signal_id, priority));
            }
        });
    } else {
        run_subscribers(signal_id);
    }
}

/// Run subscriber callbacks for a signal.
fn run_subscribers(signal_id: SignalId) {
    // Delegate to the runtime's subscriber runner
    // We need to call the runtime's notify which runs subscribers
    // But since we're outside the normal batch, we call run_subscribers directly
    // by using notify (which checks batching — but we're not batching here)
    runtime::notify(signal_id);
}

/// Execute a closure with high-priority batching.
///
/// High-priority signal updates within the closure are flushed first.
pub fn batch_high<F: FnOnce() -> R, R>(f: F) -> R {
    let was_batching = PRIORITY_BATCH.with(|b| b.borrow().batching);
    if !was_batching {
        start_priority_batch();
    }
    let result = f();
    if !was_batching {
        end_priority_batch();
    }
    result
}

/// Execute a closure with normal-priority batching (same as regular batch).
pub fn batch_normal<F: FnOnce() -> R, R>(f: F) -> R {
    batch_high(f)
}

/// Execute a closure with low-priority batching.
///
/// Low-priority updates are still flushed in order, but after any
/// high-priority updates in the same batch.
pub fn batch_low<F: FnOnce() -> R, R>(f: F) -> R {
    batch_high(f)
}

/// Check if currently in a priority batch.
pub fn is_priority_batching() -> bool {
    PRIORITY_BATCH.with(|b| b.borrow().batching)
}

/// Get the number of pending dirty signals in the current batch.
pub fn pending_count() -> usize {
    PRIORITY_BATCH.with(|b| b.borrow().dirty.len())
}

/// Reset all priority batch state.
pub fn reset() {
    PRIORITY_BATCH.with(|b| {
        b.borrow_mut().batching = false;
        b.borrow_mut().dirty.clear();
    });
    clear_priorities();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signal;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_set_get_signal_priority() {
        clear_priorities();
        let sig = Signal::new(42);
        let id = sig.id();
        assert_eq!(get_signal_priority(id), Priority::Normal);
        set_signal_priority(id, Priority::High);
        assert_eq!(get_signal_priority(id), Priority::High);
        clear_priorities();
        assert_eq!(get_signal_priority(id), Priority::Normal);
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::High.to_string(), "high");
        assert_eq!(Priority::Normal.to_string(), "normal");
        assert_eq!(Priority::Low.to_string(), "low");
    }

    #[test]
    fn test_batch_high_executes() {
        reset();
        let sig = Signal::new(0);
        batch_high(|| {
            sig.set(10);
        });
        assert_eq!(sig.get_untracked(), 10);
    }

    #[test]
    fn test_batch_normal_executes() {
        reset();
        let sig = Signal::new(0);
        batch_normal(|| {
            sig.set(5);
        });
        assert_eq!(sig.get_untracked(), 5);
    }

    #[test]
    fn test_batch_low_executes() {
        reset();
        let sig = Signal::new(0);
        batch_low(|| {
            sig.set(99);
        });
        assert_eq!(sig.get_untracked(), 99);
    }

    #[test]
    fn test_is_priority_batching() {
        reset();
        assert!(!is_priority_batching());
        start_priority_batch();
        assert!(is_priority_batching());
        end_priority_batch();
        assert!(!is_priority_batching());
    }

    #[test]
    fn test_pending_count() {
        reset();
        start_priority_batch();
        assert_eq!(pending_count(), 0);
        end_priority_batch();
    }

    #[test]
    fn test_notify_with_priority_outside_batch() {
        reset();
        let sig = Signal::new(0);
        // Outside batch — should run immediately
        notify_with_priority(sig.id());
    }

    #[test]
    fn test_default_priority() {
        let p: Priority = Default::default();
        assert_eq!(p, Priority::Normal);
    }
}
