//! `rpg test --generate` CLI command — Goal 155.
//!
//! Generates test scaffolding for rye components. AI agents can use this
//! to automatically create test files that cover rendering, events,
//! and prop validation without writing tests from scratch.

use std::fs;
use std::path::Path;

/// Run the `rpg test --generate` command.
///
/// Usage:
///   rpg test --generate <path>           Generate tests for a component file
///   rpg test --generate --all            Generate tests for all components
///   rpg test --generate --dir <dir>      Generate tests for all components in a directory
pub fn run(args: &[String]) {
    // Check for --all flag
    if args.iter().any(|a| a == "--all") {
        generate_for_directory("src/components", "tests");
        return;
    }

    // Check for --dir flag
    if let Some(dir_idx) = args.iter().position(|a| a == "--dir") {
        if let Some(dir) = args.get(dir_idx + 1) {
            generate_for_directory(dir, "tests");
            return;
        }
        eprintln!("Usage: rpg test --generate --dir <directory>");
        return;
    }

    // Otherwise, generate for a specific file
    let file_arg = args.iter().find(|a| !a.starts_with('-'));
    if let Some(file_path) = file_arg {
        generate_for_file(file_path);
    } else {
        print_test_generate_help();
    }
}

/// Generate a test file for a single component file.
fn generate_for_file(component_path: &str) {
    let path = Path::new(component_path);
    if !path.exists() {
        eprintln!("File not found: {}", component_path);
        std::process::exit(1);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let components = parse_components(&content);

    if components.is_empty() {
        eprintln!("No components found in {}", component_path);
        return;
    }

    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("component");

    let test_path = format!("tests/{}_test.rs", file_stem);
    let test_code = generate_test_file(&components, file_stem);

    if let Some(parent) = Path::new(&test_path).parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&test_path, &test_code) {
        Ok(_) => {
            println!(
                "Generated test: {} -> {} ({} components)",
                component_path,
                test_path,
                components.len()
            );
            for comp in &components {
                println!("  - {} ({} test functions)", comp.name, comp.test_count());
            }
        }
        Err(e) => {
            eprintln!("Failed to write test file: {}", e);
            std::process::exit(1);
        }
    }
}

/// Generate test files for all components in a directory.
fn generate_for_directory(src_dir: &str, test_dir: &str) {
    let src_path = Path::new(src_dir);
    if !src_path.exists() {
        eprintln!("Directory not found: {}", src_dir);
        std::process::exit(1);
    }

    let mut count = 0;
    let entries = match fs::read_dir(src_path) {
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
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let components = parse_components(&content);
                if components.is_empty() {
                    continue;
                }

                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("component");

                let test_path = format!("{}/{}_test.rs", test_dir, file_stem);
                let test_code = generate_test_file(&components, file_stem);

                if let Some(parent) = Path::new(&test_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }

                if fs::write(&test_path, &test_code).is_ok() {
                    println!("Generated: {} ({} components)", test_path, components.len());
                    count += 1;
                }
            }
        }
    }

    if count == 0 {
        println!("No components found in {}", src_dir);
    } else {
        println!("\nGenerated {} test file(s).", count);
    }
}

/// Parsed component info from source code.
#[derive(Debug, Clone)]
struct ParsedComponent {
    /// Component function name (PascalCase).
    name: String,
    /// Props struct name if found.
    props_type: Option<String>,
    /// Prop fields parsed from the Props struct.
    props: Vec<(String, String)>,
    /// Whether the component has an island attribute.
    is_island: bool,
}

impl ParsedComponent {
    fn test_count(&self) -> usize {
        // renders + tag + (props validation if props) + event handling
        2 + if self.props.is_empty() { 0 } else { 1 } + 1
    }
}

