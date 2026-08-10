//! `rpg scaffold` CLI command — Goal 154.
//!
//! AI-friendly scaffolding that generates complete component structures
//! with props, styles, tests, and registration. Designed so AI agents
//! can modify generated files rather than writing from scratch.

use std::fs;
use std::path::Path;

/// Run the `rpg scaffold` command.
///
/// Usage:
///   rpg scaffold component <Name> [options]
///   rpg scaffold page <Name> [options]
///   rpg scaffold store <Name> [options]
///   rpg scaffold action <Name> [options]
///
/// Options:
///   --props <field:type,...>  Generate Props struct with typed fields
///   --style                   Generate scoped style block
///   --test                    Generate test file
///   --island                  Mark as island component
///   --path <path>             Custom output path
pub fn run(args: &[String]) {
    if args.is_empty() {
        print_scaffold_help();
        return;
    }

    match args[0].as_str() {
        "component" => scaffold_component(&args[1..]),
        "page" => scaffold_page(&args[1..]),
        "store" => scaffold_store(&args[1..]),
        "action" => scaffold_action(&args[1..]),
        other => {
            eprintln!(
                "Unknown scaffold target: {}. Use 'component', 'page', 'store', or 'action'.",
                other
            );
            print_scaffold_help();
        }
    }
}

/// Parse --props flag value: "name:String,count:i32,active:bool"
fn parse_props(spec: &str) -> Vec<(String, String)> {
    spec.split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Scaffold a component with full structure.
fn scaffold_component(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg scaffold component <Name> [options]");
        eprintln!("Options:");
        eprintln!("  --props <field:type,...>  Generate Props struct with typed fields");
        eprintln!("  --style                   Generate scoped style block");
        eprintln!("  --test                    Generate test file");
        eprintln!("  --island                  Mark as island component");
        eprintln!("  --path <path>             Custom output path");
        return;
    }

    let name = &args[0];
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let props_spec = args
        .iter()
        .position(|a| a == "--props")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");

    let with_style = args.iter().any(|a| a == "--style");
    let with_test = args.iter().any(|a| a == "--test");
    let is_island = args.iter().any(|a| a == "--island");
    let custom_path = args
        .iter()
        .position(|a| a == "--path")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let props = parse_props(props_spec);
    let file_path = custom_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("src/components/{}.rs", snake));

    // Generate component file
    let code = generate_component_code(&pascal, &snake, &props, with_style, is_island);

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, &code) {
        Ok(_) => {
            println!("Created component: {} -> {}", pascal, file_path);
            register_in_mod_file(&snake, "src/components/mod.rs");

            if with_test {
                let test_path = format!("tests/{}_test.rs", snake);
                let test_code = generate_component_test(&pascal, &snake, &props);
                if let Some(parent) = Path::new(&test_path).parent() {
                    let _ = fs::create_dir_all(parent);
                }
                match fs::write(&test_path, &test_code) {
                    Ok(_) => println!("Created test: {}", test_path),
                    Err(e) => eprintln!("Failed to create test file: {}", e),
                }
            }

            if is_island {
                println!("Marked as island — will be hydrated client-side only.");
            }
        }
        Err(e) => {
            eprintln!("Failed to create component file: {}", e);
            std::process::exit(1);
        }
    }
}

/// Scaffold a page component (with route registration hint).
fn scaffold_page(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg scaffold page <Name> [options]");
        eprintln!("Options:");
        eprintln!("  --props <field:type,...>  Generate Props struct");
        eprintln!("  --path <path>             Custom output path");
        eprintln!("  --route <path>            Route path (e.g. /about)");
        return;
    }

    let name = &args[0];
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let props_spec = args
        .iter()
        .position(|a| a == "--props")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");

    let default_route = format!("/{}", snake);
    let route = args
        .iter()
        .position(|a| a == "--route")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or(&default_route);

    let custom_path = args
        .iter()
        .position(|a| a == "--path")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let props = parse_props(props_spec);
    let file_path = custom_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("src/pages/{}.rs", snake));

    let code = generate_page_code(&pascal, &snake, &props, route);

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, &code) {
        Ok(_) => {
            println!(
                "Created page: {} -> {} (route: {})",
                pascal, file_path, route
            );
            register_in_mod_file(&snake, "src/pages/mod.rs");
        }
        Err(e) => {
            eprintln!("Failed to create page file: {}", e);
            std::process::exit(1);
        }
    }
}

