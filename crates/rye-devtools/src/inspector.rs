//! Inspector — component tree inspector for devtools.

/// Component tree inspector — shows the hierarchy of mounted components.
pub struct Inspector {
    // TODO: component tree snapshot, selected component, props viewer
}

impl Inspector {
    /// Create a new inspector.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}