/// Parse component definitions from Rust source code.
fn parse_components(source: &str) -> Vec<ParsedComponent> {
    let mut components = Vec::new();
    let mut props_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    // First pass: find Props structs
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("struct ") && trimmed.contains("Props") {
            if let Some(name) = extract_struct_name(trimmed) {
                // Look ahead for fields
                let props = extract_props_fields(source, &name);
                props_map.insert(name, props);
            }
        }
    }

    // Second pass: find #[component] functions
    let lines: Vec<&str> = source.lines().collect();
    let mut consumed = vec![false; lines.len()];
    for i in 0..lines.len() {
        if consumed[i] {
            continue;
        }
        let line = lines[i].trim();

        // Check for #[island] or #[component] attribute
        let is_island_attr = line.starts_with("#[rye::island]") || line.starts_with("#[island]");
        let is_component_attr = line.starts_with("#[component]");

        if is_island_attr || is_component_attr {
            consumed[i] = true;
            // Track if we saw an island attribute in this attribute group
            let mut found_island = is_island_attr;

            // Look ahead for the fn line, skipping other attribute lines
            let mut j = i + 1;
            while j < lines.len() {
                consumed[j] = true;
                let next_line = lines[j].trim();
                if next_line.starts_with("#[") {
                    if next_line.starts_with("#[rye::island]") || next_line.starts_with("#[island]")
                    {
                        found_island = true;
                    }
                    j += 1;
                    continue;
                }
                // Found the fn line
                if let Some(name) = extract_fn_name(next_line) {
                    let props_type = extract_props_type_from_fn(next_line);
                    let props = props_type
                        .as_ref()
                        .and_then(|pt| props_map.get(pt))
                        .cloned()
                        .unwrap_or_default();

                    components.push(ParsedComponent {
                        name,
                        props_type,
                        props,
                        is_island: found_island,
                    });
                }
                break;
            }
        }
    }

    components
}

fn extract_struct_name(line: &str) -> Option<String> {
    let after_struct = line.strip_prefix("struct ")?;
    let name = after_struct
        .split(|c: char| c.is_whitespace() || c == '{')
        .next()?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn extract_fn_name(line: &str) -> Option<String> {
    let after_fn = line.find("fn ")?;
    let rest = &line[after_fn + 3..];
    let name = rest
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()?;
    let name = name.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn extract_props_type_from_fn(line: &str) -> Option<String> {
    // Look for pattern: props: SomeType
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() >= 2 {
        let type_part = parts[1].trim();
        let type_name = type_part
            .split(|c: char| c.is_whitespace() || c == ',' || c == ')')
            .next()?
            .trim();
        if !type_name.is_empty() && type_name.ends_with("Props") {
            return Some(type_name.to_string());
        }
    }
    None
}

fn extract_props_fields(source: &str, struct_name: &str) -> Vec<(String, String)> {
    let mut props = Vec::new();

    // Find the struct definition and extract fields from its body
    let struct_pattern = format!("struct {} {{", struct_name);
    let alt_pattern = format!("struct {}{{", struct_name);

    let start = source
        .find(&struct_pattern)
        .or_else(|| source.find(&alt_pattern));

    if let Some(start_idx) = start {
        if let Some(open_brace) = source[start_idx..].find('{') {
            let abs_open = start_idx + open_brace;
            if let Some(close_brace) = source[abs_open..].find('}') {
                let body = &source[abs_open + 1..abs_open + close_brace];
                for line in body.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*")
                    {
                        continue;
                    }
                    // Parse field: name: Type,
                    let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let field_name = parts[0].trim().to_string();
                        let field_type = parts[1].trim().trim_end_matches(',').trim().to_string();
                        if !field_name.is_empty() && !field_type.is_empty() {
                            props.push((field_name, field_type));
                        }
                    }
                }
            }
        }
    }

    props
}

fn default_value_for_type(ty: &str) -> &str {
    let ty = ty.trim();
    match ty {
        "String" | "&str" | "&'static str" => "\"\"",
        "bool" => "false",
        "i8" | "i16" | "i32" | "i64" | "isize" => "0",
        "u8" | "u16" | "u32" | "u64" | "usize" => "0",
        "f32" | "f64" => "0.0",
        _ if ty.starts_with("Vec<") => "vec![]",
        _ if ty.starts_with("Option<") => "None",
        _ => "Default::default()",
    }
}

