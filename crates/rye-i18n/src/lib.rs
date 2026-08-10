//! # rye-i18n
//!
//! Internationalization for rye — compile-time extraction, reactive locale, CLDR pluralization.
//!
//! **STUB** — This crate is a placeholder. Types are defined but message
//! formatting, plural rules, and locale switching are not implemented.

pub mod locale;
pub mod messages;
pub mod plural;

pub use locale::Locale;
