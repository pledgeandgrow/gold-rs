//! Goals 246-250: Ecosystem & interop deep cuts.
//!
//! React component wrapping, Vue component wrapping, Tailwind 4.0 integration,
//! WebGPU compute shaders, and Figma plugin.

use std::collections::HashMap;

// === Goal 246: React component wrapping ===

/// A wrapped React component.
#[derive(Debug, Clone)]
pub struct ReactWrapper {
    /// The React component name.
    pub name: String,
    /// The module path (e.g. "./MyComponent").
    pub module: String,
    /// The prop mappings (rye prop name → React prop name).
    pub prop_mappings: HashMap<String, String>,
    /// The event mappings (rye event → React callback).
    pub event_mappings: HashMap<String, String>,
    /// Whether the component has children.
    pub has_children: bool,
}

impl ReactWrapper {
    /// Create a new wrapper.
    pub fn new(name: &str, module: &str) -> Self {
        Self {
            name: name.to_string(),
            module: module.to_string(),
            prop_mappings: HashMap::new(),
            event_mappings: HashMap::new(),
            has_children: true,
        }
    }

    /// Add a prop mapping.
    pub fn map_prop(mut self, rye_name: &str, react_name: &str) -> Self {
        self.prop_mappings.insert(rye_name.to_string(), react_name.to_string());
        self
    }

    /// Add an event mapping.
    pub fn map_event(mut self, rye_event: &str, react_callback: &str) -> Self {
        self.event_mappings.insert(rye_event.to_string(), react_callback.to_string());
        self
    }

    /// Generate the JavaScript bridge code.
    pub fn generate_bridge_js(&self) -> String {
        let mut js = String::new();
        js.push_str(&format!("import {} from '{}';\n", self.name, self.module));
        js.push_str(&format!(
            "export function {}_wrapper(props) {{\n",
            self.name.to_lowercase(),
        ));
        js.push_str("  const reactProps = {};\n");
        for (rye, react) in &self.prop_mappings {
            js.push_str(&format!("  if (props.{rye} !== undefined) reactProps.{react} = props.{rye};\n"));
        }
        for (rye, react) in &self.event_mappings {
            js.push_str(&format!("  if (props.{rye}) reactProps.{react} = props.{rye};\n"));
        }
        if self.has_children {
            js.push_str("  return React.createElement(");
            js.push_str(&self.name);
            js.push_str(", reactProps, props.children);\n");
        } else {
            js.push_str(&format!("  return React.createElement({}, reactProps);\n", self.name));
        }
        js.push_str("}\n");
        js
    }

    /// Generate the Rust wrapper macro.
    pub fn generate_rust_macro(&self) -> String {
        format!(
            "#[macro_export]\nmacro_rules! wrap_{name} {{\n    ($($prop:ident: $val:expr),*) => {{\n        rye_react_bridge(\"{module}\", \"{name}\", &[$($prop),*], &[$($val),*]);\n    }};\n}}",
            name = self.name.to_lowercase(),
            module = self.module,
        )
    }
}

// === Goal 247: Vue component wrapping ===

/// A wrapped Vue component.
#[derive(Debug, Clone)]
pub struct VueWrapper {
    /// The Vue component name.
    pub name: String,
    /// The SFC file path.
    pub sfc_path: String,
    /// The prop mappings.
    pub prop_mappings: HashMap<String, String>,
    /// The event mappings.
    pub event_mappings: HashMap<String, String>,
}

impl VueWrapper {
    /// Create a new wrapper.
    pub fn new(name: &str, sfc_path: &str) -> Self {
        Self {
            name: name.to_string(),
            sfc_path: sfc_path.to_string(),
            prop_mappings: HashMap::new(),
            event_mappings: HashMap::new(),
        }
    }

    /// Add a prop mapping.
    pub fn map_prop(mut self, rye_name: &str, vue_name: &str) -> Self {
        self.prop_mappings.insert(rye_name.to_string(), vue_name.to_string());
        self
    }

    /// Add an event mapping.
    pub fn map_event(mut self, rye_event: &str, vue_event: &str) -> Self {
        self.event_mappings.insert(rye_event.to_string(), vue_event.to_string());
        self
    }

    /// Generate the mount script.
    pub fn generate_mount_script(&self) -> String {
        let mut js = String::new();
        js.push_str(&format!("import {{ createApp }} from 'vue';\n"));
        js.push_str(&format!("import Component from '{}';\n", self.sfc_path));
        js.push_str(&format!(
            "export function mount_{name}(el, props) {{\n",
            name = self.name.to_lowercase(),
        ));
        js.push_str("  const app = createApp(Component, {\n");
        for (rye, vue) in &self.prop_mappings {
            js.push_str(&format!("    {}: props.{},\n", vue, rye));
        }
        js.push_str("  });\n");
        js.push_str("  app.mount(el);\n");
        js.push_str("  return app;\n");
        js.push_str("}\n");
        js
    }
}

