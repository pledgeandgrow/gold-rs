//! Debounced and throttled computed signals.
//!
//! `Memo::debounced(duration)` and `Memo::throttled(duration)` for
//! derived state that shouldn't update on every dependency change.

use crate::runtime;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// A debounced signal — only updates after `delay` has elapsed
/// without any source signal changes.
///
/// In a synchronous runtime, the debounce is checked on read.
/// In a real app with a timer, the update fires after the delay.
pub struct Debounced<T: Clone + 'static> {
    inner: Rc<RefCell<DebouncedInner<T>>>,
    signal_id: runtime::SignalId,
}

struct DebouncedInner<T> {
    source_value: T,
    output_value: T,
    last_change: Instant,
    delay: Duration,
    compute: Box<dyn Fn() -> T>,
}

impl<T: Clone + PartialEq + 'static> Debounced<T> {
    /// Create a new debounced signal.
    ///
    /// The output only updates when `delay` has elapsed since the last
    /// source change without any new changes.
    pub fn new<F: Fn() -> T + 'static>(compute: F, delay: Duration) -> Self {
        let signal_id = runtime::next_id();

        // Initial computation
        let initial = compute();
        let inner = Rc::new(RefCell::new(DebouncedInner {
            source_value: initial.clone(),
            output_value: initial,
            last_change: Instant::now(),
            delay,
            compute: Box::new(compute),
        }));

        Self { inner, signal_id }
    }

    /// Read the debounced value (tracked).
    pub fn get(&self) -> T {
        self.check_update();
        runtime::track(self.signal_id);
        self.inner.borrow().output_value.clone()
    }

    /// Read the debounced value (untracked).
    pub fn get_untracked(&self) -> T {
        self.check_update();
        self.inner.borrow().output_value.clone()
    }

    /// Read the current source value (without debounce).
    pub fn source_value(&self) -> T {
        (self.inner.borrow().compute)()
    }

    /// Force an immediate flush of the debounced value.
    pub fn flush(&self) {
        let new_value = (self.inner.borrow().compute)();
        let mut inner = self.inner.borrow_mut();
        let changed = inner.output_value != new_value;
        inner.source_value = new_value.clone();
        inner.output_value = new_value;
        inner.last_change = Instant::now();
        drop(inner);
        if changed {
            runtime::notify(self.signal_id);
        }
    }

    fn check_update(&self) {
        let should_update = {
            let inner = self.inner.borrow();
            let new_source = (inner.compute)();
            let source_changed = new_source != inner.source_value;
            if source_changed {
                let elapsed = inner.last_change.elapsed();
                elapsed >= inner.delay
            } else {
                false
            }
        };

        if should_update {
            self.flush();
        } else {
            // Update source tracking even if we don't output
            let new_source = (self.inner.borrow().compute)();
            let mut inner = self.inner.borrow_mut();
            if inner.source_value != new_source {
                inner.source_value = new_source;
                inner.last_change = Instant::now();
            }
        }
    }
}

impl<T: Clone + PartialEq + 'static> Clone for Debounced<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            signal_id: self.signal_id,
        }
    }
}

/// A throttled signal — updates at most once per `interval`.
///
/// The first update is immediate. Subsequent updates are delayed
/// until the interval has elapsed.
pub struct Throttled<T: Clone + 'static> {
    inner: Rc<RefCell<ThrottledInner<T>>>,
    signal_id: runtime::SignalId,
}

struct ThrottledInner<T> {
    output_value: T,
    last_update: Instant,
    interval: Duration,
    compute: Box<dyn Fn() -> T>,
    pending: Option<T>,
}

