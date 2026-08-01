//! CSS Houdini Paint API bridge — custom CSS paint effects.
//!
//! `use_paint_worklet()` hook for custom CSS paint effects via Houdini.
//! On native, falls back to wgpu shaders. Enables custom backgrounds,
//! borders, and effects that CSS can't express.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A paint worklet — custom rendering logic for CSS paint effects.
pub struct PaintWorklet {
    /// Unique name for this worklet (used in CSS: `paint(my-worklet)`).
    pub name: String,
    /// Input properties the worklet reads (CSS custom properties).
    pub input_properties: Vec<String>,
    /// The paint callback that generates pixels.
    painter: Box<dyn Fn(&PaintContext) -> PaintOutput>,
}

/// Context provided to the paint callback.
#[derive(Debug, Clone)]
pub struct PaintContext {
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
    /// Input property values (CSS custom properties).
    pub properties: HashMap<String, String>,
    /// Device pixel ratio.
    pub pixel_ratio: f64,
}

impl PaintContext {
    /// Create a new paint context.
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            properties: HashMap::new(),
            pixel_ratio: 1.0,
        }
    }

    /// Set a property value.
    pub fn set_property(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), value.to_string());
    }

    /// Get a property value.
    pub fn get_property(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(|s| s.as_str())
    }
}

/// The output of a paint callback.
#[derive(Debug, Clone)]
pub struct PaintOutput {
    /// Raw RGBA pixel data.
    pub pixels: Vec<u8>,
    /// Width of the output.
    pub width: u32,
    /// Height of the output.
    pub height: u32,
}

impl PaintOutput {
    /// Create a solid-color paint output.
    pub fn solid_color(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Self {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for i in (0..pixels.len()).step_by(4) {
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
            pixels[i + 3] = a;
        }
        Self { pixels, width, height }
    }

    /// Create a gradient paint output (top-to-bottom).
    pub fn gradient(width: u32, height: u32, from: [u8; 4], to: [u8; 4]) -> Self {
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let denom = height.max(1) as f64 - 1.0;
        for y in 0..height {
            let t = if denom > 0.0 { y as f64 / denom } else { 0.0 };
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = (from[0] as f64 + (to[0] as f64 - from[0] as f64) * t) as u8;
                pixels[idx + 1] = (from[1] as f64 + (to[1] as f64 - from[1] as f64) * t) as u8;
                pixels[idx + 2] = (from[2] as f64 + (to[2] as f64 - from[2] as f64) * t) as u8;
                pixels[idx + 3] = (from[3] as f64 + (to[3] as f64 - from[3] as f64) * t) as u8;
            }
        }
        Self { pixels, width, height }
    }
}

impl PaintWorklet {
    /// Create a new paint worklet.
    pub fn new<F: Fn(&PaintContext) -> PaintOutput + 'static>(
        name: &str,
        input_properties: Vec<String>,
        painter: F,
    ) -> Self {
        Self {
            name: name.to_string(),
            input_properties,
            painter: Box::new(painter),
        }
    }

    /// Paint with the given context.
    pub fn paint(&self, ctx: &PaintContext) -> PaintOutput {
        (self.painter)(ctx)
    }

    /// Get the CSS registration script for this worklet.
    pub fn registration_script(&self) -> String {
        let _props: String = self
            .input_properties
            .iter()
            .map(|p| format!("'{}'", p))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"CSS.paintWorklet.addModule('/paint-worklets/{name}.js').then(function(){{CSS.registerProperty({{name:'--{name}-color',syntax:'<color>',inherits:false,initialValue:'#000'}});}});"#,
            name = self.name,
        )
    }

    /// Get the CSS usage string: `paint(name)`.
    pub fn css_paint(&self) -> String {
        format!("paint({})", self.name)
    }
}

/// Registry of paint worklets.
pub struct PaintWorkletRegistry {
    worklets: RefCell<HashMap<String, Rc<PaintWorklet>>>,
}

