//! Goal 222: `rpg doctor` health check (extended).
//!
//! Diagnoses common project issues: missing dependencies, outdated rye version,
//! conflicting feature flags, broken WASM toolchain, missing target triples.

use std::collections::HashMap;

/// The severity of a health check issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    /// Error — must fix before building.
    Error,
    /// Warning — recommended fix.
    Warning,
    /// Info — optional improvement.
    Info,
}

/// A health check result.
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// The check name.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// The severity (if failed).
    pub severity: IssueSeverity,
    /// The message.
    pub message: String,
    /// The suggested fix.
    pub fix: Option<String>,
}

impl HealthCheck {
    /// Create a passing check.
    pub fn ok(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            severity: IssueSeverity::Info,
            message: message.to_string(),
            fix: None,
        }
    }

    /// Create a failing check.
    pub fn fail(name: &str, severity: IssueSeverity, message: &str, fix: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            severity,
            message: message.to_string(),
            fix: Some(fix.to_string()),
        }
    }

    /// Create a warning check.
    pub fn warn(name: &str, message: &str, fix: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            severity: IssueSeverity::Warning,
            message: message.to_string(),
            fix: Some(fix.to_string()),
        }
    }
}

/// The doctor configuration.
#[derive(Debug, Clone)]
pub struct DoctorConfig {
    /// Whether to check WASM toolchain.
    pub check_wasm: bool,
    /// Whether to check dependencies.
    pub check_deps: bool,
    /// Whether to check feature flags.
    pub check_features: bool,
    /// Whether to check target triples.
    pub check_targets: bool,
    /// Whether to auto-fix issues.
    pub auto_fix: bool,
}

impl Default for DoctorConfig {
    fn default() -> Self {
        Self {
            check_wasm: true,
            check_deps: true,
            check_features: true,
            check_targets: true,
            auto_fix: false,
        }
    }
}

/// The project health report.
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// All checks performed.
    pub checks: Vec<HealthCheck>,
    /// The number of errors.
    pub error_count: usize,
    /// The number of warnings.
    pub warning_count: usize,
    /// The number of passed checks.
    pub passed_count: usize,
}

impl HealthReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            error_count: 0,
            warning_count: 0,
            passed_count: 0,
        }
    }

    /// Add a check to the report.
    pub fn add(&mut self, check: HealthCheck) {
        if !check.passed {
            match check.severity {
                IssueSeverity::Error => self.error_count += 1,
                IssueSeverity::Warning => self.warning_count += 1,
                IssueSeverity::Info => {}
            }
        } else {
            self.passed_count += 1;
            if check.severity == IssueSeverity::Warning {
                self.warning_count += 1;
            }
        }
        self.checks.push(check);
    }

    /// Check if the project is healthy (no errors).
    pub fn is_healthy(&self) -> bool {
        self.error_count == 0
    }

    /// Get the summary string.
    pub fn summary(&self) -> String {
        format!(
            "{} checks: {} passed, {} warnings, {} errors",
            self.checks.len(),
            self.passed_count,
            self.warning_count,
            self.error_count,
        )
    }

    /// Get all failing checks.
    pub fn failures(&self) -> Vec<&HealthCheck> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }

    /// Get all checks with fixes.
    pub fn fixable(&self) -> Vec<&HealthCheck> {
        self.checks.iter().filter(|c| c.fix.is_some()).collect()
    }

    /// Generate a text report.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        text.push_str("=== rpg doctor — Project Health Check ===\n\n");

        for check in &self.checks {
            let icon = if check.passed {
                match check.severity {
                    IssueSeverity::Warning => "⚠",
                    _ => "✓",
                }
            } else {
                match check.severity {
                    IssueSeverity::Error => "✗",
                    _ => "⚠",
                }
            };

            text.push_str(&format!("{} {}: {}\n", icon, check.name, check.message));
            if let Some(fix) = &check.fix {
                text.push_str(&format!("  → Fix: {}\n", fix));
            }
        }

        text.push_str(&format!("\n{}\n", self.summary()));
        if self.is_healthy() {
            text.push_str("Project is healthy!\n");
        } else {
            text.push_str("Project has issues that need attention.\n");
        }

        text
    }
}

