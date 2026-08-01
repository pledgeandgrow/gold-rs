//! Goal 137: Crash reporting.
//!
//! This module is re-exported from `telemetry` since crash reporting and
//! telemetry are closely related. See `telemetry::CrashReport`,
//! `telemetry::CrashReporter`, and `telemetry::crash_handler_script`.

pub use super::telemetry::{CrashReport, CrashReporter, crash_handler_script};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_exports() {
        let crash = CrashReport::new("Test error");
        assert!(!crash.message.is_empty());

        let reporter = CrashReporter::new();
        assert_eq!(reporter.len(), 0);

        assert!(crash_handler_script().contains("addEventListener"));
    }
}
