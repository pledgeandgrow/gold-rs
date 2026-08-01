//! Pluralization — CLDR plural rules.

/// Plural categories as defined by CLDR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

/// Determine the plural category for a count and locale.
pub fn plural_category(count: f64, _locale: &str) -> PluralCategory {
    // TODO: implement CLDR plural rules per locale
    if count == 1.0 {
        PluralCategory::One
    } else {
        PluralCategory::Other
    }
}
