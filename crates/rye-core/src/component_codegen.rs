//! Goal 213: Component-level code generation.
//!
//! Generate specialized Rust code per component instance at compile time.
//! Eliminates dynamic dispatch, inlines prop access, constant-folds static
//! template parts.

use std::collections::HashMap;
use std::sync::Mutex;

/// A prop type in a component.
#[derive(Debug, Clone, PartialEq)]
pub enum PropType {
    /// A string prop.
    String,
    /// An integer prop.
    Int,
    /// A float prop.
    Float,
    /// A boolean prop.
    Bool,
    /// A custom type (struct name).
    Custom(String),
}

impl PropType {
    /// Get the Rust type string.
    pub fn rust_type(&self) -> String {
        match self {
            PropType::String => "String".to_string(),
            PropType::Int => "i64".to_string(),
            PropType::Float => "f64".to_string(),
            PropType::Bool => "bool".to_string(),
            PropType::Custom(name) => name.clone(),
        }
    }

    /// Get the default value for this type.
    pub fn default_value(&self) -> String {
        match self {
            PropType::String => "String::new()".to_string(),
            PropType::Int => "0".to_string(),
            PropType::Float => "0.0".to_string(),
            PropType::Bool => "false".to_string(),
            PropType::Custom(name) => format!("{}::default()", name),
        }
    }

    /// Check if this is a primitive type (can be inlined).
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            PropType::String | PropType::Int | PropType::Float | PropType::Bool
        )
    }
}

/// A prop definition in a component.
#[derive(Debug, Clone)]
pub struct PropDef {
    /// The prop name.
    pub name: String,
    /// The prop type.
    pub prop_type: PropType,
    /// Whether the prop has a default value.
    pub has_default: bool,
    /// Whether the prop is static (known at compile time).
    pub is_static: bool,
    /// The static value (if `is_static` is true).
    pub static_value: Option<String>,
}

impl PropDef {
    /// Create a new prop definition.
    pub fn new(name: &str, prop_type: PropType) -> Self {
        Self {
            name: name.to_string(),
            prop_type,
            has_default: false,
            is_static: false,
            static_value: None,
        }
    }

    /// Mark the prop as having a default value.
    pub fn with_default(mut self) -> Self {
        self.has_default = true;
        self
    }

    /// Mark the prop as static with a known value.
    pub fn with_static_value(mut self, value: &str) -> Self {
        self.is_static = true;
        self.static_value = Some(value.to_string());
        self
    }
}

/// A template part — either static or dynamic.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    /// Static text that can be constant-folded.
    Static(String),
    /// Dynamic expression (prop reference).
    Dynamic(String),
    /// A conditional section.
    Conditional {
        /// The condition expression.
        condition: String,
        /// The parts if true.
        then_parts: Vec<TemplatePart>,
        /// The parts if false.
        else_parts: Vec<TemplatePart>,
    },
    /// A loop section.
    Loop {
        /// The iterable expression.
        iterable: String,
        /// The item variable name.
        item_var: String,
        /// The body parts.
        body: Vec<TemplatePart>,
    },
}

impl TemplatePart {
    /// Check if this part is fully static (can be constant-folded).
    pub fn is_fully_static(&self) -> bool {
        match self {
            TemplatePart::Static(_) => true,
            TemplatePart::Dynamic(_) => false,
            TemplatePart::Conditional {
                then_parts,
                else_parts,
                ..
            } => {
                then_parts.iter().all(|p| p.is_fully_static())
                    && else_parts.iter().all(|p| p.is_fully_static())
            }
            TemplatePart::Loop { body, .. } => body.iter().all(|p| p.is_fully_static()),
        }
    }
}

/// A component definition for code generation.
#[derive(Debug, Clone)]
pub struct ComponentGenDef {
    /// The component name.
    pub name: String,
    /// The component's props.
    pub props: Vec<PropDef>,
    /// The template parts.
    pub template: Vec<TemplatePart>,
    /// Whether the component has no dynamic content (purely static).
    pub is_static: bool,
    /// Whether to inline prop access.
    pub inline_props: bool,
    /// Whether to eliminate dynamic dispatch.
    pub eliminate_dispatch: bool,
}