/// Generate a complete test file from parsed components.
fn generate_test_file(components: &[ParsedComponent], file_stem: &str) -> String {
    let mut out = String::new();

    out.push_str("use rye::prelude::*;\n");
    out.push_str("use rye_testing::*;\n");
    out.push_str("use rye_testing::events::*;\n\n");

    for comp in components {
        let snake = to_snake_case(&comp.name);

        // Props initialization
        let props_init = if comp.props.is_empty() {
            String::new()
        } else {
            let props_type = comp.props_type.as_deref().unwrap_or("Props");
            let fields: Vec<String> = comp
                .props
                .iter()
                .map(|(name, ty)| format!("    {}: {},", name, default_value_for_type(ty)))
                .collect();
            format!(
                "    let props = {} {{\n{}\n    }};\n",
                props_type,
                fields.join("\n")
            )
        };

        let render_call = if comp.props.is_empty() {
            format!("{}()", comp.name)
        } else {
            format!("{}(props)", comp.name)
        };

        // Test 1: renders without panic
        out.push_str(&format!(
            "#[test]\nfn test_{}_renders() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n    // Verify it renders without panic\n}}\n\n",
            snake, props_init, render_call
        ));

        // Test 2: has correct root tag
        out.push_str(&format!(
            "#[test]\nfn test_{}_root_tag() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n    // TODO: verify root element tag\n    // let root = renderer.root_element();\n    // assert!(!root.children.is_empty());\n}}\n\n",
            snake, props_init, render_call
        ));

        // Test 3: props validation (if component has props)
        if !comp.props.is_empty() {
            let props_type = comp.props_type.as_deref().unwrap_or("Props");
            let fields: Vec<String> = comp
                .props
                .iter()
                .map(|(name, ty)| format!("    {}: {},", name, default_value_for_type(ty)))
                .collect();
            out.push_str(&format!(
                "#[test]\nfn test_{}_with_props() {{\n    let props = {} {{\n{}\n    }};\n    let _element = {}(props);\n    // TODO: verify props are rendered correctly\n}}\n\n",
                snake, props_type, fields.join("\n"), comp.name
            ));
        }

        // Test 4: event handling
        out.push_str(&format!(
            "#[test]\nfn test_{}_event_handling() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n    // TODO: test event handling\n    // let buttons = get_by_tag(&renderer.root(), \"button\");\n    // if !buttons.is_empty() {{\n    //     fire_click(&mut renderer, &buttons[0]);\n    // }}\n}}\n\n",
            snake, props_init, render_call
        ));

        // Test 5: island-specific test
        if comp.is_island {
            out.push_str(&format!(
                "#[test]\nfn test_{}_island_marker() {{\n    // TODO: verify island hydration marker is present\n    // Island components should have data-rye-island attribute\n}}\n\n",
                snake
            ));
        }
    }

    out
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else {
            result.push(c);
        }
    }
    result
}