impl PaintWorkletRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            worklets: RefCell::new(HashMap::new()),
        }
    }

    /// Register a paint worklet.
    pub fn register(&self, worklet: PaintWorklet) {
        self.worklets
            .borrow_mut()
            .insert(worklet.name.clone(), Rc::new(worklet));
    }

    /// Get a worklet by name.
    pub fn get(&self, name: &str) -> Option<Rc<PaintWorklet>> {
        self.worklets.borrow().get(name).cloned()
    }

    /// Get all registered worklet names.
    pub fn names(&self) -> Vec<String> {
        self.worklets.borrow().keys().cloned().collect()
    }

    /// Paint with a named worklet.
    pub fn paint(&self, name: &str, ctx: &PaintContext) -> Option<PaintOutput> {
        self.get(name).map(|w| w.paint(ctx))
    }

    /// Get the count of registered worklets.
    pub fn len(&self) -> usize {
        self.worklets.borrow().len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.worklets.borrow().is_empty()
    }

    /// Clear all worklets.
    pub fn clear(&self) {
        self.worklets.borrow_mut().clear();
    }
}

impl Default for PaintWorkletRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register a paint worklet on the global registry.
pub fn use_paint_worklet(worklet: PaintWorklet) {
    GLOBAL_REGISTRY.with(|r| {
        let mut reg = r.borrow_mut();
        if reg.is_none() {
            *reg = Some(Rc::new(PaintWorkletRegistry::new()));
        }
        reg.as_ref().unwrap().register(worklet);
    });
}

// Global paint worklet registry.
thread_local! {
    static GLOBAL_REGISTRY: RefCell<Option<Rc<PaintWorkletRegistry>>> = const { RefCell::new(None) };
}

/// Get the global paint worklet registry.
pub fn global_paint_registry() -> Option<Rc<PaintWorkletRegistry>> {
    GLOBAL_REGISTRY.with(|r| r.borrow().clone())
}

/// Initialize the global paint worklet registry.
pub fn init_global_paint_registry() {
    GLOBAL_REGISTRY.with(|r| {
        *r.borrow_mut() = Some(Rc::new(PaintWorkletRegistry::new()));
    });
}

/// A wgpu shader fallback for native platforms (where Houdini isn't available).
#[derive(Debug, Clone)]
pub struct WgpuShaderFallback {
    /// The shader source (WGSL).
    pub source: String,
    /// The entry point function name.
    pub entry_point: String,
    /// Bind group layout descriptions.
    pub bindings: Vec<BindingDescription>,
}

/// A binding description for a wgpu shader.
#[derive(Debug, Clone)]
pub struct BindingDescription {
    /// Binding index.
    pub binding: u32,
    /// The type of binding.
    pub kind: BindingKind,
    /// Whether this binding is visible in the fragment shader.
    pub fragment_visible: bool,
}

/// The kind of wgpu binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// Uniform buffer.
    Uniform,
    /// Sampled texture.
    Texture,
    /// Sampler.
    Sampler,
    /// Storage buffer.
    Storage,
}

impl WgpuShaderFallback {
    /// Create a new wgpu shader fallback.
    pub fn new(source: &str, entry_point: &str) -> Self {
        Self {
            source: source.to_string(),
            entry_point: entry_point.to_string(),
            bindings: Vec::new(),
        }
    }

