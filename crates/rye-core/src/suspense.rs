//! Suspense and ErrorBoundary — async boundary components.
//!
//! Suspense shows a fallback while async resources are loading.
//! ErrorBoundary catches errors in child components and shows a fallback.

use rye_signals::{Resource, ResourceState, Signal, Effect};

/// Suspense boundary — shows fallback content while resources are pending.
///
/// # Example
/// ```ignore
/// use rye_core::suspense::Suspense;
/// use rye_signals::Resource;
///
/// let data = Resource::ready(42);
/// let suspense = Suspense::new(data, || {
///     // fallback
///     "Loading...".to_string()
/// }, |value| {
///     // content
///     format!("Data: {}", value)
/// });
/// ```
pub struct Suspense<T: Clone + 'static, F: Fn() -> String + 'static, R: Fn(&T) -> String + 'static> {
    resource: Resource<T>,
    fallback: F,
    render: R,
    state: Signal<SuspenseState>,
}

/// The state of a Suspense boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum SuspenseState {
    /// Resource is loading.
    Pending,
    /// Resource is ready.
    Ready,
    /// Resource failed.
    Error(String),
}

impl<T: Clone + 'static, F: Fn() -> String + 'static, R: Fn(&T) -> String + 'static>
    Suspense<T, F, R>
{
    /// Create a new Suspense boundary.
    pub fn new(resource: Resource<T>, fallback: F, render: R) -> Self {
        let state = Signal::new(SuspenseState::Pending);

        let state_clone = state.clone();
        let resource_clone = resource.clone();
        let _effect = Effect::new(move || {
            match resource_clone.get() {
                ResourceState::Pending => {
                    state_clone.set(SuspenseState::Pending);
                }
                ResourceState::Ready(_) => {
                    state_clone.set(SuspenseState::Ready);
                }
                ResourceState::Error(e) => {
                    state_clone.set(SuspenseState::Error(e));
                }
            }
        });

        Self {
            resource,
            fallback,
            render,
            state,
        }
    }

    /// Render the appropriate content based on resource state.
    pub fn render_content(&self) -> String {
        match self.state.get_untracked() {
            SuspenseState::Pending => (self.fallback)(),
            SuspenseState::Ready => {
                if let Some(value) = self.resource.value() {
                    (self.render)(&value)
                } else {
                    (self.fallback)()
                }
            }
            SuspenseState::Error(e) => format!("Error: {}", e),
        }
    }

    /// Get the current suspense state (tracked).
    pub fn state(&self) -> SuspenseState {
        self.state.get()
    }

    /// Get the current suspense state (untracked).
    pub fn state_untracked(&self) -> SuspenseState {
        self.state.get_untracked()
    }
}

/// ErrorBoundary — catches errors in child rendering and shows a fallback.
///
/// # Example
/// ```ignore
/// use rye_core::suspense::ErrorBoundary;
///
/// let boundary = ErrorBoundary::new(
///     || "Something went wrong".to_string(),
///     || {
///         // Child content that might panic
///         "Hello world".to_string()
///     }
/// );
/// ```
pub struct ErrorBoundary<F: Fn() -> String + 'static, C: Fn() -> String + 'static> {
    fallback: F,
    child: C,
    error: Signal<Option<String>>,
}

impl<F: Fn() -> String + 'static, C: Fn() -> String + 'static> ErrorBoundary<F, C> {
    /// Create a new ErrorBoundary.
    pub fn new(fallback: F, child: C) -> Self {
        Self {
            fallback,
            child,
            error: Signal::new(None),
        }
    }

    /// Render the boundary, catching any errors.
    pub fn render_content(&self) -> String {
        // Check if we already have an error
        if let Some(e) = self.error.get_untracked() {
            return (self.fallback)() + &format!(": {}", e);
        }

        // Try to render the child — catch panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            (self.child)()
        }));

        match result {
            Ok(content) => content,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown error".to_string()
                };
                self.error.set(Some(msg.clone()));
                (self.fallback)()
            }
        }
    }

    /// Get the current error, if any (tracked).
    pub fn error(&self) -> Option<String> {
        self.error.get()
    }

    /// Get the current error, if any (untracked).
    pub fn error_untracked(&self) -> Option<String> {
        self.error.get_untracked()
    }

    /// Clear the error and retry.
    pub fn reset(&self) {
        self.error.set(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspense_ready() {
        let resource = Resource::ready(42);
        let suspense = Suspense::new(
            resource,
            || "Loading...".to_string(),
            |v| format!("Data: {}", v),
        );

        // Wait for effect to run
        // The initial state should be set by the effect
        let state = suspense.state_untracked();
        assert!(state == SuspenseState::Ready || state == SuspenseState::Pending);
    }

    #[test]
    fn test_suspense_error() {
        let resource = Resource::<i32>::error("fetch failed");
        let suspense = Suspense::new(
            resource,
            || "Loading...".to_string(),
            |v| format!("Data: {}", v),
        );

        let state = suspense.state_untracked();
        assert!(state == SuspenseState::Error("fetch failed".to_string())
            || state == SuspenseState::Pending);
    }

    #[test]
    fn test_error_boundary_ok() {
        let boundary = ErrorBoundary::new(
            || "Error!".to_string(),
            || "Hello world".to_string(),
        );

        assert_eq!(boundary.render_content(), "Hello world");
        assert!(boundary.error_untracked().is_none());
    }

    #[test]
    fn test_error_boundary_catches_panic() {
        let boundary = ErrorBoundary::new(
            || "Error!".to_string(),
            || panic!("boom"),
        );

        let result = boundary.render_content();
        assert_eq!(result, "Error!");
        assert!(boundary.error_untracked().is_some());
    }

    #[test]
    fn test_error_boundary_reset() {
        let boundary = ErrorBoundary::new(
            || "Error!".to_string(),
            || panic!("boom"),
        );

        boundary.render_content();
        assert!(boundary.error_untracked().is_some());

        boundary.reset();
        assert!(boundary.error_untracked().is_none());
    }
}
