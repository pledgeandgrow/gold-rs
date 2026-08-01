//! # rye-i18n
//!
//! Internationalization for rye — compile-time extraction, reactive locale, CLDR pluralization.

pub mod locale;
pub mod messages;
pub mod plural;

pub use locale::Locale;
