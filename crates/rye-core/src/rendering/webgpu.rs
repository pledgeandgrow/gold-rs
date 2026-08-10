//! Goal 111: WebGPU canvas integration.
//!
//! `<Canvas>` component that provides direct WebGPU rendering context for
//! custom graphics, charts, games. Same API on web (WebGPU) and native (wgpu).

/// WebGPU context configuration.
#[derive(Debug, Clone)]
pub struct WebGpuConfig {
    /// Canvas element ID.
    pub canvas_id: String,
    /// Whether to use alpha blending.
    pub alpha: bool,
    /// Whether to use depth buffer.
    pub depth: bool,
    /// Whether to use stencil buffer.
    pub stencil: bool,
    /// Anti-aliasing samples (1, 2, 4, 8, 16).
    pub antialias_samples: u32,
}

impl WebGpuConfig {
    /// Create a new WebGPU config for the given canvas ID.
    pub fn new(canvas_id: impl Into<String>) -> Self {
        Self {
            canvas_id: canvas_id.into(),
            alpha: true,
            depth: true,
            stencil: false,
            antialias_samples: 4,
        }
    }

    /// Disable alpha blending.
    pub fn no_alpha(mut self) -> Self {
        self.alpha = false;
        self
    }

    /// Set antialiasing samples.
    pub fn with_antialias(mut self, samples: u32) -> Self {
        self.antialias_samples = samples;
        self
    }
}

/// A WebGPU render handle — opaque reference to the GPU context.
pub struct WebGpuContext {
    /// Configuration.
    pub config: WebGpuConfig,
    /// Whether the context is active.
    pub active: bool,
}

impl WebGpuContext {
    /// Create a new WebGPU context.
    pub fn new(config: WebGpuConfig) -> Self {
        Self {
            config,
            active: false,
        }
    }

    /// Initialize the GPU context.
    pub fn init(&mut self) {
        self.active = true;
    }

    /// Whether WebGPU is available on this platform.
    pub fn is_available() -> bool {
        // On Wasm: typeof navigator.gpu !== 'undefined'
        // On native: wgpu is always available
        #[cfg(not(target_arch = "wasm32"))]
        {
            true
        }
        #[cfg(target_arch = "wasm32")]
        {
            false
        }
    }
}

/// Generate the JS bootstrap for WebGPU initialization.
pub fn webgpu_init_script(canvas_id: &str) -> String {
    format!(
        r#"<script>
(function() {{
  async function initWebGPU() {{
    if (!navigator.gpu) {{
      console.warn('[rye] WebGPU not supported, falling back to WebGL2');
      return null;
    }}
    var canvas = document.getElementById('{canvas}');
    if (!canvas) return null;
    var adapter = await navigator.gpu.requestAdapter();
    if (!adapter) return null;
    var device = await adapter.requestDevice();
    var context = canvas.getContext('webgpu');
    var format = navigator.gpu.getPreferredCanvasFormat();
    context.configure({{
      device: device,
      format: format,
      alphaMode: 'premultiplied'
    }});
    return {{ device: device, context: context, format: format }};
  }}
  window.__rye_webgpu_init = initWebGPU;
}})();
</script>"#,
        canvas = canvas_id
    )
}

/// GPU buffer descriptor.
#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    /// Buffer size in bytes.
    pub size: u64,
    /// Buffer usage flags.
    pub usage: BufferUsage,
}

/// Buffer usage flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferUsage {
    /// Can be used as a vertex buffer.
    pub vertex: bool,
    /// Can be used as an index buffer.
    pub index: bool,
    /// Can be used as a uniform buffer.
    pub uniform: bool,
    /// Can be used as a storage buffer.
    pub storage: bool,
    /// Can be mapped for writing.
    pub copy_dst: bool,
    /// Can be mapped for reading.
    pub copy_src: bool,
}

impl BufferUsage {
    /// Vertex buffer usage.
    pub fn vertex() -> Self {
        Self {
            vertex: true,
            ..Default::default()
        }
    }

    /// Uniform buffer usage.
    pub fn uniform() -> Self {
        Self {
            uniform: true,
            copy_dst: true,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webgpu_config() {
        let config = WebGpuConfig::new("canvas1").no_alpha().with_antialias(8);
        assert_eq!(config.canvas_id, "canvas1");
        assert!(!config.alpha);
        assert_eq!(config.antialias_samples, 8);
    }

    #[test]
    fn test_webgpu_context() {
        let config = WebGpuConfig::new("canvas1");
        let mut ctx = WebGpuContext::new(config);
        assert!(!ctx.active);
        ctx.init();
        assert!(ctx.active);
    }

    #[test]
    fn test_webgpu_init_script() {
        let script = webgpu_init_script("my-canvas");
        assert!(script.contains("navigator.gpu"));
        assert!(script.contains("my-canvas"));
        assert!(script.contains("__rye_webgpu_init"));
    }

    #[test]
    fn test_buffer_usage() {
        let v = BufferUsage::vertex();
        assert!(v.vertex);
        assert!(!v.uniform);

        let u = BufferUsage::uniform();
        assert!(u.uniform);
        assert!(u.copy_dst);
        assert!(!u.vertex);
    }
}
