//! Locale — reactive locale management.

/// A locale identifier (e.g. "en", "fr", "ja").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale(String);

impl Locale {
    /// Create a new locale from a string.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Get the locale code.
    pub fn code(&self) -> &str {
        &self.0
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}
