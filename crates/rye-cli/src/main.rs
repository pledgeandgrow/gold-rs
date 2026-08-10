//! # rye CLI (`rpg`)
//!
//! Command-line tool for rye — new, dev, build, test, deploy, add, upgrade.
//! Binary name is `rpg` (rye project generator).

mod build;
mod bundle;
mod dev_server;
mod doctor;
mod doctor_ext;
mod ecosystem;
mod editor_ext;
mod explain;
mod generate;
mod init_wizard;
mod lint;
mod playground;
mod profile;
mod scaffold;
mod test;
mod test_gen;
mod upgrade_ext;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "new" => cmd_new(&args[2..]),
        "dev" => cmd_dev(&args[2..]),
        "build" => build::run(&args[2..]),
        "test" => test::run(&args[2..]),
        "deploy" => cmd_deploy(&args[2..]),
        "add" => cmd_add(&args[2..]),
        "upgrade" => cmd_upgrade(&args[2..]),
        "explain" => explain::run(&args[2..]),
        "scaffold" => scaffold::run(&args[2..]),
        "lint" => lint::run(&args[2..]),
        "doctor" => doctor::run(&args[2..]),
        "playground" => playground::run(&args[2..]),
        "profile" => profile::run(&args[2..]),
        "bundle" => bundle::run(&args[2..]),
        "init" => init_wizard::run(&args[2..]),
        "generate" => generate::run(&args[2..]),
        "monorepo" => ecosystem::run_monorepo(&args[2..]),
        "publish" => ecosystem::run_publish(&args[2..]),
        "theme" => ecosystem::run_theme(&args[2..]),
        "docs" => ecosystem::run_docs(&args[2..]),
        "ci" => ecosystem::run_ci(&args[2..]),
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-v" => print_version(),
        cmd => {
            eprintln!("Unknown command: {cmd}");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("rpg — rye CLI — A cross-platform UI framework for Rust");
    println!();
    println!("USAGE:");
    println!("  rpg <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  new       Scaffold a new project");
    println!("  dev       Start dev server with hot reloading");
    println!("  build     Build for production (per target)");
    println!("  test      Run tests (use --generate to scaffold tests)");
    println!("  deploy    Deploy to web, desktop, or mobile");
    println!("  add       Add a component, plugin, or package");
    println!("  upgrade   Upgrade the framework with codemods");
    println!("  explain   Explain error codes (R001-R899)");
    println!("  scaffold  Generate component/page/store/action structures");
    println!("  lint      AI-aware linter (use --ai flag)");
    println!("  doctor    Project health check");
    println!("  playground  Online code editor");
    println!("  profile   Performance profiler");
    println!("  bundle    Size analyzer with tree map");
    println!("  init      Interactive project wizard");
    println!("  generate  Generate code from OpenAPI/schema");
    println!("  monorepo  Workspace management");
    println!("  publish   Publish component library");
    println!("  theme     Design token CLI");
    println!("  docs      Local documentation server");
    println!("  ci        CI/CD template generator");
    println!("  help      Show this help message");
    println!("  version   Show version info");
    println!();
    println!("EXAMPLES:");
    println!("  rpg new my-app --template web");
    println!("  rpg dev --port 3000");
    println!("  rpg build --target web");
    println!("  rpg add component Counter");
    println!("  rpg add component Dialog --with-props");
    println!("  rpg add component Card --with-props --with-style");
    println!("  rpg explain R001");
    println!("  rpg explain R800 --json");
    println!("  rpg scaffold component Button --props label:String --style --test");
    println!("  rpg test --generate src/components/button.rs");
    println!("  rpg lint --ai src/components/button.rs");
    println!("  rpg doctor");
}

fn print_version() {
    println!("rpg (rye) 0.1.0");
}

fn cmd_new(args: &[String]) {
    let template = args
        .iter()
        .find(|a| a.starts_with("--template"))
        .map(|a| &a[11..])
        .unwrap_or("web");
    let name = args.iter().last().map(|s| s.as_str()).unwrap_or("my-app");
    println!(
        "Creating new rye project: {} (template: {})",
        name, template
    );
    // TODO: scaffold project from template
}

fn cmd_dev(args: &[String]) {
    let port: u16 = args
        .iter()
        .find(|a| a.starts_with("--port"))
        .and_then(|a| a[7..].parse().ok())
        .unwrap_or(8080);

    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let static_dir = project_root.join("static");

    let config = dev_server::DevServerConfig {
        port,
        project_root,
        static_dir,
        pkg_name: "rye_app".to_string(),
        debounce_ms: 200,
    };

    dev_server::start_server(config);
}