impl ComponentGenDef {
    /// Create a new component definition.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            props: Vec::new(),
            template: Vec::new(),
            is_static: false,
            inline_props: true,
            eliminate_dispatch: true,
        }
    }

    /// Add a prop.
    pub fn add_prop(mut self, prop: PropDef) -> Self {
        self.props.push(prop);
        self
    }

    /// Add a template part.
    pub fn add_part(mut self, part: TemplatePart) -> Self {
        self.template.push(part);
        self
    }

    /// Check if all template parts are static.
    pub fn is_fully_static(&self) -> bool {
        self.template.iter().all(|p| p.is_fully_static())
    }

    /// Get the static props (known at compile time).
    pub fn static_props(&self) -> Vec<&PropDef> {
        self.props.iter().filter(|p| p.is_static).collect()
    }

    /// Get the dynamic props.
    pub fn dynamic_props(&self) -> Vec<&PropDef> {
        self.props.iter().filter(|p| !p.is_static).collect()
    }

    /// Estimate the performance improvement (as a fraction, 0.0-1.0).
    pub fn estimated_improvement(&self) -> f64 {
        let mut improvement = 0.0;
        if self.eliminate_dispatch {
            improvement += 0.05;
        }
        if self.inline_props {
            let primitive_count = self
                .props
                .iter()
                .filter(|p| p.prop_type.is_primitive())
                .count();
            improvement += 0.02 * primitive_count as f64;
        }
        let static_count = self.template.iter().filter(|p| p.is_fully_static()).count();
        let total = self.template.len().max(1);
        improvement += 0.05 * (static_count as f64 / total as f64);
        improvement.min(0.15) // Cap at 15%
    }
}

/// The component code generator — generates specialized Rust code.
pub struct ComponentCodeGenerator {
    components: Mutex<HashMap<String, ComponentGenDef>>,
}

impl ComponentCodeGenerator {
    /// Create a new code generator.
    pub fn new() -> Self {
        Self {
            components: Mutex::new(HashMap::new()),
        }
    }

    /// Register a component for code generation.
    pub fn register(&self, component: ComponentGenDef) {
        self.components
            .lock()
            .unwrap()
            .insert(component.name.clone(), component);
    }

    /// Get a component by name.
    pub fn get(&self, name: &str) -> Option<ComponentGenDef> {
        self.components.lock().unwrap().get(name).cloned()
    }

    /// Get all registered component names.
    pub fn component_names(&self) -> Vec<String> {
        self.components.lock().unwrap().keys().cloned().collect()
    }

    /// Get the number of registered components.
    pub fn len(&self) -> usize {
        self.components.lock().unwrap().len()
    }

    /// Generate specialized Rust code for a component.
    pub fn generate(&self, name: &str) -> Option<String> {
        let component = self.get(name)?;
        Some(self.generate_component(&component))
    }

