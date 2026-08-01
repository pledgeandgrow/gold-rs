//! Render-to-texture — render components to a GPU texture instead of the screen.
//!
//! On desktop/mobile, render components to a GPU texture. Enables picture-in-picture,
//! component thumbnails, drag previews, screenshot generation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A texture identifier — opaque handle to a GPU texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u64);

/// Configuration for rendering to a texture.
#[derive(Debug, Clone)]
pub struct TextureConfig {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format.
    pub format: TextureFormat,
    /// Whether to use alpha blending.
    pub alpha: bool,
    /// Multi-sample anti-aliasing samples (1 = disabled).
    pub msaa_samples: u32,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            format: TextureFormat::Rgba8,
            alpha: true,
            msaa_samples: 1,
        }
    }
}

/// Pixel format for textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    /// 8-bit RGBA.
    Rgba8,
    /// 8-bit BGRA (common on Windows/DirectX).
    Bgra8,
    /// 16-bit float RGBA (HDR).
    Rgba16Float,
    /// 10-bit RGB + 2-bit alpha.
    Rgb10A2,
}

impl TextureFormat {
    /// Get the bytes per pixel for this format.
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            TextureFormat::Rgba8 | TextureFormat::Bgra8 => 4,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgb10A2 => 4,
        }
    }

    /// Get the wgpu texture format name (for interop).
    pub fn wgpu_name(&self) -> &'static str {
        match self {
            TextureFormat::Rgba8 => "Rgba8Unorm",
            TextureFormat::Bgra8 => "Bgra8Unorm",
            TextureFormat::Rgba16Float => "Rgba16Float",
            TextureFormat::Rgb10A2 => "Rgb10a2Unorm",
        }
    }
}

/// A rendered texture — the result of rendering a component to a texture.
#[derive(Debug, Clone)]
pub struct RenderedTexture {
    /// The texture ID.
    pub id: TextureId,
    /// The config used to render this texture.
    pub config: TextureConfig,
    /// The raw pixel data.
    pub pixels: Vec<u8>,
    /// Whether this texture is still valid (not disposed).
    pub valid: bool,
}

impl RenderedTexture {
    /// Get the dimensions of the texture.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get the byte length of the pixel data.
    pub fn byte_len(&self) -> usize {
        self.pixels.len()
    }

    /// Get a pixel at (x, y) as RGBA.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.config.width || y >= self.config.height {
            return None;
        }
        let bpp = self.config.format.bytes_per_pixel();
        let idx = ((y * self.config.width + x) as usize) * bpp;
        if idx + 4 > self.pixels.len() {
            return None;
        }
        Some([self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2], self.pixels[idx + 3]])
    }
}

/// The texture renderer — manages texture creation, rendering, and disposal.
pub struct TextureRenderer {
    textures: RefCell<HashMap<TextureId, RenderedTexture>>,
    next_id: RefCell<u64>,
}

impl TextureRenderer {
    /// Create a new texture renderer.
    pub fn new() -> Self {
        Self {
            textures: RefCell::new(HashMap::new()),
            next_id: RefCell::new(0),
        }
    }

    /// Render a component to a texture.
    pub fn render_to_texture<F: FnOnce(&TextureConfig) -> Vec<u8>>(
        &self,
        config: TextureConfig,
        render_fn: F,
    ) -> TextureId {
        let id = {
            let mut next = self.next_id.borrow_mut();
            *next += 1;
            TextureId(*next)
        };

        let pixels = render_fn(&config);
        let texture = RenderedTexture {
            id,
            config: config.clone(),
            pixels,
            valid: true,
        };

        self.textures.borrow_mut().insert(id, texture);
        id
    }

    /// Get a texture by ID.
    pub fn get_texture(&self, id: TextureId) -> Option<RenderedTexture> {
        self.textures.borrow().get(&id).cloned()
    }

    /// Dispose a texture, freeing its resources.
    pub fn dispose(&self, id: TextureId) {
        if let Some(tex) = self.textures.borrow_mut().get_mut(&id) {
            tex.valid = false;
        }
        self.textures.borrow_mut().remove(&id);
    }

