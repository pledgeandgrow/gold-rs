//! `rpg test` — run tests via `cargo test` with rye-specific conveniences.
//!
//! ## Flags
//!
//! - `--unit` — Run only unit tests (lib + bins)
//! - `--e2e` — Run end-to-end tests (integration tests + doc tests)
//! - `--watch` — Watch for changes and re-run tests
//! - `--release` — Run tests in release mode
//! - `--features <list>` — Comma-separated Cargo features to enable
//! - `--package <name>` — Test a specific package
//! - `--coverage` — Run tests with coverage (uses `cargo llvm-cov` or `cargo tarpaulin`)
//! - `--generate` / `-g` — Generate test scaffolding (delegates to `test_gen`)
//! - `-- <args>` — Pass remaining args directly to `cargo test`
//! - `<pattern>` — Filter tests by name pattern

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Test configuration — parsed from CLI args.
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Whether to run only unit tests.
    pub unit: bool,
    /// Whether to run only e2e tests.
    pub e2e: bool,
    /// Whether to watch for changes.
    pub watch: bool,
    /// Whether to run in release mode.
    pub release: bool,
    /// Whether to collect coverage.
    pub coverage: bool,
    /// Additional Cargo features to enable.
    pub features: Vec<String>,
    /// Specific package to test.
    pub package: Option<String>,
    /// Test name filter pattern.
    pub pattern: Option<String>,
    /// Extra args to pass directly to `cargo test` (after `--`).
    pub extra_args: Vec<String>,
    /// The project root directory.
    pub project_root: PathBuf,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            unit: false,
            e2e: false,
            watch: false,
            release: false,
            coverage: false,
            features: Vec::new(),
            package: None,
            pattern: None,
            extra_args: Vec::new(),
            project_root: PathBuf::from("."),
        }
    }
}

impl TestConfig {
    /// Parse test config from CLI args.
    ///
    /// Supported flags:
    /// - `--unit` — Unit tests only
    /// - `--e2e` — E2E tests only
    /// - `--watch` / `-w` — Watch mode
    /// - `--release` / `-r` — Release mode
    /// - `--coverage` — Coverage mode
    /// - `--features <list>` — Cargo features
    /// - `--package <name>` / `-p <name>` — Specific package
    /// - `-- <args>` — Pass-through args
    /// - `<pattern>` — Test name filter (first non-flag arg)
    pub fn from_args(args: &[String]) -> Self {
        let mut config = Self::default();

        let mut i = 0;
        let mut found_separator = false;
        while i < args.len() {
            let arg = &args[i];

            if found_separator {
                config.extra_args.push(arg.clone());
                i += 1;
                continue;
            }

            match arg.as_str() {
                "--unit" => config.unit = true,
                "--e2e" => config.e2e = true,
                "--watch" | "-w" => config.watch = true,
                "--release" | "-r" => config.release = true,
                "--coverage" => config.coverage = true,
                "--" => found_separator = true,
                "--features" if i + 1 < args.len() => {
                    config.features = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    i += 1;
                }
                s if s.starts_with("--features=") => {
                    config.features = s[11..].split(',').map(|s| s.trim().to_string()).collect();
                }
                "--package" if i + 1 < args.len() => {
                    config.package = Some(args[i + 1].clone());
                    i += 1;
                }
                "-p" if i + 1 < args.len() => {
                    config.package = Some(args[i + 1].clone());
                    i += 1;
                }
                s if s.starts_with("--package=") => {
                    config.package = Some(s[10..].to_string());
                }
                s if s.starts_with('-') => {
                    // Unknown flag — ignore.
                }
                s => {
                    // First non-flag arg is the test filter pattern.
                    if config.pattern.is_none() {
                        config.pattern = Some(s.to_string());
                    }
                }
            }
            i += 1;
        }

        config
    }
}

/// Result of a test run.
#[derive(Debug)]
pub struct TestResult {
    /// Whether the tests passed.
    pub success: bool,
    /// Number of tests passed.
    pub passed: usize,
    /// Number of tests failed.
    pub failed: usize,
    /// Number of tests ignored.
    pub ignored: usize,
    /// Raw stderr output.
    pub output: String,
}

