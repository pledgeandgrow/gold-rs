//! `rpg doctor` CLI command — Goal 161.
//!
//! Project health check that verifies rye project setup, dependencies,
//! and configuration. Helps AI agents and users diagnose issues.

use std::fs;
use std::path::Path;

/// Health check result.
#[derive(Debug, Clone)]
struct CheckResult {
    name: String,
    passed: bool,
    message: String,
    fix: Option<String>,
}

impl CheckResult {
    fn ok(name: &str, message: &str) -> Self {
        Self { name: name.to_string(), passed: true, message: message.to_string(), fix: None }
    }

    fn fail(name: &str, message: &str, fix: &str) -> Self {
        Self { name: name.to_string(), passed: false, message: message.to_string(), fix: Some(fix.to_string()) }
    }

    fn warn(name: &str, message: &str, fix: &str) -> Self {
        Self { name: name.to_string(), passed: true, message: message.to_string(), fix: Some(fix.to_string()) }
    }
}

/// Run the `rpg doctor` command.
pub fn run(args: &[String]) {
    let json_mode = args.iter().any(|a| a == "--json" || a == "-j");
    let project_root = find_project_root();

    let mut results = Vec::new();

    // Check 1: Cargo.toml exists
    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() {
        results.push(CheckResult::ok("Cargo.toml", "Found Cargo.toml"));
    } else {
        results.push(CheckResult::fail("Cargo.toml", "No Cargo.toml found", "Run 'rpg new <name>' to create a project"));
    }

    // Check 2: rye dependency
    if let Ok(content) = fs::read_to_string(&cargo_toml) {
        if content.contains("rye-core") || content.contains("rye-macros") || content.contains("rye =") {
            results.push(CheckResult::ok("rye dependency", "rye dependency found in Cargo.toml"));
        } else {
            results.push(CheckResult::fail("rye dependency", "No rye dependency in Cargo.toml", "Add rye to Cargo.toml: rye-core = { workspace = true }"));
        }

        // Check 3: Edition
        if content.contains("edition = \"2021\"") || content.contains("edition = \"2024\"") {
            results.push(CheckResult::ok("Rust edition", "Using edition 2021 or 2024"));
        } else if content.contains("edition = \"2018\"") {
            results.push(CheckResult::warn("Rust edition", "Using edition 2018 (consider upgrading)", "Update to edition 2021 in Cargo.toml"));
        } else {
            results.push(CheckResult::warn("Rust edition", "Unknown edition", "Set edition = \"2021\" in Cargo.toml"));
        }
    }

    // Check 4: src/ directory
    let src_dir = project_root.join("src");
    if src_dir.exists() && src_dir.is_dir() {
        results.push(CheckResult::ok("src/ directory", "src/ directory exists"));
    } else {
        results.push(CheckResult::fail("src/ directory", "No src/ directory", "Create src/ directory with main.rs or lib.rs"));
    }

    // Check 5: main.rs or lib.rs
    let main_rs = src_dir.join("main.rs");
    let lib_rs = src_dir.join("lib.rs");
    if main_rs.exists() {
        results.push(CheckResult::ok("Entry point", "src/main.rs found"));
    } else if lib_rs.exists() {
        results.push(CheckResult::ok("Entry point", "src/lib.rs found"));
    } else {
        results.push(CheckResult::fail("Entry point", "No src/main.rs or src/lib.rs", "Create src/main.rs or src/lib.rs"));
    }

    // Check 6: components directory
    let components_dir = src_dir.join("components");
    if components_dir.exists() {
        let count = fs::read_dir(&components_dir)
            .map(|d| d.filter(|e| e.as_ref().ok().map(|e| e.path().extension().and_then(|e| e.to_str()) == Some("rs")).unwrap_or(false))
                .count())
            .unwrap_or(0);
        if count > 0 {
            results.push(CheckResult::ok("Components", &format!("Found {} component file(s) in src/components/", count)));
        } else {
            results.push(CheckResult::warn("Components", "src/components/ exists but is empty", "Add components with 'rpg scaffold component <Name>'"));
        }
    } else {
        results.push(CheckResult::warn("Components", "No src/components/ directory", "Create with 'rpg scaffold component <Name>'"));
    }

    // Check 7: tests directory
    let tests_dir = project_root.join("tests");
    if tests_dir.exists() {
        let count = fs::read_dir(&tests_dir)
            .map(|d| d.filter(|e| e.as_ref().ok().map(|e| e.path().extension().and_then(|e| e.to_str()) == Some("rs")).unwrap_or(false))
                .count())
            .unwrap_or(0);
        if count > 0 {
            results.push(CheckResult::ok("Tests", &format!("Found {} test file(s) in tests/", count)));
        } else {
            results.push(CheckResult::warn("Tests", "tests/ exists but is empty", "Generate tests with 'rpg test --generate'"));
        }
    } else {
        results.push(CheckResult::warn("Tests", "No tests/ directory", "Generate tests with 'rpg test --generate'"));
    }

    // Check 8: .gitignore
    let gitignore = project_root.join(".gitignore");
    if gitignore.exists() {
        results.push(CheckResult::ok(".gitignore", ".gitignore exists"));
    } else {
        results.push(CheckResult::warn(".gitignore", "No .gitignore found", "Add a .gitignore with target/ and *.rs.bk"));
    }

    // Check 9: rye config
    let rye_config = project_root.join("rye.toml");
    if rye_config.exists() {
        results.push(CheckResult::ok("rye.toml", "rye.toml config found"));
    } else {
        results.push(CheckResult::warn("rye.toml", "No rye.toml config (optional)", "Create rye.toml for project-specific settings"));
    }

    // Check 10: Cargo.lock
    let cargo_lock = project_root.join("Cargo.lock");
    if cargo_lock.exists() {
        results.push(CheckResult::ok("Cargo.lock", "Cargo.lock exists (dependencies resolved)"));
    } else {
        results.push(CheckResult::warn("Cargo.lock", "No Cargo.lock", "Run 'cargo build' to generate Cargo.lock"));
    }

    // Output results
    if json_mode {
        print_json(&results);
    } else {
        print_text(&results);
    }

    // Exit code
    let failures = results.iter().filter(|r| !r.passed).count();
    if failures > 0 {
        std::process::exit(1);
    }
}