// === Goal 248: Tailwind 4.0 engine integration ===

/// Tailwind 4.0 (Oxide engine) configuration.
#[derive(Debug, Clone)]
pub struct Tailwind4Config {
    /// Whether arbitrary values are enabled.
    pub arbitrary_values: bool,
    /// Whether container queries are enabled.
    pub container_queries: bool,
    /// Whether 3D transforms are enabled.
    pub transforms_3d: bool,
    /// The content paths to scan.
    pub content: Vec<String>,
    /// Custom theme extensions.
    pub theme_extensions: HashMap<String, String>,
}

impl Default for Tailwind4Config {
    fn default() -> Self {
        Self {
            arbitrary_values: true,
            container_queries: true,
            transforms_3d: true,
            content: vec!["./src/**/*.rs".to_string(), "./src/**/*.html".to_string()],
            theme_extensions: HashMap::new(),
        }
    }
}

impl Tailwind4Config {
    /// Generate the Tailwind config.
    pub fn generate_config(&self) -> String {
        let mut config = String::new();
        config.push_str("/** @type {import('tailwindcss').Config} */\n");
        config.push_str("export default {\n");
        config.push_str("  content: [\n");
        for path in &self.content {
            config.push_str(&format!("    '{}',\n", path));
        }
        config.push_str("  ],\n");
        config.push_str(&format!("  arbitraryValues: {},\n", self.arbitrary_values));
        config.push_str(&format!("  containerQueries: {},\n", self.container_queries));
        config.push_str(&format!("  transforms3d: {},\n", self.transforms_3d));
        if !self.theme_extensions.is_empty() {
            config.push_str("  theme: {\n    extend: {\n");
            for (key, val) in &self.theme_extensions {
                config.push_str(&format!("      {}: {},\n", key, val));
            }
            config.push_str("    }\n  },\n");
        }
        config.push_str("}\n");
        config
    }

    /// Process utility classes and generate CSS.
    pub fn process_utilities(&self, classes: &[String]) -> String {
        let mut css = String::new();
        for class in classes {
            if let Some(generated) = self.generate_utility(class) {
                css.push_str(&generated);
            }
        }
        css
    }

    /// Generate CSS for a single utility class.
    fn generate_utility(&self, class: &str) -> Option<String> {
        // Basic utility generation
        if class == "flex" {
            return Some(".flex { display: flex; }\n".to_string());
        }
        if class == "hidden" {
            return Some(".hidden { display: none; }\n".to_string());
        }
        if class.starts_with("text-") {
            let size = &class[5..];
            return Some(format!(".{} {{ font-size: {}; }}\n", class, size));
        }
        if class.starts_with("bg-") {
            let color = &class[3..];
            return Some(format!(".{} {{ background-color: {}; }}\n", class, color));
        }
        if class.starts_with("p-") {
            let size = &class[2..];
            return Some(format!(".{} {{ padding: {}; }}\n", class, size));
        }
        None
    }
}

// === Goal 249: WebGPU compute shaders ===

/// A compute shader definition.
#[derive(Debug, Clone)]
pub struct ComputeShader {
    /// The shader name.
    pub name: String,
    /// The WGSL shader source.
    pub source: String,
    /// The workgroup size (x, y, z).
    pub workgroup_size: (u32, u32, u32),
    /// The input bindings.
    pub inputs: Vec<ComputeBinding>,
    /// The output bindings.
    pub outputs: Vec<ComputeBinding>,
}

/// A compute shader binding.
#[derive(Debug, Clone)]
pub struct ComputeBinding {
    /// The binding name.
    pub name: String,
    /// The binding type.
    pub binding_type: ComputeBindingType,
    /// The size in bytes.
    pub size: u64,
}

/// The type of compute binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeBindingType {
    /// A storage buffer (read-write).
    StorageBuffer,
    /// A uniform buffer (read-only).
    UniformBuffer,
    /// A storage texture.
    StorageTexture,
    /// A sampled texture.
    SampledTexture,
}

impl ComputeBindingType {
    /// Get the WGSL access qualifier.
    pub fn wgsl_access(&self) -> &'static str {
        match self {
            ComputeBindingType::StorageBuffer => "read_write",
            ComputeBindingType::UniformBuffer => "read",
            ComputeBindingType::StorageTexture => "write",
            ComputeBindingType::SampledTexture => "read",
        }
    }
}

