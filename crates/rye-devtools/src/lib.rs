//! # rye-devtools
//!
//! Developer tools for rye — component inspector, signal viewer, profiler, render highlight.
//!
//! **STUB** — This crate is a placeholder. Types are defined but inspector,
//! profiler, and render highlight logic is not implemented.

#![deny(missing_docs)]

pub mod inspector;
pub mod profiler;

pub use inspector::Inspector;
pub use profiler::Profiler;
