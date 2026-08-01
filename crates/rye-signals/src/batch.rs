//! Batch — group multiple signal writes into a single update.

use crate::runtime;

/// Execute a closure with batching enabled. All signal writes within
/// the closure are collected and subscribers are notified once after
/// the closure completes.
///
/// # Example
/// ```
/// use rye_signals::{Signal, batch};
///
/// let a = Signal::new(0);
/// let b = Signal::new(0);
///
/// batch(|| {
///     a.set(1);
///     b.set(2);
///     // Subscribers are notified once, not twice
/// });
/// ```
pub fn batch<F: FnOnce() -> R, R>(f: F) -> R {
    let was_batching = runtime::is_batching();
    if !was_batching {
        runtime::start_batch();
    }
    let result = f();
    if !was_batching {
        runtime::end_batch();
    }
    result
}

/// Check if we are currently inside a batch.
pub fn is_batching() -> bool {
    runtime::is_batching()
}