impl ComputeShader {
    /// Create a new compute shader.
    pub fn new(name: &str, source: &str) -> Self {
        Self {
            name: name.to_string(),
            source: source.to_string(),
            workgroup_size: (1, 1, 1),
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Set workgroup size.
    pub fn with_workgroup_size(mut self, x: u32, y: u32, z: u32) -> Self {
        self.workgroup_size = (x, y, z);
        self
    }

    /// Add an input binding.
    pub fn add_input(mut self, binding: ComputeBinding) -> Self {
        self.inputs.push(binding);
        self
    }

    /// Add an output binding.
    pub fn add_output(mut self, binding: ComputeBinding) -> Self {
        self.outputs.push(binding);
        self
    }

    /// Generate the complete WGSL shader.
    pub fn generate_wgsl(&self) -> String {
        let mut wgsl = String::new();
        let mut binding_idx = 0u32;

        for input in &self.inputs {
            wgsl.push_str(&format!(
                "@group(0) @binding({}) var<{}> {}: array<f32>;\n",
                binding_idx,
                input.binding_type.wgsl_access(),
                input.name,
            ));
            binding_idx += 1;
        }
        for output in &self.outputs {
            wgsl.push_str(&format!(
                "@group(0) @binding({}) var<{}> {}: array<f32>;\n",
                binding_idx,
                output.binding_type.wgsl_access(),
                output.name,
            ));
            binding_idx += 1;
        }

        wgsl.push_str(&format!(
            "@compute @workgroup_size({}, {}, {})\nfn {}(@builtin(global_invocation_id) gid: vec3<u32>) {{\n",
            self.workgroup_size.0,
            self.workgroup_size.1,
            self.workgroup_size.2,
            self.name,
        ));
        wgsl.push_str(&self.source);
        wgsl.push_str("\n}\n");
        wgsl
    }
}

// === Goal 250: Figma plugin ===

/// A Figma design export.
#[derive(Debug, Clone)]
pub struct FigmaExport {
    /// The component name.
    pub name: String,
    /// The layout structure.
    pub layout: FigmaNode,
    /// The styles.
    pub styles: HashMap<String, String>,
    /// The interactive states.
    pub states: Vec<FigmaState>,
}

/// A Figma node — a layer in the design.
#[derive(Debug, Clone)]
pub struct FigmaNode {
    /// The node name.
    pub name: String,
    /// The node type.
    pub node_type: FigmaNodeType,
    /// The x position.
    pub x: f64,
    /// The y position.
    pub y: f64,
    /// The width.
    pub width: f64,
    /// The height.
    pub height: f64,
    /// The children.
    pub children: Vec<FigmaNode>,
    /// The text content (if text node).
    pub text: Option<String>,
    /// The style properties.
    pub style: HashMap<String, String>,
}

/// The type of a Figma node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigmaNodeType {
    /// A frame.
    Frame,
    /// A text node.
    Text,
    /// A rectangle.
    Rectangle,
    /// An ellipse.
    Ellipse,
    /// A group.
    Group,
    /// A component.
    Component,
    /// An image.
    Image,
}

impl FigmaNode {
    /// Create a new node.
    pub fn new(name: &str, node_type: FigmaNodeType) -> Self {
        Self {
            name: name.to_string(),
            node_type,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            children: Vec::new(),
            text: None,
            style: HashMap::new(),
        }
    }

    /// Set position.
    pub fn at_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Set size.
    pub fn with_size(mut self, w: f64, h: f64) -> Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Set text.
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Add a style.
    pub fn add_style(mut self, key: &str, value: &str) -> Self {
        self.style.insert(key.to_string(), value.to_string());
        self
    }

    /// Add a child.
    pub fn add_child(mut self, child: FigmaNode) -> Self {
        self.children.push(child);
        self
    }