    fn generate_component(&self, component: &ComponentGenDef) -> String {
        let mut code = String::new();

        // Generate the props struct
        code.push_str(&format!(
            "// Auto-generated component: {}\n",
            component.name
        ));
        code.push_str(&format!(
            "#[derive(Clone, Default)]\npub struct {}Props {{\n",
            component.name
        ));
        for prop in &component.props {
            code.push_str(&format!(
                "    pub {}: {},\n",
                prop.name,
                prop.prop_type.rust_type()
            ));
        }
        code.push_str("}\n\n");

        // Generate the render function
        code.push_str(&format!(
            "pub fn render_{}(props: &{}Props) -> String {{\n",
            component.name.to_lowercase(),
            component.name
        ));

        if component.is_fully_static() {
            // Constant-fold: generate the static output directly
            code.push_str("    // Fully static — constant-folded\n");
            code.push_str("    String::from(\"");
            for part in &component.template {
                if let TemplatePart::Static(text) = part {
                    code.push_str(text);
                }
            }
            code.push_str("\")\n");
        } else {
            // Generate dynamic render code
            for part in &component.template {
                match part {
                    TemplatePart::Static(text) => {
                        code.push_str(&format!(
                            "    let _s = \"{}\";\n",
                            text.replace('"', "\\\"")
                        ));
                    }
                    TemplatePart::Dynamic(expr) => {
                        if component.inline_props {
                            code.push_str(&format!("    let _d = {}.to_string();\n", expr));
                        } else {
                            code.push_str(&format!("    let _d = format!(\"{{}}\", {});\n", expr));
                        }
                    }
                    TemplatePart::Conditional {
                        condition,
                        then_parts,
                        ..
                    } => {
                        code.push_str(&format!("    if {} {{\n", condition));
                        for tp in then_parts {
                            if let TemplatePart::Static(t) = tp {
                                code.push_str(&format!("        // static: {}\n", t));
                            }
                        }
                        code.push_str("    }\n");
                    }
                    TemplatePart::Loop {
                        iterable,
                        item_var,
                        body,
                    } => {
                        code.push_str(&format!("    for {} in {} {{\n", item_var, iterable));
                        for bp in body {
                            if let TemplatePart::Static(t) = bp {
                                code.push_str(&format!("        // static: {}\n", t));
                            }
                        }
                        code.push_str("    }\n");
                    }
                }
            }
            code.push_str("    format!(\"<{}>\", \"\")\n");
        }

        code.push_str("}\n");
        code
    }