impl<T: Clone + PartialEq + 'static> Throttled<T> {
    /// Create a new throttled signal.
    pub fn new<F: Fn() -> T + 'static>(compute: F, interval: Duration) -> Self {
        let signal_id = runtime::next_id();
        let initial = compute();

        let inner = Rc::new(RefCell::new(ThrottledInner {
            output_value: initial,
            last_update: Instant::now(),
            interval,
            compute: Box::new(compute),
            pending: None,
        }));

        Self { inner, signal_id }
    }

    /// Read the throttled value (tracked).
    pub fn get(&self) -> T {
        self.check_update();
        runtime::track(self.signal_id);
        self.inner.borrow().output_value.clone()
    }

    /// Read the throttled value (untracked).
    pub fn get_untracked(&self) -> T {
        self.check_update();
        self.inner.borrow().output_value.clone()
    }

    /// Read the current source value (without throttle).
    pub fn source_value(&self) -> T {
        (self.inner.borrow().compute)()
    }

    /// Force an immediate flush of pending updates.
    pub fn flush(&self) {
        let new_value = (self.inner.borrow().compute)();
        let mut inner = self.inner.borrow_mut();
        let changed = inner.output_value != new_value;
        inner.output_value = new_value;
        inner.last_update = Instant::now();
        inner.pending = None;
        drop(inner);
        if changed {
            runtime::notify(self.signal_id);
        }
    }

    fn check_update(&self) {
        let new_source = (self.inner.borrow().compute)();
        let should_update = {
            let inner = self.inner.borrow();
            if inner.output_value == new_source {
                false
            } else {
                inner.last_update.elapsed() >= inner.interval
            }
        };

        if should_update {
            self.flush();
        } else {
            let mut inner = self.inner.borrow_mut();
            if inner.output_value != new_source {
                inner.pending = Some(new_source);
            }
        }
    }
}

impl<T: Clone + PartialEq + 'static> Clone for Throttled<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            signal_id: self.signal_id,
        }
    }
}

/// Create a debounced signal from a compute closure.
pub fn debounced<T: Clone + PartialEq + 'static, F: Fn() -> T + 'static>(
    compute: F,
    delay: Duration,
) -> Debounced<T> {
    Debounced::new(compute, delay)
}

/// Create a throttled signal from a compute closure.
pub fn throttled<T: Clone + PartialEq + 'static, F: Fn() -> T + 'static>(
    compute: F,
    interval: Duration,
) -> Throttled<T> {
    Throttled::new(compute, interval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Signal;

    #[test]
    fn test_debounced_initial_value() {
        let sig = Signal::new(42);
        let sig_clone = sig.clone();
        let deb = debounced(move || sig_clone.get(), Duration::from_millis(100));
        assert_eq!(deb.get(), 42);
    }

    #[test]
    fn test_debounced_flush() {
        let sig = Signal::new(0);
        let sig_clone = sig.clone();
        let deb = debounced(move || sig_clone.get(), Duration::from_secs(10));
        sig.set(99);
        // Without flush, debounced value may be stale
        deb.flush();
        assert_eq!(deb.get(), 99);
    }

    #[test]
    fn test_debounced_source_value() {
        let sig = Signal::new(5);
        let sig_clone = sig.clone();
        let deb = debounced(move || sig_clone.get(), Duration::from_secs(10));
        sig.set(100);
        assert_eq!(deb.source_value(), 100);
    }

    #[test]
    fn test_debounced_immediate_update_after_delay() {
        let sig = Signal::new(0);
        let sig_clone = sig.clone();
        let deb = debounced(move || sig_clone.get(), Duration::from_millis(1));
        sig.set(50);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(deb.get(), 50);
    }

    #[test]
    fn test_throttled_initial_value() {
        let sig = Signal::new(42);
        let sig_clone = sig.clone();
        let thr = throttled(move || sig_clone.get(), Duration::from_millis(100));
        assert_eq!(thr.get(), 42);
    }

    #[test]
    fn test_throttled_flush() {
        let sig = Signal::new(0);
        let sig_clone = sig.clone();
        let thr = throttled(move || sig_clone.get(), Duration::from_secs(10));
        sig.set(77);
        thr.flush();
        assert_eq!(thr.get(), 77);
    }

    #[test]
    fn test_throttled_source_value() {
        let sig = Signal::new(5);
        let sig_clone = sig.clone();
        let thr = throttled(move || sig_clone.get(), Duration::from_secs(10));
        sig.set(100);
        assert_eq!(thr.source_value(), 100);
    }

    #[test]
    fn test_throttled_update_after_interval() {
        let sig = Signal::new(0);
        let sig_clone = sig.clone();
        let thr = throttled(move || sig_clone.get(), Duration::from_millis(1));
        sig.set(33);
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(thr.get(), 33);
    }

    #[test]
    fn test_debounced_clone() {
        let sig = Signal::new(10);
        let sig_clone = sig.clone();
        let deb = debounced(move || sig_clone.get(), Duration::from_millis(50));
        let deb2 = deb.clone();
        assert_eq!(deb.get(), deb2.get());
    }

    #[test]
    fn test_throttled_clone() {
        let sig = Signal::new(20);
        let sig_clone = sig.clone();
        let thr = throttled(move || sig_clone.get(), Duration::from_millis(50));
        let thr2 = thr.clone();
        assert_eq!(thr.get(), thr2.get());
    }
}