    /// Convert to rye component code.
    pub fn to_rye_code(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let mut code = String::new();

        match self.node_type {
            FigmaNodeType::Frame | FigmaNodeType::Component | FigmaNodeType::Group => {
                code.push_str(&format!("{}div {{\n", pad));
                if !self.style.is_empty() {
                    let style_str: Vec<String> = self.style.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                    code.push_str(&format!("{}  style: \"{}\",\n", pad, style_str.join("; ")));
                }
                for child in &self.children {
                    code.push_str(&child.to_rye_code(indent + 1));
                }
                code.push_str(&format!("{}}}\n", pad));
            }
            FigmaNodeType::Text => {
                code.push_str(&format!("{}span {{ \"{}\" }}\n", pad, self.text.as_deref().unwrap_or("")));
            }
            FigmaNodeType::Rectangle => {
                code.push_str(&format!("{}div {{ style: \"width:{}px; height:{}px;\" }}\n", pad, self.width, self.height));
            }
            FigmaNodeType::Ellipse => {
                code.push_str(&format!("{}div {{ style: \"border-radius:50%; width:{}px; height:{}px;\" }}\n", pad, self.width, self.height));
            }
            FigmaNodeType::Image => {
                code.push_str(&format!("{}img {{ src: \"{}\" }}\n", pad, self.style.get("src").unwrap_or(&"".to_string())));
            }
        }
        code
    }
}

/// A Figma interactive state.
#[derive(Debug, Clone)]
pub struct FigmaState {
    /// The state name (e.g. "hover", "active").
    pub name: String,
    /// The style overrides.
    pub style_overrides: HashMap<String, String>,
}

impl FigmaState {
    /// Create a new state.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            style_overrides: HashMap::new(),
        }
    }

    /// Add a style override.
    pub fn add_override(mut self, key: &str, value: &str) -> Self {
        self.style_overrides.insert(key.to_string(), value.to_string());
        self
    }
}

