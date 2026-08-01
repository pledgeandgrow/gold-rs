//! Router component — manages navigation and renders matched routes.

/// The router component. Listens to navigation events and renders
/// the matched route's component.
pub struct Router {
    // TODO: route table, current route, history
}

impl Router {
    /// Create a new router with the given route table.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