/// Run the `rpg test` command.
///
/// This is the main entry point called from `cmd_test` in main.rs.
pub fn run(args: &[String]) {
    // Check for --generate flag — delegates to test_gen module.
    if args.iter().any(|a| a == "--generate" || a == "-g") {
        let gen_args: Vec<String> = args
            .iter()
            .filter(|a| *a != "--generate" && *a != "-g")
            .cloned()
            .collect();
        crate::test_gen::run(&gen_args);
        return;
    }

    let config = TestConfig::from_args(args);

    println!("  rye test");
    if config.unit {
        println!("  scope: unit tests");
    } else if config.e2e {
        println!("  scope: e2e tests");
    } else {
        println!("  scope: all tests");
    }
    if config.release {
        println!("  mode:  release");
    }
    if config.coverage {
        println!("  coverage: enabled");
    }
    if let Some(pkg) = &config.package {
        println!("  package: {}", pkg);
    }
    if let Some(pattern) = &config.pattern {
        println!("  filter:  {}", pattern);
    }
    if !config.features.is_empty() {
        println!("  features: {}", config.features.join(", "));
    }
    println!();

    if config.watch {
        run_watch(&config);
        return;
    }

    let result = execute(&config);
    if result.success {
        println!();
        println!(
            "  Test result: ok. {} passed; {} failed; {} ignored",
            result.passed, result.failed, result.ignored
        );
    } else {
        eprintln!();
        eprintln!(
            "  Test result: FAILED. {} passed; {} failed; {} ignored",
            result.passed, result.failed, result.ignored
        );
        std::process::exit(1);
    }
}

/// Execute the test run based on the config.
fn execute(config: &TestConfig) -> TestResult {
    if config.coverage {
        return run_with_coverage(config);
    }
    run_cargo_test(config)
}

/// Run `cargo test` with the configured options.
fn run_cargo_test(config: &TestConfig) -> TestResult {
    let project_root = find_project_root(&config.project_root);

    let mut cmd = Command::new("cargo");
    cmd.arg("test").current_dir(&project_root);

    if config.release {
        cmd.arg("--release");
    }

    if let Some(pkg) = &config.package {
        cmd.arg("--package").arg(pkg);
    }

    if !config.features.is_empty() {
        cmd.arg("--features").arg(config.features.join(","));
    }

    // Unit vs e2e scope.
    if config.unit {
        // Unit tests: lib + bins only (no integration tests, no doc tests).
        cmd.arg("--lib").arg("--bins");
    } else if config.e2e {
        // E2e tests: integration tests + doc tests.
        cmd.arg("--tests").arg("--doc");
    }

    // Test name filter pattern.
    if let Some(pattern) = &config.pattern {
        cmd.arg(pattern);
    }

    // Pass-through extra args after `--`.
    if !config.extra_args.is_empty() {
        cmd.arg("--").args(&config.extra_args);
    }

    // Build display string.
    let display = format_command(&cmd);
    println!("  Running: {}", display);

    let output = cmd.stdout(Stdio::inherit()).stderr(Stdio::piped()).output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let (passed, failed, ignored) = parse_test_summary(&stderr);
            TestResult {
                success: result.status.success(),
                passed,
                failed,
                ignored,
                output: stderr,
            }
        }
        Err(e) => TestResult {
            success: false,
            passed: 0,
            failed: 0,
            ignored: 0,
            output: format!("Failed to run cargo test: {}", e),
        },
    }
}

/// Run tests with coverage using `cargo llvm-cov` or `cargo tarpaulin`.
fn run_with_coverage(config: &TestConfig) -> TestResult {
    let project_root = find_project_root(&config.project_root);

    // Try cargo llvm-cov first (more common on stable).
    let tool = if is_tool_installed("cargo-llvm-cov") {
        "cargo-llvm-cov"
    } else if is_tool_installed("cargo-tarpaulin") {
        "cargo-tarpaulin"
    } else {
        return TestResult {
            success: false,
            passed: 0,
            failed: 0,
            ignored: 0,
            output: "No coverage tool installed. Install one of:\n  cargo install cargo-llvm-cov\n  cargo install cargo-tarpaulin".to_string(),
        };
    };

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&project_root);

    if tool == "cargo-llvm-cov" {
        cmd.arg("llvm-cov").arg("--text");
    } else {
        cmd.arg("tarpaulin");
    }

    // Common args.
    if let Some(pkg) = &config.package {
        cmd.arg("--package").arg(pkg);
    }

    if !config.features.is_empty() {
        cmd.arg("--features").arg(config.features.join(","));
    }

    if let Some(pattern) = &config.pattern {
        cmd.arg(pattern);
    }

    let display = format_command(&cmd);
    println!("  Running: {}", display);
    println!("  Coverage tool: {}", tool);

    let output = cmd.stdout(Stdio::inherit()).stderr(Stdio::piped()).output();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let (passed, failed, ignored) = parse_test_summary(&stderr);
            TestResult {
                success: result.status.success(),
                passed,
                failed,
                ignored,
                output: stderr,
            }
        }
        Err(e) => TestResult {
            success: false,
            passed: 0,
            failed: 0,
            ignored: 0,
            output: format!("Failed to run coverage tool: {}", e),
        },
    }
}

