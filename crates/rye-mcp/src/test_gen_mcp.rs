//! Test generation for MCP server (no filesystem I/O).
//! Parses component source code and generates test scaffolding.

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

struct ParsedComponent {
    name: String,
    props_type: Option<String>,
    props: Vec<(String, String)>,
    is_island: bool,
}

fn extract_fn_name(line: &str) -> Option<String> {
    let after_fn = line.find("fn ")?;
    let rest = &line[after_fn + 3..];
    let name = rest
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn extract_props_type_from_fn(line: &str) -> Option<String> {
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
                    if trimmed.is_empty() || trimmed.starts_with("//") {
                        continue;
                    }
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

fn parse_components(source: &str) -> Vec<ParsedComponent> {
    let mut components = Vec::new();
    let mut props_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("struct ") && trimmed.contains("Props") {
            if let Some(name) = extract_struct_name(trimmed) {
                let props = extract_props_fields(source, &name);
                props_map.insert(name, props);
            }
        }
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut consumed = vec![false; lines.len()];
    for i in 0..lines.len() {
        if consumed[i] {
            continue;
        }
        let line = lines[i].trim();
        let is_island_attr = line.starts_with("#[rye::island]") || line.starts_with("#[island]");
        let is_component_attr = line.starts_with("#[component]");

        if is_island_attr || is_component_attr {
            consumed[i] = true;
            let mut found_island = is_island_attr;
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

pub fn generate_test_from_source(source: &str) -> String {
    let components = parse_components(source);

    if components.is_empty() {
        return "// No components found in source code\n".to_string();
    }

    let mut out = String::new();
    out.push_str("use rye::prelude::*;\n");
    out.push_str("use rye_testing::*;\n");
    out.push_str("use rye_testing::events::*;\n\n");

    for comp in &components {
        let snake = to_snake_case(&comp.name);

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

        out.push_str(&format!(
            "#[test]\nfn test_{}_renders() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n}}\n\n",
            snake, props_init, render_call
        ));
        out.push_str(&format!(
            "#[test]\nfn test_{}_root_tag() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n}}\n\n",
            snake, props_init, render_call
        ));

        if !comp.props.is_empty() {
            let props_type = comp.props_type.as_deref().unwrap_or("Props");
            let fields: Vec<String> = comp
                .props
                .iter()
                .map(|(name, ty)| format!("    {}: {},", name, default_value_for_type(ty)))
                .collect();
            out.push_str(&format!(
                "#[test]\nfn test_{}_with_props() {{\n    let props = {} {{\n{}\n    }};\n    let _element = {}(props);\n}}\n\n",
                snake, props_type, fields.join("\n"), comp.name
            ));
        }

        out.push_str(&format!(
            "#[test]\nfn test_{}_event_handling() {{\n    let mut renderer = TestRenderer::new();\n{}    let _element = {};\n}}\n\n",
            snake, props_init, render_call
        ));

        if comp.is_island {
            out.push_str(&format!(
                "#[test]\nfn test_{}_island_marker() {{\n    // TODO: verify island hydration marker\n}}\n\n",
                snake
            ));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let source = "#[component]\nfn Button() { div { \"Hi\" } }";
        let comps = parse_components(source);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].name, "Button");
    }

    #[test]
    fn test_parse_with_props() {
        let source = r#"
struct ButtonProps {
    label: String,
}

#[component]
fn Button(props: ButtonProps) { div { } }
"#;
        let comps = parse_components(source);
        assert_eq!(comps.len(), 1);
        assert_eq!(comps[0].props.len(), 1);
        assert_eq!(comps[0].props[0].0, "label");
    }

    #[test]
    fn test_parse_island() {
        let source = "#[rye::island]\n#[component]\nfn Widget() { div { } }";
        let comps = parse_components(source);
        assert_eq!(comps.len(), 1);
        assert!(comps[0].is_island);
    }

    #[test]
    fn test_generate_test_simple() {
        let source = "#[component]\nfn Button() { div { \"Hi\" } }";
        let test = generate_test_from_source(source);
        assert!(test.contains("test_button_renders"));
        assert!(test.contains("test_button_root_tag"));
        assert!(test.contains("test_button_event_handling"));
    }

    #[test]
    fn test_generate_test_no_components() {
        let test = generate_test_from_source("// nothing here");
        assert!(test.contains("No components found"));
    }

    #[test]
    fn test_generate_test_island() {
        let source = "#[rye::island]\n#[component]\nfn Widget() { div { } }";
        let test = generate_test_from_source(source);
        assert!(test.contains("test_widget_island_marker"));
    }
}