fn print_test_generate_help() {
    println!("rpg test --generate — Generate test scaffolding for components");
    println!();
    println!("USAGE:");
    println!("  rpg test --generate <file>       Generate tests for a component file");
    println!(
        "  rpg test --generate --all        Generate tests for all components in src/components"
    );
    println!("  rpg test --generate --dir <dir>  Generate tests for all components in a directory");
    println!();
    println!("GENERATED TESTS:");
    println!("  - Renders without panic");
    println!("  - Has correct root tag");
    println!("  - Props are accepted (if component has props)");
    println!("  - Event handling (if interactive)");
    println!("  - Island marker (if island component)");
    println!();
    println!("EXAMPLES:");
    println!("  rpg test --generate src/components/button.rs");
    println!("  rpg test --generate --all");
    println!("  rpg test --generate --dir src/components");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_components_simple() {
        let source = r#"
#[component]
fn Button() {
    div { "Click" }
}
"#;
        let components = parse_components(source);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name, "Button");
        assert!(components[0].props.is_empty());
        assert!(!components[0].is_island);
    }

    #[test]
    fn test_parse_components_with_props() {
        let source = r#"
struct ButtonProps {
    label: String,
    disabled: bool,
}

#[component]
fn Button(props: ButtonProps) {
    div { "Click" }
}
"#;
        let components = parse_components(source);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].name, "Button");
        assert_eq!(components[0].props_type.as_deref(), Some("ButtonProps"));
        assert_eq!(components[0].props.len(), 2);
        assert_eq!(
            components[0].props[0],
            ("label".to_string(), "String".to_string())
        );
        assert_eq!(
            components[0].props[1],
            ("disabled".to_string(), "bool".to_string())
        );
    }

    #[test]
    fn test_parse_components_island() {
        let source = r#"
#[rye::island]
#[component]
fn Widget() {
    div { "Widget" }
}
"#;
        let components = parse_components(source);
        assert_eq!(components.len(), 1);
        assert!(components[0].is_island);
    }

    #[test]
    fn test_parse_components_multiple() {
        let source = r#"
#[component]
fn Header() {
    div { "Header" }
}

#[component]
fn Footer() {
    div { "Footer" }
}
"#;
        let components = parse_components(source);
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].name, "Header");
        assert_eq!(components[1].name, "Footer");
    }

    #[test]
    fn test_generate_test_file_simple() {
        let components = vec![ParsedComponent {
            name: "Button".to_string(),
            props_type: None,
            props: vec![],
            is_island: false,
        }];
        let test = generate_test_file(&components, "button");
        assert!(test.contains("test_button_renders"));
        assert!(test.contains("test_button_root_tag"));
        assert!(test.contains("test_button_event_handling"));
        assert!(test.contains("TestRenderer::new()"));
        assert!(test.contains("Button()"));
    }

    #[test]
    fn test_generate_test_file_with_props() {
        let components = vec![ParsedComponent {
            name: "Card".to_string(),
            props_type: Some("CardProps".to_string()),
            props: vec![
                ("title".to_string(), "String".to_string()),
                ("count".to_string(), "i32".to_string()),
            ],
            is_island: false,
        }];
        let test = generate_test_file(&components, "card");
        assert!(test.contains("test_card_renders"));
        assert!(test.contains("test_card_with_props"));
        assert!(test.contains("CardProps"));
        assert!(test.contains("title: \"\""));
        assert!(test.contains("count: 0"));
        assert!(test.contains("Card(props)"));
    }

    #[test]
    fn test_generate_test_file_island() {
        let components = vec![ParsedComponent {
            name: "Widget".to_string(),
            props_type: None,
            props: vec![],
            is_island: true,
        }];
        let test = generate_test_file(&components, "widget");
        assert!(test.contains("test_widget_island_marker"));
        assert!(test.contains("data-rye-island"));
    }

    #[test]
    fn test_default_value_for_type() {
        assert_eq!(default_value_for_type("String"), "\"\"");
        assert_eq!(default_value_for_type("bool"), "false");
        assert_eq!(default_value_for_type("i32"), "0");
        assert_eq!(default_value_for_type("Vec<String>"), "vec![]");
        assert_eq!(default_value_for_type("Option<i32>"), "None");
    }

    #[test]
    fn test_extract_fn_name() {
        assert_eq!(extract_fn_name("fn Button() {").as_deref(), Some("Button"));
        assert_eq!(
            extract_fn_name("fn MyComponent(props: Props) {").as_deref(),
            Some("MyComponent")
        );
        assert_eq!(
            extract_fn_name("fn counter() {").as_deref(),
            Some("counter")
        );
    }

    #[test]
    fn test_extract_struct_name() {
        assert_eq!(
            extract_struct_name("struct ButtonProps {").as_deref(),
            Some("ButtonProps")
        );
        assert_eq!(
            extract_struct_name("struct CardProps{").as_deref(),
            Some("CardProps")
        );
    }
}
