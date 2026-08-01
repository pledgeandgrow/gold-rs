//! `rpg lint --ai` CLI command — Goal 156.
//!
//! AI-aware linter that checks rye source code for common mistakes.
//! Uses the code review engine from rye-core.

use rye_core::ai::code_review;
use std::fs;
use std::path::Path;

/// Run the `rpg lint --ai` command.
///
/// Usage:
///   rpg lint --ai <file>           Lint a specific file
///   rpg lint --ai --dir <dir>      Lint all .rs files in a directory
///   rpg lint --ai --json <file>    Output as JSON
pub fn run(args: &[String]) {
    let json_mode = args.iter().any(|a| a == "--json" || a == "-j");

    // Check for --dir flag
    if let Some(dir_idx) = args.iter().position(|a| a == "--dir" || a == "-d") {
        if let Some(dir) = args.get(dir_idx + 1) {
            lint_directory(dir, json_mode);
            return;
        }
        eprintln!("Usage: rpg lint --ai --dir <directory>");
        return;
    }

    // Otherwise, lint a specific file
    let file_arg = args.iter().find(|a| !a.starts_with('-'));
    if let Some(file_path) = file_arg {
        lint_file(file_path, json_mode);
    } else {
        print_lint_help();
    }
}

fn lint_file(file_path: &str, json: bool) {
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("File not found: {}", file_path);
        std::process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let result = code_review::review_source(file_path, &source);

    if json {
        println!("{}", result.format_json());
    } else {
        print!("{}", result.format_text());
    }

    let errors = result.findings.iter().filter(|f| f.severity == code_review::Severity::Error).count();
    if errors > 0 {
        std::process::exit(1);
    }
}

fn lint_directory(dir: &str, json: bool) {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        eprintln!("Directory not found: {}", dir);
        std::process::exit(1);
    }

    let mut total_errors = 0;
    let mut total_files = 0;
    let mut all_results = Vec::new();

    let entries = match fs::read_dir(dir_path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
            std::process::exit(1);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Some(path_str) = path.to_str() {
                let source = match fs::read_to_string(&path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let result = code_review::review_source(path_str, &source);
                let errors = result.findings.iter().filter(|f| f.severity == code_review::Severity::Error).count();
                total_errors += errors;
                total_files += 1;

                if json {
                    all_results.push(result);
                } else if errors > 0 || !result.findings.is_empty() {
                    print!("{}", result.format_text());
                    println!("---");
                }
            }
        }
    }

    if json {
        let entries: Vec<String> = all_results.iter().map(|r| r.format_json()).collect();
        println!("[{}]", entries.join(","));
    } else {
        println!("\nLinted {} file(s), found {} error(s).", total_files, total_errors);
    }

    if total_errors > 0 {
        std::process::exit(1);
    }
}

fn print_lint_help() {
    println!("rpg lint --ai — AI-aware linter for rye code");
    println!();
    println!("USAGE:");
    println!("  rpg lint --ai <file>           Lint a specific file");
    println!("  rpg lint --ai --dir <dir>       Lint all .rs files in a directory");
    println!("  rpg lint --ai --json <file>     Output as JSON");
    println!();
    println!("CHECKS:");
    println!("  - Missing #[component] attribute (R805)");
    println!("  - Signal used without .get() (R802)");
    println!("  - Closure missing 'move' keyword (R801)");
    println!("  - Direct Signal assignment instead of .set() (R803)");
    println!("  - use_effect for derived state instead of Memo (R806)");
    println!("  - Unnecessary .clone() (R807)");
    println!("  - Raw async instead of use_resource (R809)");
    println!("  - snake_case component name (R804)");
    println!();
    println!("EXAMPLES:");
    println!("  rpg lint --ai src/components/button.rs");
    println!("  rpg lint --ai --dir src/components");
    println!("  rpg lint --ai --json src/components/button.rs");
}
