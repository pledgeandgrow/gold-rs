//! Route definition and matching.

/// A route definition with a path pattern and associated component.
pub struct Route {
    /// The path pattern, e.g. `/users/:id`.
    pub path: &'static str,
    // TODO: component, children, guards
}

/// A matched route with extracted parameters.
pub struct RouteMatch {
    /// The matched route path.
    pub path: String,
    /// Extracted parameters from the path.
    pub params: Vec<(String, String)>,
}