    /// Update a texture's content without recreating it.
    pub fn update_texture<F: FnOnce(&TextureConfig) -> Vec<u8>>(
        &self,
        id: TextureId,
        render_fn: F,
    ) -> bool {
        let mut textures = self.textures.borrow_mut();
        if let Some(tex) = textures.get_mut(&id) {
            tex.pixels = render_fn(&tex.config);
            return true;
        }
        false
    }

    /// Get the number of active textures.
    pub fn texture_count(&self) -> usize {
        self.textures.borrow().len()
    }

    /// Dispose all textures.
    pub fn dispose_all(&self) {
        self.textures.borrow_mut().clear();
    }

    /// Create a thumbnail of a texture at a smaller size.
    pub fn create_thumbnail(&self, id: TextureId, max_dim: u32) -> Option<TextureId> {
        let tex = self.get_texture(id)?;
        let scale = max_dim as f64 / tex.config.width.max(tex.config.height) as f64;
        let new_w = (tex.config.width as f64 * scale) as u32;
        let new_h = (tex.config.height as f64 * scale) as u32;

        let config = TextureConfig {
            width: new_w,
            height: new_h,
            format: tex.config.format,
            alpha: tex.config.alpha,
            msaa_samples: 1,
        };

        // Simple nearest-neighbor downscaling
        let bpp = config.format.bytes_per_pixel();
        let mut pixels = vec![0u8; (new_w * new_h) as usize * bpp];
        for y in 0..new_h {
            for x in 0..new_w {
                let src_x = (x as f64 / scale) as u32;
                let src_y = (y as f64 / scale) as u32;
                if let Some(px) = tex.pixel(src_x, src_y) {
                    let idx = ((y * new_w + x) as usize) * bpp;
                    if idx + 4 <= pixels.len() {
                        pixels[idx] = px[0];
                        pixels[idx + 1] = px[1];
                        pixels[idx + 2] = px[2];
                        pixels[idx + 3] = px[3];
                    }
                }
            }
        }

        Some(self.render_to_texture(config, |_| pixels))
    }
}

impl Default for TextureRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// A drag preview — a texture used for drag-and-drop visual feedback.
pub struct DragPreview {
    renderer: Rc<TextureRenderer>,
    texture_id: Option<TextureId>,
}

impl DragPreview {
    /// Create a new drag preview manager.
    pub fn new(renderer: Rc<TextureRenderer>) -> Self {
        Self {
            renderer,
            texture_id: None,
        }
    }

    /// Start a drag preview with the given render function.
    pub fn start<F: FnOnce(&TextureConfig) -> Vec<u8>>(&mut self, config: TextureConfig, render_fn: F) {
        if let Some(id) = self.texture_id.take() {
            self.renderer.dispose(id);
        }
        self.texture_id = Some(self.renderer.render_to_texture(config, render_fn));
    }

    /// Update the drag preview content.
    pub fn update<F: FnOnce(&TextureConfig) -> Vec<u8>>(&self, render_fn: F) -> bool {
        if let Some(id) = self.texture_id {
            return self.renderer.update_texture(id, render_fn);
        }
        false
    }

    /// End the drag preview, disposing the texture.
    pub fn end(&mut self) {
        if let Some(id) = self.texture_id.take() {
            self.renderer.dispose(id);
        }
    }