/// Scaffold a signal store.
fn scaffold_store(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg scaffold store <Name> [options]");
        eprintln!("Options:");
        eprintln!("  --fields <field:type,...>  Store fields");
        eprintln!("  --path <path>              Custom output path");
        return;
    }

    let name = &args[0];
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let fields_spec = args
        .iter()
        .position(|a| a == "--fields")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");

    let custom_path = args
        .iter()
        .position(|a| a == "--path")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let fields = parse_props(fields_spec);
    let file_path = custom_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("src/stores/{}.rs", snake));

    let code = generate_store_code(&pascal, &snake, &fields);

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, &code) {
        Ok(_) => {
            println!("Created store: {} -> {}", pascal, file_path);
            register_in_mod_file(&snake, "src/stores/mod.rs");
        }
        Err(e) => {
            eprintln!("Failed to create store file: {}", e);
            std::process::exit(1);
        }
    }
}

/// Scaffold a server action.
fn scaffold_action(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg scaffold action <Name> [options]");
        eprintln!("Options:");
        eprintln!("  --params <field:type,...>  Action parameters");
        eprintln!(
            "  --returns <type>           Return type (default: Result<String, ServerError>)"
        );
        eprintln!("  --path <path>              Custom output path");
        return;
    }

    let name = &args[0];
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let params_spec = args
        .iter()
        .position(|a| a == "--params")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");

    let returns = args
        .iter()
        .position(|a| a == "--returns")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("Result<String, ServerError>");

    let custom_path = args
        .iter()
        .position(|a| a == "--path")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let params = parse_props(params_spec);
    let file_path = custom_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("src/actions/{}.rs", snake));

    let code = generate_action_code(&pascal, &snake, &params, returns);

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, &code) {
        Ok(_) => {
            println!("Created server action: {} -> {}", pascal, file_path);
            register_in_mod_file(&snake, "src/actions/mod.rs");
        }
        Err(e) => {
            eprintln!("Failed to create action file: {}", e);
            std::process::exit(1);
        }
    }
}

// ===== Code generators =====

fn generate_component_code(
    pascal: &str,
    snake: &str,
    props: &[(String, String)],
    with_style: bool,
    is_island: bool,
) -> String {
    let island_attr = if is_island { "#[rye::island]\n" } else { "" };

    let props_struct = if props.is_empty() {
        String::new()
    } else {
        let fields: Vec<String> = props
            .iter()
            .map(|(name, ty)| format!("    {}: {},", name, ty))
            .collect();
        format!(
            "#[derive(Props)]\nstruct {pascal}Props {{\n{fields}\n}}\n\n",
            pascal = pascal,
            fields = fields.join("\n")
        )
    };

    let style_block = if with_style {
        format!(
            "style! {{\n    .{} {{\n        /* TODO: add styles */\n    }}\n}}\n\n",
            snake
        )
    } else {
        String::new()
    };

    let class_attr = if with_style {
        format!("\n        class: \"{}\",", snake)
    } else {
        String::new()
    };

    let props_param = if props.is_empty() {
        String::new()
    } else {
        format!("props: {}Props", pascal)
    };

    format!(
        r#"use rye::prelude::*;

{props_struct}{style_block}{island_attr}#[component]
fn {pascal}({props_param}) {{
    div {{{class_attr}
        "{pascal}"
    }}
}}
"#,
        props_struct = props_struct,
        style_block = style_block,
        island_attr = island_attr,
        pascal = pascal,
        props_param = props_param,
        class_attr = class_attr,
    )
}

fn generate_component_test(pascal: &str, snake: &str, props: &[(String, String)]) -> String {
    let props_init = if props.is_empty() {
        String::new()
    } else {
        let fields: Vec<String> = props
            .iter()
            .map(|(name, ty)| {
                let default = default_value_for_type(ty);
                format!("    {}: {},", name, default)
            })
            .collect();
        format!(
            "    let props = {pascal}Props {{\n{fields}\n    }};\n",
            pascal = pascal,
            fields = fields.join("\n")
        )
    };

    let render_call = if props.is_empty() {
        format!("{}()", pascal)
    } else {
        format!("{}(props)", pascal)
    };

    format!(
        r#"use rye::prelude::*;
use rye_testing::*;

#[test]
fn test_{snake}_renders() {{
    let mut renderer = TestRenderer::new();
{props_init}
    let element = {render_call};
    // TODO: assert on rendered output
    // let root = renderer.root_element();
    // assert!(root.text_content().contains("{pascal}"));
}}

#[test]
fn test_{snake}_has_correct_tag() {{
    let mut renderer = TestRenderer::new();
{props_init}
    let element = {render_call};
    // TODO: verify the root element tag
    // let divs = get_by_tag(&renderer.root(), "div");
    // assert!(!divs.is_empty());
}}
"#,
        snake = snake,
        props_init = props_init,
        render_call = render_call,
        pascal = pascal,
    )
}

