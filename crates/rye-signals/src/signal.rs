//! Signal — the core reactive primitive.

use crate::runtime::{self, SignalId};
use std::cell::RefCell;
use std::rc::Rc;

/// A reactive value that tracks reads and notifies writes.
///
/// Reading a signal inside a tracking scope (Effect, Memo, template binding)
/// automatically registers a dependency. Writing to a signal notifies all
/// subscribers.
///
/// # Example
/// ```
/// use rye_signals::Signal;
///
/// let count = Signal::new(0);
/// assert_eq!(count.get(), 0);
/// count.set(5);
/// assert_eq!(count.get(), 5);
/// ```
pub struct Signal<T: 'static> {
    inner: Rc<SignalInner<T>>,
}

impl<T: 'static> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.inner.id)
            .finish()
    }
}

struct SignalInner<T> {
    id: SignalId,
    value: RefCell<T>,
}

impl<T: Clone + 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(SignalInner {
                id: runtime::next_id(),
                value: RefCell::new(value),
            }),
        }
    }

    /// Read the current value. If called inside a tracking scope,
    /// registers a dependency.
    pub fn get(&self) -> T {
        runtime::track(self.inner.id);
        self.inner.value.borrow().clone()
    }

    /// Read the current value without registering a dependency.
    pub fn get_untracked(&self) -> T {
        self.inner.value.borrow().clone()
    }

    /// Set a new value and notify all subscribers.
    pub fn set(&self, value: T) {
        {
            *self.inner.value.borrow_mut() = value;
        }
        runtime::notify(self.inner.id);
    }

    /// Set a new value without notifying subscribers.
    pub fn set_untracked(&self, value: T) {
        *self.inner.value.borrow_mut() = value;
    }

    /// Functional update — apply a closure to the current value.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        {
            let mut val = self.inner.value.borrow_mut();
            f(&mut val);
        }
        runtime::notify(self.inner.id);
    }

    /// Functional update without notifying subscribers.
    pub fn update_untracked<F: FnOnce(&mut T)>(&self, f: F) {
        let mut val = self.inner.value.borrow_mut();
        f(&mut val);
    }

    /// Run a closure with a reference to the current value (read-only, tracked).
    pub fn with_value<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        runtime::track(self.inner.id);
        let val = self.inner.value.borrow();
        f(&val)
    }

    /// Get a read-only handle that can be cloned and passed around.
    pub fn read_only(&self) -> ReadSignal<T> {
        ReadSignal {
            inner: Rc::clone(&self.inner),
        }
    }

    /// Get a write-only handle that can be cloned and passed around.
    pub fn write_only(&self) -> WriteSignal<T> {
        WriteSignal {
            inner: Rc::clone(&self.inner),
        }
    }

    /// Get the signal's unique ID.
    pub fn id(&self) -> SignalId {
        self.inner.id
    }
}

/// A read-only handle to a Signal. Can be cloned and passed to child components.
pub struct ReadSignal<T: 'static> {
    inner: Rc<SignalInner<T>>,
}

impl<T: Clone + 'static> ReadSignal<T> {
    /// Read the current value (tracked).
    pub fn get(&self) -> T {
        runtime::track(self.inner.id);
        self.inner.value.borrow().clone()
    }

    /// Read the current value (untracked).
    pub fn get_untracked(&self) -> T {
        self.inner.value.borrow().clone()
    }
}

impl<T: Clone + 'static> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// A write-only handle to a Signal. Can be cloned and passed to child components.
pub struct WriteSignal<T: 'static> {
    inner: Rc<SignalInner<T>>,
}

impl<T: Clone + 'static> WriteSignal<T> {
    /// Set a new value and notify subscribers.
    pub fn set(&self, value: T) {
        {
            *self.inner.value.borrow_mut() = value;
        }
        runtime::notify(self.inner.id);
    }

    /// Functional update.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        {
            let mut val = self.inner.value.borrow_mut();
            f(&mut val);
        }
        runtime::notify(self.inner.id);
    }
}

impl<T: Clone + 'static> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: Clone + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Convenience method — shorthand for `.get()`.
impl<T: Clone + 'static> Signal<T> {
    /// Read via call syntax helper — shorthand for `.get()`.
    pub fn call(&self) -> T {
        self.get()
    }
}

// Operator overloads for ergonomic signal usage
impl<T: Clone + std::ops::AddAssign + 'static> std::ops::AddAssign<T> for Signal<T> {
    fn add_assign(&mut self, rhs: T) {
        self.update(|v| *v += rhs);
    }
}

impl<T: Clone + std::ops::SubAssign + 'static> std::ops::SubAssign<T> for Signal<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.update(|v| *v -= rhs);
    }
}
