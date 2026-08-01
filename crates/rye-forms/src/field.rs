//! Field — individual form field state.

/// A form field with value, dirty/pristine/touched state, and validation errors.
pub struct Field {
    // TODO: value signal, dirty, touched, errors
}

impl Field {
    /// Create a new field.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}