impl FigmaExport {
    /// Create a new export.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            layout: FigmaNode::new("root", FigmaNodeType::Frame),
            styles: HashMap::new(),
            states: Vec::new(),
        }
    }

    /// Generate the full rye component code.
    pub fn to_rye_component(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!("#[component]\nfn {}() {{\n", self.name));
        code.push_str(&self.layout.to_rye_code(1));
        code.push_str("}\n");
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // React wrapper tests
    #[test]
    fn test_react_wrapper_new() {
        let w = ReactWrapper::new("Button", "./Button");
        assert_eq!(w.name, "Button");
        assert!(w.has_children);
    }

    #[test]
    fn test_react_wrapper_mappings() {
        let w = ReactWrapper::new("Button", "./Button")
            .map_prop("label", "text")
            .map_event("click", "onClick");
        assert_eq!(w.prop_mappings.get("label"), Some(&"text".to_string()));
        assert_eq!(w.event_mappings.get("click"), Some(&"onClick".to_string()));
    }

    #[test]
    fn test_react_wrapper_generate_js() {
        let w = ReactWrapper::new("Button", "./Button")
            .map_prop("label", "text")
            .map_event("click", "onClick");
        let js = w.generate_bridge_js();
        assert!(js.contains("import Button"));
        assert!(js.contains("reactProps"));
    }

    #[test]
    fn test_react_wrapper_generate_rust_macro() {
        let w = ReactWrapper::new("Button", "./Button");
        let macro_code = w.generate_rust_macro();
        assert!(macro_code.contains("wrap_button"));
        assert!(macro_code.contains("rye_react_bridge"));
    }

    // Vue wrapper tests
    #[test]
    fn test_vue_wrapper_new() {
        let w = VueWrapper::new("Card", "./Card.vue");
        assert_eq!(w.name, "Card");
        assert_eq!(w.sfc_path, "./Card.vue");
    }

    #[test]
    fn test_vue_wrapper_mappings() {
        let w = VueWrapper::new("Card", "./Card.vue")
            .map_prop("title", "title")
            .map_event("close", "onClose");
        assert_eq!(w.prop_mappings.len(), 1);
        assert_eq!(w.event_mappings.len(), 1);
    }

    #[test]
    fn test_vue_wrapper_mount_script() {
        let w = VueWrapper::new("Card", "./Card.vue").map_prop("title", "title");
        let js = w.generate_mount_script();
        assert!(js.contains("createApp"));
        assert!(js.contains("mount"));
    }

    // Tailwind tests
    #[test]
    fn test_tailwind4_config_default() {
        let config = Tailwind4Config::default();
        assert!(config.arbitrary_values);
        assert!(config.container_queries);
        assert!(config.transforms_3d);
    }

    #[test]
    fn test_tailwind4_generate_config() {
        let config = Tailwind4Config::default();
        let cfg = config.generate_config();
        assert!(cfg.contains("content"));
        assert!(cfg.contains("arbitraryValues"));
    }

    #[test]
    fn test_tailwind4_process_utilities() {
        let config = Tailwind4Config::default();
        let css = config.process_utilities(&["flex".to_string(), "hidden".to_string()]);
        assert!(css.contains("display: flex"));
        assert!(css.contains("display: none"));
    }

    #[test]
    fn test_tailwind4_generate_utility_text() {
        let config = Tailwind4Config::default();
        let css = config.process_utilities(&["text-16px".to_string()]);
        assert!(css.contains("font-size: 16px"));
    }

    #[test]
    fn test_tailwind4_generate_utility_bg() {
        let config = Tailwind4Config::default();
        let css = config.process_utilities(&["bg-red".to_string()]);
        assert!(css.contains("background-color: red"));
    }

    // Compute shader tests
    #[test]
    fn test_compute_shader_new() {
        let shader = ComputeShader::new("process", "// shader code");
        assert_eq!(shader.name, "process");
        assert_eq!(shader.workgroup_size, (1, 1, 1));
    }

    #[test]
    fn test_compute_shader_builder() {
        let shader = ComputeShader::new("process", "// code")
            .with_workgroup_size(8, 8, 1)
            .add_input(ComputeBinding { name: "data".into(), binding_type: ComputeBindingType::StorageBuffer, size: 1024 })
            .add_output(ComputeBinding { name: "result".into(), binding_type: ComputeBindingType::StorageBuffer, size: 1024 });
        assert_eq!(shader.workgroup_size, (8, 8, 1));
        assert_eq!(shader.inputs.len(), 1);
        assert_eq!(shader.outputs.len(), 1);
    }

    #[test]
    fn test_compute_binding_type_wgsl_access() {
        assert_eq!(ComputeBindingType::StorageBuffer.wgsl_access(), "read_write");
        assert_eq!(ComputeBindingType::UniformBuffer.wgsl_access(), "read");
    }

    #[test]
    fn test_compute_shader_generate_wgsl() {
        let shader = ComputeShader::new("process", "  // shader body")
            .with_workgroup_size(8, 8, 1)
            .add_input(ComputeBinding { name: "input".into(), binding_type: ComputeBindingType::StorageBuffer, size: 1024 })
            .add_output(ComputeBinding { name: "output".into(), binding_type: ComputeBindingType::StorageBuffer, size: 1024 });
        let wgsl = shader.generate_wgsl();
        assert!(wgsl.contains("@compute"));
        assert!(wgsl.contains("@workgroup_size(8, 8, 1)"));
        assert!(wgsl.contains("input"));
        assert!(wgsl.contains("output"));
    }

    // Figma tests
    #[test]
    fn test_figma_node_new() {
        let node = FigmaNode::new("root", FigmaNodeType::Frame);
        assert_eq!(node.name, "root");
        assert_eq!(node.node_type, FigmaNodeType::Frame);
    }

    #[test]
    fn test_figma_node_builder() {
        let node = FigmaNode::new("btn", FigmaNodeType::Component)
            .at_position(10.0, 20.0)
            .with_size(100.0, 40.0)
            .with_text("Click")
            .add_style("color", "blue");
        assert_eq!(node.x, 10.0);
        assert_eq!(node.width, 100.0);
        assert_eq!(node.text, Some("Click".to_string()));
        assert_eq!(node.style.get("color"), Some(&"blue".to_string()));
    }

    #[test]
    fn test_figma_node_to_rye_code_frame() {
        let node = FigmaNode::new("root", FigmaNodeType::Frame)
            .add_child(FigmaNode::new("text", FigmaNodeType::Text).with_text("Hello"));
        let code = node.to_rye_code(0);
        assert!(code.contains("div"));
        assert!(code.contains("Hello"));
    }

    #[test]
    fn test_figma_node_to_rye_code_text() {
        let node = FigmaNode::new("label", FigmaNodeType::Text).with_text("World");
        let code = node.to_rye_code(0);
        assert!(code.contains("span"));
        assert!(code.contains("World"));
    }

    #[test]
    fn test_figma_node_to_rye_code_ellipse() {
        let node = FigmaNode::new("circle", FigmaNodeType::Ellipse).with_size(50.0, 50.0);
        let code = node.to_rye_code(0);
        assert!(code.contains("border-radius:50%"));
    }

    #[test]
    fn test_figma_state_new() {
        let state = FigmaState::new("hover").add_override("color", "red");
        assert_eq!(state.name, "hover");
        assert_eq!(state.style_overrides.get("color"), Some(&"red".to_string()));
    }

    #[test]
    fn test_figma_export_to_rye_component() {
        let mut export = FigmaExport::new("MyComponent");
        export.layout = FigmaNode::new("root", FigmaNodeType::Frame)
            .add_child(FigmaNode::new("text", FigmaNodeType::Text).with_text("Hello"));
        let code = export.to_rye_component();
        assert!(code.contains("#[component]"));
        assert!(code.contains("MyComponent"));
        assert!(code.contains("Hello"));
    }
}