fn find_project_root() -> std::path::PathBuf {
    let mut current = std::env::current_dir().unwrap_or_default();
    loop {
        if current.join("Cargo.toml").exists() {
            return current;
        }
        if !current.pop() {
            return std::env::current_dir().unwrap_or_default();
        }
    }
}

fn print_text(results: &[CheckResult]) {
    println!("rpg doctor — Project health check\n");

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let warnings = results.iter().filter(|r| r.passed && r.fix.is_some()).count();

    for r in results {
        let icon = if r.passed {
            if r.fix.is_some() { "[WARN]" } else { "[OK]  " }
        } else {
            "[FAIL]"
        };
        println!("  {} {} — {}", icon, r.name, r.message);
        if let Some(fix) = &r.fix {
            println!("         Fix: {}", fix);
        }
    }

    println!("\n{} passed, {} warnings, {} failed", passed, warnings, failed);

    if failed > 0 {
        println!("\nFix the issues above before proceeding.");
    } else if warnings > 0 {
        println!("\nProject is functional, but consider fixing warnings.");
    } else {
        println!("\nProject is healthy!");
    }
}

fn print_json(results: &[CheckResult]) {
    let entries: Vec<String> = results
        .iter()
        .map(|r| {
            let fix = r.fix.as_ref().map(|f| format!(",\"fix\":\"{}\"", f.replace('"', "\\\""))).unwrap_or_default();
            format!(
                r#"{{"name":"{}","passed":{},"message":"{}"{} }}"#,
                r.name, r.passed, r.message.replace('"', "\\\""), fix
            )
        })
        .collect();

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    println!(
        r#"{{"checks":[{}],"passed":{},"failed":{}}}"#,
        entries.join(","),
        passed,
        failed
    );
}