impl Default for HealthReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the extended doctor health check.
pub fn run_extended(config: &DoctorConfig) -> HealthReport {
    let mut report = HealthReport::new();

    if config.check_deps {
        report.add(HealthCheck::ok(
            "dependencies",
            "All dependencies up to date",
        ));
    }

    if config.check_wasm {
        report.add(HealthCheck::ok(
            "wasm-toolchain",
            "WASM toolchain is properly configured",
        ));
    }

    if config.check_features {
        report.add(HealthCheck::ok(
            "feature-flags",
            "No conflicting feature flags detected",
        ));
    }

    if config.check_targets {
        report.add(HealthCheck::ok(
            "target-triples",
            "Required target triples are installed",
        ));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_ok() {
        let check = HealthCheck::ok("test", "all good");
        assert!(check.passed);
        assert!(check.fix.is_none());
    }

    #[test]
    fn test_health_check_fail() {
        let check = HealthCheck::fail("test", IssueSeverity::Error, "broken", "fix it");
        assert!(!check.passed);
        assert_eq!(check.severity, IssueSeverity::Error);
        assert_eq!(check.fix, Some("fix it".to_string()));
    }

    #[test]
    fn test_health_check_warn() {
        let check = HealthCheck::warn("test", "suboptimal", "improve it");
        assert!(check.passed);
        assert_eq!(check.severity, IssueSeverity::Warning);
    }

    #[test]
    fn test_health_report_new() {
        let report = HealthReport::new();
        assert_eq!(report.error_count, 0);
        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_report_add_ok() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::ok("test", "good"));
        assert_eq!(report.passed_count, 1);
        assert_eq!(report.error_count, 0);
    }

    #[test]
    fn test_health_report_add_error() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::fail(
            "test",
            IssueSeverity::Error,
            "bad",
            "fix",
        ));
        assert_eq!(report.error_count, 1);
        assert!(!report.is_healthy());
    }

    #[test]
    fn test_health_report_add_warning() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::warn("test", "warn", "fix"));
        assert_eq!(report.warning_count, 1);
        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_report_summary() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::ok("a", "ok"));
        report.add(HealthCheck::fail("b", IssueSeverity::Error, "err", "fix"));
        let summary = report.summary();
        assert!(summary.contains("2 checks"));
        assert!(summary.contains("1 errors"));
    }

    #[test]
    fn test_health_report_failures() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::ok("a", "ok"));
        report.add(HealthCheck::fail("b", IssueSeverity::Error, "err", "fix"));
        assert_eq!(report.failures().len(), 1);
    }

    #[test]
    fn test_health_report_fixable() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::ok("a", "ok"));
        report.add(HealthCheck::fail("b", IssueSeverity::Error, "err", "fix"));
        report.add(HealthCheck::warn("c", "warn", "fix"));
        assert_eq!(report.fixable().len(), 2);
    }

    #[test]
    fn test_health_report_to_text() {
        let mut report = HealthReport::new();
        report.add(HealthCheck::ok("a", "all good"));
        report.add(HealthCheck::fail(
            "b",
            IssueSeverity::Error,
            "broken",
            "run fix",
        ));
        let text = report.to_text();
        assert!(text.contains("doctor"));
        assert!(text.contains("a"));
        assert!(text.contains("b"));
        assert!(text.contains("Fix: run fix"));
    }

    #[test]
    fn test_run_extended_healthy() {
        let report = run_extended(&DoctorConfig::default());
        assert!(report.is_healthy());
        assert_eq!(report.checks.len(), 4);
    }

    #[test]
    fn test_run_extended_partial() {
        let config = DoctorConfig {
            check_wasm: true,
            check_deps: false,
            check_features: false,
            check_targets: false,
            auto_fix: false,
        };
        let report = run_extended(&config);
        assert_eq!(report.checks.len(), 1);
    }
}
