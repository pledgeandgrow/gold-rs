//! Validation — sync and async field validation.

/// A validation rule that checks a value and returns an error message if invalid.
pub type ValidationRule<T> = Box<dyn Fn(&T) -> Option<String>>;

/// Validate a value against multiple rules.
pub fn validate<T>(value: &T, rules: &[ValidationRule<T>]) -> Vec<String> {
    rules.iter().filter_map(|rule| rule(value)).collect()
}