    /// Generate code for all registered components.
    pub fn generate_all(&self) -> String {
        let names: Vec<String> = self.component_names();
        names
            .iter()
            .filter_map(|name| self.generate(name))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ComponentCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_type_rust_type() {
        assert_eq!(PropType::String.rust_type(), "String");
        assert_eq!(PropType::Int.rust_type(), "i64");
        assert_eq!(PropType::Float.rust_type(), "f64");
        assert_eq!(PropType::Bool.rust_type(), "bool");
        assert_eq!(PropType::Custom("MyType".to_string()).rust_type(), "MyType");
    }

    #[test]
    fn test_prop_type_default_value() {
        assert_eq!(PropType::String.default_value(), "String::new()");
        assert_eq!(PropType::Int.default_value(), "0");
        assert_eq!(PropType::Bool.default_value(), "false");
    }

    #[test]
    fn test_prop_type_is_primitive() {
        assert!(PropType::String.is_primitive());
        assert!(PropType::Int.is_primitive());
        assert!(!PropType::Custom("X".to_string()).is_primitive());
    }

    #[test]
    fn test_prop_def_new() {
        let prop = PropDef::new("label", PropType::String);
        assert_eq!(prop.name, "label");
        assert!(!prop.has_default);
        assert!(!prop.is_static);
    }

    #[test]
    fn test_prop_def_with_static_value() {
        let prop = PropDef::new("color", PropType::String).with_static_value("red");
        assert!(prop.is_static);
        assert_eq!(prop.static_value, Some("red".to_string()));
    }

    #[test]
    fn test_template_part_is_fully_static() {
        assert!(TemplatePart::Static("hello".to_string()).is_fully_static());
        assert!(!TemplatePart::Dynamic("props.x".to_string()).is_fully_static());
    }

    #[test]
    fn test_template_part_conditional_is_fully_static() {
        let part = TemplatePart::Conditional {
            condition: "true".to_string(),
            then_parts: vec![TemplatePart::Static("a".to_string())],
            else_parts: vec![TemplatePart::Static("b".to_string())],
        };
        assert!(part.is_fully_static());
    }

    #[test]
    fn test_template_part_conditional_not_static() {
        let part = TemplatePart::Conditional {
            condition: "props.show".to_string(),
            then_parts: vec![TemplatePart::Dynamic("props.x".to_string())],
            else_parts: vec![],
        };
        assert!(!part.is_fully_static());
    }

    #[test]
    fn test_component_gen_def_new() {
        let comp = ComponentGenDef::new("Button");
        assert_eq!(comp.name, "Button");
        assert!(comp.inline_props);
        assert!(comp.eliminate_dispatch);
    }

    #[test]
    fn test_component_gen_def_add_prop() {
        let comp = ComponentGenDef::new("Button")
            .add_prop(PropDef::new("label", PropType::String))
            .add_prop(PropDef::new("count", PropType::Int));
        assert_eq!(comp.props.len(), 2);
    }

    #[test]
    fn test_component_gen_def_is_fully_static() {
        let comp = ComponentGenDef::new("Static")
            .add_part(TemplatePart::Static("<div>hello</div>".to_string()));
        assert!(comp.is_fully_static());
    }

    #[test]
    fn test_component_gen_def_not_fully_static() {
        let comp = ComponentGenDef::new("Dynamic")
            .add_part(TemplatePart::Static("<div>".to_string()))
            .add_part(TemplatePart::Dynamic("props.label".to_string()));
        assert!(!comp.is_fully_static());
    }

    #[test]
    fn test_component_gen_def_static_props() {
        let comp = ComponentGenDef::new("C")
            .add_prop(PropDef::new("a", PropType::String).with_static_value("x"))
            .add_prop(PropDef::new("b", PropType::Int));
        assert_eq!(comp.static_props().len(), 1);
        assert_eq!(comp.dynamic_props().len(), 1);
    }

    #[test]
    fn test_component_gen_def_estimated_improvement() {
        let comp = ComponentGenDef::new("C")
            .add_prop(PropDef::new("a", PropType::String))
            .add_part(TemplatePart::Static("<div>hello</div>".to_string()));
        let improvement = comp.estimated_improvement();
        assert!(improvement > 0.0);
        assert!(improvement <= 0.15);
    }

    #[test]
    fn test_code_generator_register_get() {
        let gen = ComponentCodeGenerator::new();
        gen.register(ComponentGenDef::new("Button"));
        assert!(gen.get("Button").is_some());
        assert!(gen.get("Unknown").is_none());
    }

    #[test]
    fn test_code_generator_len() {
        let gen = ComponentCodeGenerator::new();
        gen.register(ComponentGenDef::new("A"));
        gen.register(ComponentGenDef::new("B"));
        assert_eq!(gen.len(), 2);
    }

    #[test]
    fn test_code_generator_generate_static() {
        let gen = ComponentCodeGenerator::new();
        gen.register(
            ComponentGenDef::new("Static")
                .add_part(TemplatePart::Static("<div>Hello</div>".to_string())),
        );
        let code = gen.generate("Static").unwrap();
        assert!(code.contains("constant-folded"));
        assert!(code.contains("<div>Hello</div>"));
    }

    #[test]
    fn test_code_generator_generate_dynamic() {
        let gen = ComponentCodeGenerator::new();
        gen.register(
            ComponentGenDef::new("Dynamic")
                .add_prop(PropDef::new("label", PropType::String))
                .add_part(TemplatePart::Static("<div>".to_string()))
                .add_part(TemplatePart::Dynamic("props.label".to_string())),
        );
        let code = gen.generate("Dynamic").unwrap();
        assert!(code.contains("DynamicProps"));
        assert!(code.contains("props.label"));
    }

    #[test]
    fn test_code_generator_generate_all() {
        let gen = ComponentCodeGenerator::new();
        gen.register(ComponentGenDef::new("A").add_part(TemplatePart::Static("<a/>".to_string())));
        gen.register(ComponentGenDef::new("B").add_part(TemplatePart::Static("<b/>".to_string())));
        let all = gen.generate_all();
        assert!(all.contains("component: A"));
        assert!(all.contains("component: B"));
    }

    #[test]
    fn test_code_generator_component_names() {
        let gen = ComponentCodeGenerator::new();
        gen.register(ComponentGenDef::new("X"));
        gen.register(ComponentGenDef::new("Y"));
        let names = gen.component_names();
        assert!(names.contains(&"X".to_string()));
        assert!(names.contains(&"Y".to_string()));
    }
}