fn generate_page_code(
    pascal: &str,
    snake: &str,
    props: &[(String, String)],
    route: &str,
) -> String {
    let props_struct = if props.is_empty() {
        String::new()
    } else {
        let fields: Vec<String> = props
            .iter()
            .map(|(name, ty)| format!("    {}: {},", name, ty))
            .collect();
        format!(
            "#[derive(Props)]\nstruct {pascal}Props {{\n{fields}\n}}\n\n",
            pascal = pascal,
            fields = fields.join("\n")
        )
    };

    let props_param = if props.is_empty() {
        String::new()
    } else {
        format!("props: {}Props", pascal)
    };

    format!(
        r#"use rye::prelude::*;

// Route: {route}
{props_struct}#[component]
fn {pascal}({props_param}) {{
    div {{
        h1 {{ "{pascal}" }}
        p {{ "TODO: implement {pascal} page" }}
    }}
}}
"#,
        route = route,
        props_struct = props_struct,
        pascal = pascal,
        props_param = props_param,
    )
}

fn generate_store_code(pascal: &str, snake: &str, fields: &[(String, String)]) -> String {
    let field_defs: Vec<String> = fields
        .iter()
        .map(|(name, ty)| format!("    pub {}: Signal<{}>,", name, ty))
        .collect();

    let field_inits: Vec<String> = fields
        .iter()
        .map(|(name, ty)| {
            let default = default_value_for_type(ty);
            format!("            {}: Signal::new({}),", name, default)
        })
        .collect();

    let field_accessors: Vec<String> = fields
        .iter()
        .map(|(name, ty)| {
            format!(
                "    pub fn {name}(&self) -> {ty} {{\n        self.{name}.get()\n    }}\n\n    pub fn set_{name}(&self, val: {ty}) {{\n        self.{name}.set(val);\n    }}",
                name = name,
                ty = ty
            )
        })
        .collect();

    format!(
        r#"use rye::prelude::*;
use rye_signals::Signal;

pub struct {pascal} {{
{field_defs}
}}

impl {pascal} {{
    pub fn new() -> Self {{
        Self {{
{field_inits}
        }}
    }}

{field_accessors}
}}

impl Default for {pascal} {{
    fn default() -> Self {{
        Self::new()
    }}
}}
"#,
        pascal = pascal,
        field_defs = field_defs.join("\n"),
        field_inits = field_inits.join("\n"),
        field_accessors = field_accessors.join("\n\n"),
    )
}

