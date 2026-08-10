//! AI-native tooling modules — Goals 159–165.
//!
//! - **Goal 159**: AI prompt templates for common patterns
//! - **Goal 160**: AI context window optimization
//! - **Goal 162**: AI-friendly error recovery suggestions
//! - **Goal 163**: Component usage analytics
//! - **Goal 164**: AI code review integration
//! - **Goal 165**: Natural language component search

pub mod code_review;
pub mod context_optimizer;
pub mod error_recovery;
pub mod nl_search;
pub mod prompt_templates;
pub mod usage_analytics;

/// Shared mutex for tests that mutate the global `component_registry`.
///
/// Tests in `context_optimizer` and `nl_search` both call `clear()` +
/// `register()` on the shared global registry. Without serialization they
/// race when run in parallel, causing flaky failures. Tests that touch the
/// registry should acquire this lock for the duration of their setup +
/// assertions.
#[cfg(test)]
pub(crate) static REGISTRY_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
