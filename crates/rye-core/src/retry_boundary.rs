//! Error boundaries with retry strategies — extend ErrorBoundary with configurable retry.
//!
//! Extends existing `ErrorBoundary` (goal 39) with configurable retry:
//! exponential backoff, max retries, fallback to cached data, fallback to
//! static content. `ErrorBoundary::with_retry(strategy)`.

use rye_signals::Signal;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Retry strategy for an error boundary.
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// No retry — show fallback immediately.
    None,
    /// Fixed delay between retries, up to max_retries.
    Fixed {
        delay: Duration,
        max_retries: u32,
    },
    /// Exponential backoff — delay doubles each retry.
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        max_retries: u32,
    },
    /// Fallback to cached data on error.
    FallbackCached {
        max_retries: u32,
    },
    /// Fallback to static content on error.
    FallbackStatic {
        static_content: String,
        max_retries: u32,
    },
    /// Custom retry logic.
    Custom {
        delays: Vec<Duration>,
    },
}

impl RetryStrategy {
    /// Get the delay before the next retry (0-indexed attempt).
    pub fn delay_for(&self, attempt: u32) -> Option<Duration> {
        match self {
            RetryStrategy::None => None,
            RetryStrategy::Fixed { delay, max_retries } => {
                if attempt < *max_retries {
                    Some(*delay)
                } else {
                    None
                }
            }
            RetryStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                max_retries,
            } => {
                if attempt < *max_retries {
                    let delay_ms = initial_delay.as_millis() as u64 * (1u64 << attempt);
                    let capped = delay_ms.min(max_delay.as_millis() as u64);
                    Some(Duration::from_millis(capped))
                } else {
                    None
                }
            }
            RetryStrategy::FallbackCached { max_retries } => {
                if attempt < *max_retries {
                    Some(Duration::ZERO)
                } else {
                    None
                }
            }
            RetryStrategy::FallbackStatic { max_retries, .. } => {
                if attempt < *max_retries {
                    Some(Duration::ZERO)
                } else {
                    None
                }
            }
            RetryStrategy::Custom { delays } => {
                delays.get(attempt as usize).copied()
            }
        }
    }

    /// Get the maximum number of retries.
    pub fn max_retries(&self) -> u32 {
        match self {
            RetryStrategy::None => 0,
            RetryStrategy::Fixed { max_retries, .. } => *max_retries,
            RetryStrategy::ExponentialBackoff { max_retries, .. } => *max_retries,
            RetryStrategy::FallbackCached { max_retries } => *max_retries,
            RetryStrategy::FallbackStatic { max_retries, .. } => *max_retries,
            RetryStrategy::Custom { delays } => delays.len() as u32,
        }
    }

    /// Check if this strategy has a static fallback.
    pub fn static_fallback(&self) -> Option<&str> {
        match self {
            RetryStrategy::FallbackStatic { static_content, .. } => Some(static_content),
            _ => None,
        }
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            max_retries: 3,
        }
    }
}

/// The state of a retry boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum RetryState {
    /// No error — rendering normally.
    Ok,
    /// Error occurred, waiting to retry.
    Waiting { attempt: u32, error: String },
    /// All retries exhausted, showing fallback.
    Failed { error: String },
    /// Retrying now.
    Retrying { attempt: u32 },
}

/// An error boundary with configurable retry strategy.
pub struct RetryErrorBoundary {
    strategy: RetryStrategy,
    state: Signal<RetryState>,
    cached_content: RefCell<Option<String>>,
    retry_count: RefCell<u32>,
    /// Callback to run on each retry attempt.
    on_retry: Option<Rc<dyn Fn()>>,
}

impl RetryErrorBoundary {
    /// Create a new retry error boundary with the given strategy.
    pub fn new(strategy: RetryStrategy) -> Self {
        Self {
            strategy,
            state: Signal::new(RetryState::Ok),
            cached_content: RefCell::new(None),
            retry_count: RefCell::new(0),
            on_retry: None,
        }
    }

