//! Goal 140: Web Vitals tracking.
//!
//! This module is re-exported from `analytics` since Web Vitals and analytics
//! are closely related. See `analytics::WebVitals` and `analytics::web_vitals_script`.

pub use super::analytics::{web_vitals_script, VitalRating, WebVitals};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_exports() {
        let vitals = WebVitals::default();
        assert_eq!(vitals.lcp, None);
        assert_eq!(VitalRating::Unknown.as_str(), "unknown");
        assert!(web_vitals_script().contains("PerformanceObserver"));
    }
}