    /// Add a binding.
    pub fn with_binding(mut self, binding: u32, kind: BindingKind, fragment_visible: bool) -> Self {
        self.bindings.push(BindingDescription {
            binding,
            kind,
            fragment_visible,
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paint_output_solid_color() {
        let output = PaintOutput::solid_color(2, 2, 255, 0, 0, 255);
        assert_eq!(output.width, 2);
        assert_eq!(output.height, 2);
        assert_eq!(output.pixels.len(), 16);
        assert_eq!(&output.pixels[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn test_paint_output_gradient() {
        let output = PaintOutput::gradient(2, 4, [0, 0, 0, 255], [255, 255, 255, 255]);
        assert_eq!(output.pixels.len(), 32);
        // Top should be black
        assert_eq!(&output.pixels[0..4], &[0, 0, 0, 255]);
        // Bottom should be white
        assert_eq!(&output.pixels[28..32], &[255, 255, 255, 255]);
    }

    #[test]
    fn test_paint_worklet_basic() {
        let worklet = PaintWorklet::new("my-paint", vec!["--color".to_string()], |ctx| {
            let _color = ctx.get_property("--color").unwrap_or("#000");
            PaintOutput::solid_color(ctx.width as u32, ctx.height as u32, 255, 0, 0, 255)
        });

        let mut ctx = PaintContext::new(4.0, 4.0);
        ctx.set_property("--color", "#ff0000");
        let output = worklet.paint(&ctx);
        assert_eq!(output.width, 4);
    }

    #[test]
    fn test_paint_worklet_css_paint() {
        let worklet = PaintWorklet::new("ripple", vec![], |_| {
            PaintOutput::solid_color(1, 1, 0, 0, 0, 0)
        });
        assert_eq!(worklet.css_paint(), "paint(ripple)");
    }

    #[test]
    fn test_paint_worklet_registration_script() {
        let worklet = PaintWorklet::new("ripple", vec!["--ripple-color".to_string()], |_| {
            PaintOutput::solid_color(1, 1, 0, 0, 0, 0)
        });
        let script = worklet.registration_script();
        assert!(script.contains("paintWorklet"));
        assert!(script.contains("ripple"));
    }

    #[test]
    fn test_paint_worklet_registry() {
        let registry = PaintWorkletRegistry::new();
        registry.register(PaintWorklet::new("test", vec![], |_| {
            PaintOutput::solid_color(1, 1, 0, 0, 0, 255)
        }));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_paint_worklet_registry_paint() {
        let registry = PaintWorkletRegistry::new();
        registry.register(PaintWorklet::new("red", vec![], |ctx| {
            PaintOutput::solid_color(ctx.width as u32, ctx.height as u32, 255, 0, 0, 255)
        }));
        let ctx = PaintContext::new(2.0, 2.0);
        let output = registry.paint("red", &ctx).unwrap();
        assert_eq!(&output.pixels[0..4], &[255, 0, 0, 255]);
    }

    #[test]
    fn test_paint_context_properties() {
        let mut ctx = PaintContext::new(10.0, 10.0);
        ctx.set_property("--color", "red");
        assert_eq!(ctx.get_property("--color"), Some("red"));
        assert_eq!(ctx.get_property("--nonexistent"), None);
    }

    #[test]
    fn test_paint_worklet_registry_names() {
        let registry = PaintWorkletRegistry::new();
        registry.register(PaintWorklet::new("a", vec![], |_| PaintOutput::solid_color(1, 1, 0, 0, 0, 0)));
        registry.register(PaintWorklet::new("b", vec![], |_| PaintOutput::solid_color(1, 1, 0, 0, 0, 0)));
        let names = registry.names();
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn test_paint_worklet_registry_clear() {
        let registry = PaintWorkletRegistry::new();
        registry.register(PaintWorklet::new("a", vec![], |_| PaintOutput::solid_color(1, 1, 0, 0, 0, 0)));
        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_wgpu_shader_fallback() {
        let shader = WgpuShaderFallback::new("@fragment fn fs() -> vec4f { return vec4f(1.0); }", "fs")
            .with_binding(0, BindingKind::Uniform, true)
            .with_binding(1, BindingKind::Texture, true);
        assert_eq!(shader.bindings.len(), 2);
        assert_eq!(shader.bindings[0].binding, 0);
        assert_eq!(shader.bindings[0].kind, BindingKind::Uniform);
    }

    #[test]
    fn test_use_paint_worklet_global() {
        init_global_paint_registry();
        use_paint_worklet(PaintWorklet::new("global-test", vec![], |_| {
            PaintOutput::solid_color(1, 1, 0, 255, 0, 255)
        }));
        let registry = global_paint_registry().unwrap();
        assert!(registry.get("global-test").is_some());
    }
}