    /// Get the current texture ID.
    pub fn texture_id(&self) -> Option<TextureId> {
        self.texture_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_color_texture(config: &TextureConfig, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let bpp = config.format.bytes_per_pixel();
        let mut pixels = vec![0u8; (config.width * config.height) as usize * bpp];
        for i in (0..pixels.len()).step_by(bpp) {
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
            pixels[i + 3] = a;
        }
        pixels
    }

    #[test]
    fn test_texture_renderer_basic() {
        let renderer = TextureRenderer::new();
        let config = TextureConfig { width: 4, height: 4, ..Default::default() };
        let id = renderer.render_to_texture(config, |c| solid_color_texture(c, 255, 0, 0, 255));
        let tex = renderer.get_texture(id).unwrap();
        assert_eq!(tex.dimensions(), (4, 4));
        assert_eq!(tex.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_texture_dispose() {
        let renderer = TextureRenderer::new();
        let id = renderer.render_to_texture(TextureConfig::default(), |c| solid_color_texture(c, 0, 255, 0, 255));
        assert_eq!(renderer.texture_count(), 1);
        renderer.dispose(id);
        assert_eq!(renderer.texture_count(), 0);
        assert!(renderer.get_texture(id).is_none());
    }

    #[test]
    fn test_texture_update() {
        let renderer = TextureRenderer::new();
        let id = renderer.render_to_texture(TextureConfig::default(), |c| solid_color_texture(c, 0, 0, 0, 255));
        assert!(renderer.update_texture(id, |c| solid_color_texture(c, 255, 255, 255, 255)));
        let tex = renderer.get_texture(id).unwrap();
        assert_eq!(tex.pixel(0, 0), Some([255, 255, 255, 255]));
    }

    #[test]
    fn test_texture_format_bytes_per_pixel() {
        assert_eq!(TextureFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(TextureFormat::Rgba16Float.bytes_per_pixel(), 8);
        assert_eq!(TextureFormat::Rgb10A2.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_texture_format_wgpu_name() {
        assert_eq!(TextureFormat::Rgba8.wgpu_name(), "Rgba8Unorm");
        assert_eq!(TextureFormat::Bgra8.wgpu_name(), "Bgra8Unorm");
    }

    #[test]
    fn test_create_thumbnail() {
        let renderer = TextureRenderer::new();
        let config = TextureConfig { width: 100, height: 100, ..Default::default() };
        let id = renderer.render_to_texture(config, |c| solid_color_texture(c, 0, 0, 255, 255));
        let thumb_id = renderer.create_thumbnail(id, 50).unwrap();
        let thumb = renderer.get_texture(thumb_id).unwrap();
        assert_eq!(thumb.config.width, 50);
        assert_eq!(thumb.config.height, 50);
    }

    #[test]
    fn test_drag_preview_lifecycle() {
        let renderer = Rc::new(TextureRenderer::new());
        let mut preview = DragPreview::new(Rc::clone(&renderer));

        preview.start(TextureConfig { width: 64, height: 64, ..Default::default() }, |c| {
            solid_color_texture(c, 128, 128, 128, 255)
        });
        assert!(preview.texture_id().is_some());
        assert_eq!(renderer.texture_count(), 1);

        preview.end();
        assert!(preview.texture_id().is_none());
        assert_eq!(renderer.texture_count(), 0);
    }

    #[test]
    fn test_drag_preview_update() {
        let renderer = Rc::new(TextureRenderer::new());
        let mut preview = DragPreview::new(Rc::clone(&renderer));
        preview.start(TextureConfig { width: 8, height: 8, ..Default::default() }, |c| {
            solid_color_texture(c, 0, 0, 0, 255)
        });
        assert!(preview.update(|c| solid_color_texture(c, 255, 0, 0, 255)));
        let tex = renderer.get_texture(preview.texture_id().unwrap()).unwrap();
        assert_eq!(tex.pixel(0, 0), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_rendered_texture_pixel_out_of_bounds() {
        let renderer = TextureRenderer::new();
        let id = renderer.render_to_texture(TextureConfig { width: 2, height: 2, ..Default::default() }, |c| {
            solid_color_texture(c, 0, 0, 0, 255)
        });
        let tex = renderer.get_texture(id).unwrap();
        assert!(tex.pixel(5, 5).is_none());
    }

    #[test]
    fn test_dispose_all() {
        let renderer = TextureRenderer::new();
        renderer.render_to_texture(TextureConfig::default(), |c| solid_color_texture(c, 1, 2, 3, 4));
        renderer.render_to_texture(TextureConfig::default(), |c| solid_color_texture(c, 5, 6, 7, 8));
        assert_eq!(renderer.texture_count(), 2);
        renderer.dispose_all();
        assert_eq!(renderer.texture_count(), 0);
    }
}