fn generate_action_code(
    pascal: &str,
    snake: &str,
    params: &[(String, String)],
    returns: &str,
) -> String {
    let params_str: Vec<String> = params
        .iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect();
    let param_names: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();

    let body = if returns.starts_with("Result") {
        format!(
            "    // TODO: implement server logic\n    Ok(\"{}\".to_string())",
            snake
        )
    } else {
        format!("    // TODO: implement server logic\n    Default::default()")
    };

    format!(
        r#"use rye::prelude::*;

#[server]
pub async fn {snake}({params}) -> {returns} {{
{body}
}}
"#,
        snake = snake,
        params = params_str.join(", "),
        returns = returns,
        body = body,
    )
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

fn register_in_mod_file(snake_name: &str, mod_path: &str) {
    let mod_content = match fs::read_to_string(mod_path) {
        Ok(content) => content,
        Err(_) => String::new(),
    };

    let module_line = format!("mod {};", snake_name);
    if mod_content.contains(&module_line) {
        return;
    }

    let new_content = if mod_content.is_empty() {
        module_line + "\n"
    } else {
        format!("{}\n{}", mod_content.trim_end(), module_line)
    };

    if fs::write(mod_path, new_content).is_ok() {
        println!("Registered in {}", mod_path);
    }
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

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn print_scaffold_help() {
    println!("rpg scaffold — Generate AI-friendly component structures");
    println!();
    println!("USAGE:");
    println!("  rpg scaffold <TARGET> <Name> [options]");
    println!();
    println!("TARGETS:");
    println!("  component    Scaffold a component with props, style, and test");
    println!("  page         Scaffold a page component with route");
    println!("  store        Scaffold a signal store");
    println!("  action       Scaffold a server action");
    println!();
    println!("OPTIONS:");
    println!("  --props <field:type,...>   Generate typed props (component, page)");
    println!("  --fields <field:type,...>  Store fields (store)");
    println!("  --params <field:type,...>  Action parameters (action)");
    println!("  --returns <type>           Return type (action)");
    println!("  --style                    Include scoped styles (component)");
    println!("  --test                     Generate test file (component)");
    println!("  --island                   Mark as island (component)");
    println!("  --route <path>             Route path (page)");
    println!("  --path <path>              Custom output path");
    println!();
    println!("EXAMPLES:");
    println!("  rpg scaffold component Button --props label:String,disabled:bool --style --test");
    println!("  rpg scaffold page About --route /about");
    println!("  rpg scaffold store UserStore --fields name:String,count:i32");
    println!("  rpg scaffold action GetUser --params id:u32 --returns Result<User,ServerError>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_props() {
        let props = parse_props("name:String,count:i32,active:bool");
        assert_eq!(props.len(), 3);
        assert_eq!(props[0], ("name".to_string(), "String".to_string()));
        assert_eq!(props[1], ("count".to_string(), "i32".to_string()));
        assert_eq!(props[2], ("active".to_string(), "bool".to_string()));
    }

    #[test]
    fn test_parse_props_empty() {
        let props = parse_props("");
        assert!(props.is_empty());
    }

    #[test]
    fn test_parse_props_single() {
        let props = parse_props("name:String");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0], ("name".to_string(), "String".to_string()));
    }

    #[test]
    fn test_default_value_for_type() {
        assert_eq!(default_value_for_type("String"), "\"\"");
        assert_eq!(default_value_for_type("bool"), "false");
        assert_eq!(default_value_for_type("i32"), "0");
        assert_eq!(default_value_for_type("f64"), "0.0");
        assert_eq!(default_value_for_type("Vec<String>"), "vec![]");
        assert_eq!(default_value_for_type("Option<i32>"), "None");
        assert_eq!(default_value_for_type("MyType"), "Default::default()");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("MyButton"), "my_button");
        assert_eq!(to_snake_case("Button"), "button");
        assert_eq!(to_snake_case("HTTPClient"), "h_t_t_p_client");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("my_button"), "MyButton");
        assert_eq!(to_pascal_case("button"), "Button");
        assert_eq!(to_pascal_case("my component"), "MyComponent");
    }

    #[test]
    fn test_generate_component_code_no_props() {
        let code = generate_component_code("Button", "button", &[], false, false);
        assert!(code.contains("#[component]"));
        assert!(code.contains("fn Button()"));
        assert!(!code.contains("Props"));
    }

    #[test]
    fn test_generate_component_code_with_props() {
        let props = vec![
            ("label".to_string(), "String".to_string()),
            ("disabled".to_string(), "bool".to_string()),
        ];
        let code = generate_component_code("Button", "button", &props, false, false);
        assert!(code.contains("ButtonProps"));
        assert!(code.contains("label: String"));
        assert!(code.contains("disabled: bool"));
        assert!(code.contains("props: ButtonProps"));
    }

    #[test]
    fn test_generate_component_code_with_style() {
        let code = generate_component_code("Card", "card", &[], true, false);
        assert!(code.contains("style!"));
        assert!(code.contains(".card"));
        assert!(code.contains("class: \"card\""));
    }

    #[test]
    fn test_generate_component_code_island() {
        let code = generate_component_code("Widget", "widget", &[], false, true);
        assert!(code.contains("#[rye::island]"));
    }

    #[test]
    fn test_generate_component_test() {
        let props = vec![("label".to_string(), "String".to_string())];
        let test = generate_component_test("Button", "button", &props);
        assert!(test.contains("test_button_renders"));
        assert!(test.contains("TestRenderer::new()"));
        assert!(test.contains("ButtonProps"));
        assert!(test.contains("label: \"\""));
    }

    #[test]
    fn test_generate_page_code() {
        let code = generate_page_code("About", "about", &[], "/about");
        assert!(code.contains("Route: /about"));
        assert!(code.contains("fn About()"));
        assert!(code.contains("About"));
    }

    #[test]
    fn test_generate_store_code() {
        let fields = vec![
            ("name".to_string(), "String".to_string()),
            ("count".to_string(), "i32".to_string()),
        ];
        let code = generate_store_code("UserStore", "user_store", &fields);
        assert!(code.contains("pub struct UserStore"));
        assert!(code.contains("pub name: Signal<String>"));
        assert!(code.contains("pub count: Signal<i32>"));
        assert!(code.contains("Signal::new(\"\")"));
        assert!(code.contains("Signal::new(0)"));
        assert!(code.contains("fn name(&self) -> String"));
        assert!(code.contains("fn set_name(&self, val: String)"));
    }

    #[test]
    fn test_generate_action_code() {
        let params = vec![("id".to_string(), "u32".to_string())];
        let code = generate_action_code(
            "GetUser",
            "get_user",
            &params,
            "Result<String, ServerError>",
        );
        assert!(code.contains("#[server]"));
        assert!(code.contains("pub async fn get_user(id: u32) -> Result<String, ServerError>"));
        assert!(code.contains("Ok(\"get_user\".to_string())"));
    }
}