fn cmd_deploy(args: &[String]) {
    let target = args
        .iter()
        .find(|a| a.starts_with("--target"))
        .map(|a| &a[9..])
        .unwrap_or("web");
    println!("Deploying to: {}", target);
    // TODO: deploy
}

fn cmd_add(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg add <component|plugin|package> <name> [options]");
        eprintln!("  rpg add component Counter");
        eprintln!("  rpg add component Dialog --with-props");
        eprintln!("  rpg add component Card --with-props --with-style");
        eprintln!("  rpg add @rye/ui");
        return;
    }

    match args[0].as_str() {
        "component" => cmd_add_component(&args[1..]),
        "plugin" => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
            println!("Adding plugin: {}", name);
            // TODO: add plugin
        }
        package if package.starts_with('@') || package.starts_with('-') => {
            println!("Adding package: {}", package);
            // TODO: add from registry
        }
        other => {
            eprintln!(
                "Unknown add target: {}. Use 'component', 'plugin', or a package name.",
                other
            );
        }
    }
}

fn cmd_add_component(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: rpg add component <Name> [options]");
        eprintln!("Options:");
        eprintln!("  --with-props    Generate a Props struct");
        eprintln!("  --with-style    Generate a scoped style block");
        eprintln!("  --path <path>   Custom file path (default: src/components/<name>.rs)");
        return;
    }

    let name = &args[0];
    let with_props = args.iter().any(|a| a == "--with-props");
    let with_style = args.iter().any(|a| a == "--with-style");
    let custom_path = args
        .iter()
        .position(|a| a == "--path")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let snake_name = to_snake_case(name);
    let file_path = custom_path
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("src/components/{}.rs", snake_name));

    let code = if with_props && with_style {
        generate_component_with_props_and_style(name, &snake_name)
    } else if with_props {
        generate_component_with_props(name, &snake_name)
    } else if with_style {
        generate_component_with_style(&snake_name)
    } else {
        generate_component_simple(name)
    };

    let path = Path::new(&file_path);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(&path, &code) {
        Ok(_) => {
            println!("Created component: {} -> {}", name, file_path);
            register_in_mod_file(&snake_name);
        }
        Err(e) => {
            eprintln!("Failed to create component file: {}", e);
            std::process::exit(1);
        }
    }
}

fn generate_component_simple(name: &str) -> String {
    format!(
        r#"use rye::prelude::*;

#[component]
fn {name}() {{
    // TODO: implement your component
    div {{
        "{name}"
    }}
}}
"#,
        name = name
    )
}

fn generate_component_with_props(name: &str, snake_name: &str) -> String {
    let pascal_name = to_pascal_case(name);
    format!(
        r#"use rye::prelude::*;

#[derive(Props)]
struct {pascal_name}Props {{
    // TODO: add your props here
    // Example:
    // title: String,
    // #[prop(optional)]
    // open: bool,
}}

#[component]
fn {pascal_name}(props: {pascal_name}Props) {{
    // TODO: implement your component
    div {{
        "{pascal_name}"
    }}
}}
"#,
        pascal_name = pascal_name
    )
}

fn generate_component_with_style(snake_name: &str) -> String {
    format!(
        r#"use rye::prelude::*;

style! {{
    .{snake_name} {{
        // TODO: add your styles here
    }}
}}

#[component]
fn {name}() {{
    div {{
        class: "{snake_name}",
        "{name}"
    }}
}}
"#,
        name = to_pascal_case(snake_name),
        snake_name = snake_name
    )
}

fn generate_component_with_props_and_style(name: &str, snake_name: &str) -> String {
    let pascal_name = to_pascal_case(name);
    format!(
        r#"use rye::prelude::*;

#[derive(Props)]
struct {pascal_name}Props {{
    // TODO: add your props here
}}

style! {{
    .{snake_name} {{
        // TODO: add your styles here
    }}
}}

#[component]
fn {pascal_name}(props: {pascal_name}Props) {{
    div {{
        class: "{snake_name}",
        "{pascal_name}"
    }}
}}
"#,
        pascal_name = pascal_name,
        snake_name = snake_name
    )
}

fn register_in_mod_file(snake_name: &str) {
    let mod_path = "src/components/mod.rs";
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
        println!("Registered in src/components/mod.rs");
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

fn cmd_upgrade(_args: &[String]) {
    println!("Upgrading rye...");
    // TODO: upgrade with codemods
}
