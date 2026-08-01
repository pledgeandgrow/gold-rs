//! Scaffold code generation for MCP server (no filesystem I/O).

pub fn parse_props(spec: &str) -> Vec<(String, String)> {
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

pub fn generate_component_code(
    name: &str,
    props: &[(String, String)],
    with_style: bool,
    is_island: bool,
) -> String {
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);
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
        format!("style! {{\n    .{} {{\n        /* TODO: add styles */\n    }}\n}}\n\n", snake)
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

pub fn generate_component_test(
    name: &str,
    props: &[(String, String)],
) -> String {
    let pascal = to_pascal_case(name);
    let snake = to_snake_case(name);

    let props_init = if props.is_empty() {
        String::new()
    } else {
        let fields: Vec<String> = props
            .iter()
            .map(|(name, ty)| format!("    {}: {},", name, default_value_for_type(ty)))
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
}}

#[test]
fn test_{snake}_has_correct_tag() {{
    let mut renderer = TestRenderer::new();
{props_init}
    let element = {render_call};
    // TODO: verify the root element tag
}}
"#,
        snake = snake,
        props_init = props_init,
        render_call = render_call,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_props() {
        let props = parse_props("name:String,count:i32");
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_generate_component_no_props() {
        let code = generate_component_code("Button", &[], false, false);
        assert!(code.contains("#[component]"));
        assert!(code.contains("fn Button()"));
    }

    #[test]
    fn test_generate_component_with_props() {
        let props = vec![("label".to_string(), "String".to_string())];
        let code = generate_component_code("Button", &props, false, false);
        assert!(code.contains("ButtonProps"));
        assert!(code.contains("label: String"));
    }

    #[test]
    fn test_generate_component_island() {
        let code = generate_component_code("Widget", &[], false, true);
        assert!(code.contains("#[rye::island]"));
    }

    #[test]
    fn test_generate_component_test() {
        let props = vec![("label".to_string(), "String".to_string())];
        let test = generate_component_test("Button", &props);
        assert!(test.contains("test_button_renders"));
        assert!(test.contains("ButtonProps"));
    }
}
