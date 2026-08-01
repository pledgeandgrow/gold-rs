//! `rpg explain` CLI command — Goals 151, 152.
//!
//! Explains rye error codes with human-readable and JSON output.
//! Designed for both humans and AI agents.

use rye_core::error_codes;

/// Run the `rpg explain` command.
///
/// Usage:
///   rpg explain R001           — Show explanation for error code R001
///   rpg explain R001 --json    — Output as JSON (for AI agents)
///   rpg explain --search signal — Search error codes by keyword
///   rpg explain --list          — List all error codes
///   rpg explain --list --json   — List all error codes as JSON
///   rpg explain --category ai   — List all AI error codes
pub fn run(args: &[String]) {
    let json_mode = args.iter().any(|a| a == "--json" || a == "-j");

    // Check for flags
    if args.iter().any(|a| a == "--list" || a == "-l") {
        cmd_list(json_mode, filter_category(args));
        return;
    }

    if let Some(search_idx) = args.iter().position(|a| a == "--search" || a == "-s") {
        if let Some(query) = args.get(search_idx + 1) {
            cmd_search(query, json_mode);
            return;
        }
        eprintln!("Usage: rpg explain --search <keyword>");
        return;
    }

    // Otherwise, look up a specific error code
    let code_arg = args.iter().find(|a| !a.starts_with('-'));
    if let Some(code) = code_arg {
        cmd_explain(code, json_mode);
    } else {
        print_explain_help();
    }
}

/// Filter category from args (e.g. --category ai).
fn filter_category(args: &[String]) -> Option<error_codes::ErrorCategory> {
    let cat_idx = args.iter().position(|a| a == "--category" || a == "-c")?;
    let cat = args.get(cat_idx + 1)?;
    match cat.to_lowercase().as_str() {
        "parse" | "p" => Some(error_codes::ErrorCategory::Parse),
        "validation" | "v" => Some(error_codes::ErrorCategory::Validation),
        "type" | "t" => Some(error_codes::ErrorCategory::Type),
        "reactivity" | "r" => Some(error_codes::ErrorCategory::Reactivity),
        "renderer" => Some(error_codes::ErrorCategory::Renderer),
        "router" => Some(error_codes::ErrorCategory::Router),
        "ssr" => Some(error_codes::ErrorCategory::Ssr),
        "cli" => Some(error_codes::ErrorCategory::Cli),
        "ai" | "a" => Some(error_codes::ErrorCategory::Ai),
        _ => None,
    }
}

/// Explain a specific error code.
fn cmd_explain(code: &str, json: bool) {
    match error_codes::lookup(code) {
        Some(entry) => {
            if json {
                println!("{}", entry.format_json());
            } else {
                print!("{}", entry.format_text());
            }
        }
        None => {
            if json {
                println!(r#"{{"error":"unknown_code","code":"{}","message":"No error code found for '{}'. Use 'rpg explain --list' to see all codes."}}"#, code, code);
            } else {
                eprintln!("Unknown error code: {}", code);
                eprintln!("Use 'rpg explain --list' to see all error codes.");
                eprintln!("Use 'rpg explain --search <keyword>' to search by keyword.");
            }
        }
    }
}

/// List all error codes.
fn cmd_list(json: bool, category: Option<error_codes::ErrorCategory>) {
    let codes = match category {
        Some(cat) => error_codes::list_category(cat),
        None => error_codes::all_codes().iter().collect(),
    };

    if json {
        let entries: Vec<String> = codes.iter().map(|c| c.format_json()).collect();
        println!("[{}]", entries.join(","));
    } else {
        if let Some(cat) = category {
            println!("Error codes — {} category ({}):\n", cat.name(), codes.len());
        } else {
            println!("All rye error codes ({}):\n", codes.len());
        }
        for code in &codes {
            println!("  {} — {}", code.code, code.message);
        }
        println!("\nUse 'rpg explain <CODE>' for detailed explanation.");
    }
}

/// Search error codes by keyword.
fn cmd_search(query: &str, json: bool) {
    let results = error_codes::search(query);

    if results.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("No error codes found matching '{}'.", query);
        }
        return;
    }

    if json {
        let entries: Vec<String> = results.iter().map(|c| c.format_json()).collect();
        println!("[{}]", entries.join(","));
    } else {
        println!("Error codes matching '{}' ({}):\n", query, results.len());
        for code in &results {
            println!("  {} — {}", code.code, code.message);
        }
        println!("\nUse 'rpg explain <CODE>' for detailed explanation.");
    }
}

/// Print help for the explain command.
fn print_explain_help() {
    println!("rpg explain — Explain rye error codes");
    println!();
    println!("USAGE:");
    println!("  rpg explain <CODE>           Explain a specific error code");
    println!("  rpg explain <CODE> --json    Output as JSON (for AI agents)");
    println!("  rpg explain --search <word>  Search error codes by keyword");
    println!("  rpg explain --list           List all error codes");
    println!("  rpg explain --list -c <cat>  List codes in a category");
    println!();
    println!("CATEGORIES:");
    println!("  parse, validation, type, reactivity, renderer, router, ssr, cli, ai");
    println!();
    println!("EXAMPLES:");
    println!("  rpg explain R001");
    println!("  rpg explain R800 --json");
    println!("  rpg explain --search signal");
    println!("  rpg explain --list --category ai");
}
