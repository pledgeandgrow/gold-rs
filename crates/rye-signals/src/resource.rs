//! Resource — async data with reactive state tracking.

use crate::runtime;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use std::task::{Context, Poll, Wake, Waker};

/// The state of an async resource.
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceState<T: Clone> {
    /// The resource is loading.
    Pending,
    /// The resource has resolved with a value.
    Ready(T),
    /// The resource failed with an error message.
    Error(String),
}

impl<T: Clone> Default for ResourceState<T> {
    fn default() -> Self {
        ResourceState::Pending
    }
}

/// A reactive async resource — wraps a Future and tracks its state.
///
/// # Example
/// ```
/// use rye_signals::{Resource, ResourceState};
///
/// let data = Resource::ready(42);
/// assert_eq!(data.get(), ResourceState::Ready(42));
/// ```
pub struct Resource<T: Clone + 'static> {
    inner: Rc<RefCell<ResourceInner<T>>>,
    signal_id: usize,
}

struct ResourceInner<T: Clone> {
    state: ResourceState<T>,
}

/// A simple waker that does nothing — used for polling futures synchronously.
struct NoopWaker;

impl Wake for NoopWaker {
    fn wake(self: std::sync::Arc<Self>) {}
}

impl<T: Clone + 'static> Resource<T> {
    /// Create a new resource from a future. The future is polled
    /// synchronously (once) to check if it's immediately ready.
    /// In a real app with an async runtime, the runtime drives the future.
    pub fn new<F: Future<Output = T> + 'static>(future: F) -> Self {
        let signal_id = runtime::next_id();
        let inner = Rc::new(RefCell::new(ResourceInner {
            state: ResourceState::Pending,
        }));

        let inner_clone = Rc::clone(&inner);
        let signal_id_clone = signal_id;

        let mut future = Box::pin(future);

        let waker = Waker::from(std::sync::Arc::new(NoopWaker));
        let mut cx = Context::from_waker(&waker);

        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => {
                inner_clone.borrow_mut().state = ResourceState::Ready(value);
                runtime::notify(signal_id_clone);
            }
            Poll::Pending => {
                inner_clone.borrow_mut().state = ResourceState::Pending;
            }
        }

        Self { inner, signal_id }
    }

    /// Create a resource that is immediately ready with a value.
    pub fn ready(value: T) -> Self {
        let signal_id = runtime::next_id();
        Self {
            inner: Rc::new(RefCell::new(ResourceInner {
                state: ResourceState::Ready(value),
            })),
            signal_id,
        }
    }

    /// Create a resource that is in an error state.
    pub fn error(msg: impl Into<String>) -> Self {
        let signal_id = runtime::next_id();
        Self {
            inner: Rc::new(RefCell::new(ResourceInner {
                state: ResourceState::Error(msg.into()),
            })),
            signal_id,
        }
    }

    /// Get the current state (tracked — re-runs when state changes).
    pub fn get(&self) -> ResourceState<T> {
        runtime::track(self.signal_id);
        self.inner.borrow().state.clone()
    }

    /// Get the current state (untracked).
    pub fn get_untracked(&self) -> ResourceState<T> {
        self.inner.borrow().state.clone()
    }

    /// Check if the resource is pending.
    pub fn is_pending(&self) -> bool {
        matches!(self.get_untracked(), ResourceState::Pending)
    }

    /// Check if the resource is ready.
    pub fn is_ready(&self) -> bool {
        matches!(self.get_untracked(), ResourceState::Ready(_))
    }

    /// Check if the resource errored.
    pub fn is_error(&self) -> bool {
        matches!(self.get_untracked(), ResourceState::Error(_))
    }

    /// Get the value if ready, None otherwise.
    pub fn value(&self) -> Option<T> {
        match self.get_untracked() {
            ResourceState::Ready(v) => Some(v),
            _ => None,
        }
    }

    /// Update the state and notify subscribers.
    pub fn set_state(&self, state: ResourceState<T>) {
        self.inner.borrow_mut().state = state;
        runtime::notify(self.signal_id);
    }
}

impl<T: Clone + 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            signal_id: self.signal_id,
        }
    }
}
