//! Form — reactive form state management.

/// A reactive form with field tracking, validation, and submit handling.
pub struct Form {
    // TODO: fields, validation state, dirty/pristine/touched, submit handler
}

impl Form {
    /// Create a new empty form.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}