/// Watch mode — re-run tests on file change.
fn run_watch(config: &TestConfig) {
    let project_root = find_project_root(&config.project_root);

    // Check if cargo-watch is installed.
    if !is_tool_installed("cargo-watch") {
        eprintln!("  cargo-watch is not installed. Install it with: cargo install cargo-watch");
        eprintln!("  Alternatively, run `rpg test` without --watch.");
        std::process::exit(1);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("watch").current_dir(&project_root);

    // Configure watch args.
    let mut watch_args = vec!["-x".to_string(), "test".to_string()];

    if config.release {
        watch_args.extend(["--release".to_string()]);
    }
    if let Some(pkg) = &config.package {
        watch_args.extend(["--package".to_string(), pkg.clone()]);
    }
    if !config.features.is_empty() {
        watch_args.extend(["--features".to_string(), config.features.join(",")]);
    }
    if config.unit {
        watch_args.extend(["--lib".to_string(), "--bins".to_string()]);
    } else if config.e2e {
        watch_args.extend(["--tests".to_string(), "--doc".to_string()]);
    }
    if let Some(pattern) = &config.pattern {
        watch_args.push(pattern.clone());
    }

    cmd.args(&watch_args);

    println!("  Running: cargo watch -x test ...");
    println!("  Press Ctrl+C to stop.");
    println!();

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) => {
            if !s.success() {
                std::process::exit(s.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("  Failed to run cargo watch: {}", e);
            std::process::exit(1);
        }
    }
}

/// Parse test summary from cargo test stderr output.
///
/// Looks for lines like:
///   `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
fn parse_test_summary(output: &str) -> (usize, usize, usize) {
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_ignored = 0;

    for line in output.lines() {
        if line.starts_with("test result:") {
            // Parse: "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
            let passed = extract_number(line, "passed");
            let failed = extract_number(line, "failed");
            let ignored = extract_number(line, "ignored");
            total_passed += passed;
            total_failed += failed;
            total_ignored += ignored;
        }
    }

    (total_passed, total_failed, total_ignored)
}

/// Extract a number following a keyword in a string.
fn extract_number(line: &str, keyword: &str) -> usize {
    let pattern = format!("{};", keyword);
    if let Some(idx) = line.find(&pattern) {
        let before = &line[..idx];
        // Walk backwards to find the number.
        let num_str: String = before
            .trim()
            .rsplit(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap_or("0")
            .to_string();
        num_str.parse().unwrap_or(0)
    } else {
        0
    }
}

/// Find the project root by searching for Cargo.toml.
fn find_project_root(start: &Path) -> PathBuf {
    let mut current = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    loop {
        if current.join("Cargo.toml").exists() {
            return current;
        }
        if !current.pop() {
            return std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}

/// Check if a command-line tool is installed.
fn is_tool_installed(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Format a Command into a display string for logging.
fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy().to_string();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().to_string())
        .collect();
    if args.is_empty() {
        program
    } else {
        format!("{} {}", program, args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(!config.unit);
        assert!(!config.e2e);
        assert!(!config.watch);
        assert!(!config.release);
        assert!(!config.coverage);
        assert!(config.package.is_none());
        assert!(config.pattern.is_none());
    }

    #[test]
    fn test_config_from_args_unit() {
        let args = vec!["--unit".to_string()];
        let config = TestConfig::from_args(&args);
        assert!(config.unit);
        assert!(!config.e2e);
    }

    #[test]
    fn test_config_from_args_e2e() {
        let args = vec!["--e2e".to_string()];
        let config = TestConfig::from_args(&args);
        assert!(config.e2e);
        assert!(!config.unit);
    }

    #[test]
    fn test_config_from_args_watch() {
        let args = vec!["--watch".to_string()];
        let config = TestConfig::from_args(&args);
        assert!(config.watch);
    }

    #[test]
    fn test_config_from_args_release() {
        let args = vec!["--release".to_string()];
        let config = TestConfig::from_args(&args);
        assert!(config.release);
    }

    #[test]
    fn test_config_from_args_coverage() {
        let args = vec!["--coverage".to_string()];
        let config = TestConfig::from_args(&args);
        assert!(config.coverage);
    }

    #[test]
    fn test_config_from_args_features() {
        let args = vec!["--features".to_string(), "foo,bar".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.features, vec!["foo", "bar"]);
    }

    #[test]
    fn test_config_from_args_features_equals() {
        let args = vec!["--features=foo,bar,baz".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.features, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_config_from_args_package() {
        let args = vec!["--package".to_string(), "rye-core".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.package.as_deref(), Some("rye-core"));
    }

    #[test]
    fn test_config_from_args_package_short() {
        let args = vec!["-p".to_string(), "rye-cli".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.package.as_deref(), Some("rye-cli"));
    }

    #[test]
    fn test_config_from_args_package_equals() {
        let args = vec!["--package=rye-mobile".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.package.as_deref(), Some("rye-mobile"));
    }

    #[test]
    fn test_config_from_args_pattern() {
        let args = vec!["test_foo".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.pattern.as_deref(), Some("test_foo"));
    }

    #[test]
    fn test_config_from_args_extra_args() {
        let args = vec![
            "--".to_string(),
            "--nocapture".to_string(),
            "--test-threads=4".to_string(),
        ];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.extra_args, vec!["--nocapture", "--test-threads=4"]);
    }

    #[test]
    fn test_config_from_args_combined() {
        let args = vec![
            "--unit".to_string(),
            "--release".to_string(),
            "-p".to_string(),
            "rye-core".to_string(),
            "test_render".to_string(),
            "--".to_string(),
            "--nocapture".to_string(),
        ];
        let config = TestConfig::from_args(&args);
        assert!(config.unit);
        assert!(config.release);
        assert_eq!(config.package.as_deref(), Some("rye-core"));
        assert_eq!(config.pattern.as_deref(), Some("test_render"));
        assert_eq!(config.extra_args, vec!["--nocapture"]);
    }

    #[test]
    fn test_config_from_args_unknown_flag_ignored() {
        let args = vec!["--unknown-flag".to_string(), "test_foo".to_string()];
        let config = TestConfig::from_args(&args);
        assert_eq!(config.pattern.as_deref(), Some("test_foo"));
    }

    #[test]
    fn test_parse_test_summary_ok() {
        let output = "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
        let (passed, failed, ignored) = parse_test_summary(output);
        assert_eq!(passed, 5);
        assert_eq!(failed, 0);
        assert_eq!(ignored, 0);
    }

    #[test]
    fn test_parse_test_summary_failed() {
        let output =
            "test result: FAILED. 3 passed; 2 failed; 1 ignored; 0 measured; 0 filtered out";
        let (passed, failed, ignored) = parse_test_summary(output);
        assert_eq!(passed, 3);
        assert_eq!(failed, 2);
        assert_eq!(ignored, 1);
    }

    #[test]
    fn test_parse_test_summary_multiple_lines() {
        let output = "\
running 5 tests
test foo ... ok
test bar ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 3 tests
test baz ... ok
test qux ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out";
        let (passed, failed, ignored) = parse_test_summary(output);
        assert_eq!(passed, 3);
        assert_eq!(failed, 1);
        assert_eq!(ignored, 0);
    }

    #[test]
    fn test_parse_test_summary_no_results() {
        let output = "compiling rye-core...\nno tests to run";
        let (passed, failed, ignored) = parse_test_summary(output);
        assert_eq!(passed, 0);
        assert_eq!(failed, 0);
        assert_eq!(ignored, 0);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("5 passed;", "passed"), 5);
        assert_eq!(extract_number("0 failed;", "failed"), 0);
        assert_eq!(extract_number("12 ignored;", "ignored"), 12);
        assert_eq!(extract_number("no match here", "passed"), 0);
    }

    #[test]
    fn test_find_project_root() {
        let root = find_project_root(Path::new("."));
        assert!(root.join("Cargo.toml").exists());
    }
}