    /// Set a callback to run on each retry.
    pub fn on_retry<F: Fn() + 'static>(mut self, callback: F) -> Self {
        self.on_retry = Some(Rc::new(callback));
        self
    }

    /// Report an error. The boundary will schedule a retry based on its strategy.
    pub fn report_error(&self, error: &str) {
        let attempt = *self.retry_count.borrow();
        let max_retries = self.strategy.max_retries();

        if attempt < max_retries {
            if let Some(delay) = self.strategy.delay_for(attempt) {
                self.state.set(RetryState::Waiting {
                    attempt,
                    error: error.to_string(),
                });
                // In a real app, this would use setTimeout or tokio::time::sleep.
                // For synchronous testing, we just record the state.
                let _ = delay;
            } else {
                self.state.set(RetryState::Failed {
                    error: error.to_string(),
                });
            }
        } else {
            self.state.set(RetryState::Failed {
                error: error.to_string(),
            });
        }
    }

    /// Attempt a retry. Returns true if a retry was initiated.
    pub fn retry(&self) -> bool {
        let attempt = *self.retry_count.borrow();
        let max_retries = self.strategy.max_retries();

        if attempt >= max_retries {
            return false;
        }

        *self.retry_count.borrow_mut() += 1;
        self.state.set(RetryState::Retrying { attempt });

        if let Some(callback) = &self.on_retry {
            callback();
        }

        true
    }

    /// Mark the retry as successful — clear error state.
    pub fn succeed(&self) {
        *self.retry_count.borrow_mut() = 0;
        self.state.set(RetryState::Ok);
    }

    /// Mark the retry as failed — either schedule next retry or give up.
    pub fn fail(&self, error: &str) {
        let attempt = *self.retry_count.borrow();
        let max_retries = self.strategy.max_retries();

        if attempt < max_retries {
            if let Some(delay) = self.strategy.delay_for(attempt) {
                self.state.set(RetryState::Waiting {
                    attempt,
                    error: error.to_string(),
                });
                let _ = delay;
            } else {
                self.state.set(RetryState::Failed {
                    error: error.to_string(),
                });
            }
        } else {
            self.state.set(RetryState::Failed {
                error: error.to_string(),
            });
        }
    }

    /// Cache content for fallback.
    pub fn cache_content(&self, content: &str) {
        *self.cached_content.borrow_mut() = Some(content.to_string());
    }

    /// Get cached content (for FallbackCached strategy).
    pub fn cached_content(&self) -> Option<String> {
        self.cached_content.borrow().clone()
    }

    /// Get the current state (tracked).
    pub fn state(&self) -> RetryState {
        self.state.get()
    }

    /// Get the current state (untracked).
    pub fn state_untracked(&self) -> RetryState {
        self.state.get_untracked()
    }

    /// Get the current retry count.
    pub fn retry_count(&self) -> u32 {
        *self.retry_count.borrow()
    }

    /// Get the retry strategy.
    pub fn strategy(&self) -> &RetryStrategy {
        &self.strategy
    }

    /// Reset the boundary — clear errors and retry count.
    pub fn reset(&self) {
        *self.retry_count.borrow_mut() = 0;
        self.state.set(RetryState::Ok);
    }

    /// Render the appropriate content based on state.
    pub fn render(&self, render_fn: &dyn Fn() -> String, fallback_fn: &dyn Fn() -> String) -> String {
        match self.state.get_untracked() {
            RetryState::Ok => {
                let content = render_fn();
                self.cache_content(&content);
                content
            }
            RetryState::Waiting { error, .. } => {
                // Show fallback while waiting
                let _ = error;
                fallback_fn()
            }
            RetryState::Retrying { .. } => {
                // Show fallback while retrying
                fallback_fn()
            }
            RetryState::Failed { .. } => {
                // Try fallback strategies
                if let Some(cached) = self.cached_content() {
                    return cached;
                }
                if let Some(static_content) = self.strategy.static_fallback() {
                    return static_content.to_string();
                }
                fallback_fn()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_retry_strategy_none() {
        let strategy = RetryStrategy::None;
        assert_eq!(strategy.max_retries(), 0);
        assert!(strategy.delay_for(0).is_none());
    }

    #[test]
    fn test_retry_strategy_fixed() {
        let strategy = RetryStrategy::Fixed {
            delay: Duration::from_millis(100),
            max_retries: 3,
        };
        assert_eq!(strategy.max_retries(), 3);
        assert_eq!(strategy.delay_for(0), Some(Duration::from_millis(100)));
        assert_eq!(strategy.delay_for(2), Some(Duration::from_millis(100)));
        assert!(strategy.delay_for(3).is_none());
    }

    #[test]
    fn test_retry_strategy_exponential() {
        let strategy = RetryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            max_retries: 5,
        };
        assert_eq!(strategy.delay_for(0), Some(Duration::from_millis(100)));
        assert_eq!(strategy.delay_for(1), Some(Duration::from_millis(200)));
        assert_eq!(strategy.delay_for(2), Some(Duration::from_millis(400)));
        assert_eq!(strategy.delay_for(3), Some(Duration::from_millis(800)));
        assert!(strategy.delay_for(5).is_none());
    }

    #[test]
    fn test_retry_strategy_exponential_capped() {
        let strategy = RetryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_millis(500),
            max_retries: 10,
        };
        assert_eq!(strategy.delay_for(0), Some(Duration::from_millis(100)));
        assert_eq!(strategy.delay_for(3), Some(Duration::from_millis(500)));
        assert_eq!(strategy.delay_for(9), Some(Duration::from_millis(500)));
        assert!(strategy.delay_for(10).is_none());
    }

    #[test]
    fn test_retry_strategy_custom() {
        let strategy = RetryStrategy::Custom {
            delays: vec![Duration::from_millis(10), Duration::from_millis(50)],
        };
        assert_eq!(strategy.max_retries(), 2);
        assert_eq!(strategy.delay_for(0), Some(Duration::from_millis(10)));
        assert_eq!(strategy.delay_for(1), Some(Duration::from_millis(50)));
        assert!(strategy.delay_for(2).is_none());
    }

    #[test]
    fn test_retry_strategy_fallback_static() {
        let strategy = RetryStrategy::FallbackStatic {
            static_content: "<div>Static</div>".to_string(),
            max_retries: 2,
        };
        assert_eq!(strategy.static_fallback(), Some("<div>Static</div>"));
    }

    #[test]
    fn test_retry_boundary_initial_state() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::default());
        assert_eq!(boundary.state_untracked(), RetryState::Ok);
        assert_eq!(boundary.retry_count(), 0);
    }

    #[test]
    fn test_retry_boundary_report_error() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::Fixed {
            delay: Duration::from_millis(100),
            max_retries: 3,
        });
        boundary.report_error("fetch failed");
        assert!(matches!(
            boundary.state_untracked(),
            RetryState::Waiting { .. }
        ));
    }

    #[test]
    fn test_retry_boundary_retry_and_succeed() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::Fixed {
            delay: Duration::from_millis(10),
            max_retries: 3,
        });
        boundary.report_error("error");
        assert!(boundary.retry());
        assert!(matches!(boundary.state_untracked(), RetryState::Retrying { .. }));
        boundary.succeed();
        assert_eq!(boundary.state_untracked(), RetryState::Ok);
        assert_eq!(boundary.retry_count(), 0);
    }

    #[test]
    fn test_retry_boundary_max_retries_exhausted() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::Fixed {
            delay: Duration::from_millis(10),
            max_retries: 2,
        });
        boundary.report_error("error");
        assert!(boundary.retry());
        boundary.fail("error");
        assert!(boundary.retry());
        boundary.fail("error");
        assert!(!boundary.retry()); // exhausted
        assert!(matches!(boundary.state_untracked(), RetryState::Failed { .. }));
    }

    #[test]
    fn test_retry_boundary_render_ok() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::default());
        let html = boundary.render(&|| "<div>Content</div>".to_string(), &|| "Error".to_string());
        assert_eq!(html, "<div>Content</div>");
    }

    #[test]
    fn test_retry_boundary_render_failed_with_cache() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::FallbackCached { max_retries: 1 });
        boundary.cache_content("<div>Cached</div>");
        boundary.report_error("error");
        boundary.retry();
        boundary.fail("error"); // exhaust retries
        let html = boundary.render(&|| "Content".to_string(), &|| "Fallback".to_string());
        assert_eq!(html, "<div>Cached</div>");
    }

    #[test]
    fn test_retry_boundary_render_failed_with_static() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::FallbackStatic {
            static_content: "<div>Static</div>".to_string(),
            max_retries: 1,
        });
        boundary.report_error("error");
        boundary.retry();
        boundary.fail("error"); // exhaust retries
        let html = boundary.render(&|| "Content".to_string(), &|| "Fallback".to_string());
        assert_eq!(html, "<div>Static</div>");
    }

    #[test]
    fn test_retry_boundary_reset() {
        let boundary = RetryErrorBoundary::new(RetryStrategy::default());
        boundary.report_error("error");
        boundary.retry();
        boundary.reset();
        assert_eq!(boundary.state_untracked(), RetryState::Ok);
        assert_eq!(boundary.retry_count(), 0);
    }

    #[test]
    fn test_retry_boundary_on_retry_callback() {
        let called = Rc::new(RefCell::new(false));
        let called_clone = Rc::clone(&called);
        let boundary = RetryErrorBoundary::new(RetryStrategy::Fixed {
            delay: Duration::from_millis(10),
            max_retries: 3,
        })
        .on_retry(move || {
            *called_clone.borrow_mut() = true;
        });
        boundary.report_error("error");
        boundary.retry();
        assert!(*called.borrow());
    }

    #[test]
    fn test_retry_strategy_default() {
        let strategy = RetryStrategy::default();
        assert_eq!(strategy.max_retries(), 3);
        assert!(strategy.delay_for(0).is_some());
    }
}
