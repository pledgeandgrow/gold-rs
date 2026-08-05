//! Glyph atlas — caches rasterized glyphs in a wgpu texture.
//!
//! The atlas uses an R8Unorm texture (single-channel alpha). Each glyph is
//! rasterized once and stored at a specific position in the atlas. The
//! shader samples the alpha value and multiplies by the instance color.
//!
//! A 1×1 white pixel is stored at (0, 0) for rendering solid rectangles.

use std::collections::HashMap;

use wgpu::{
    AddressMode, Extent3d, FilterMode, Origin3d, Sampler,
    SamplerDescriptor, TexelCopyTextureInfo, TexelCopyBufferLayout, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

/// A cached glyph entry — position and size in the atlas.
#[derive(Clone, Copy, Debug)]
pub struct GlyphEntry {
    /// X position in the atlas (pixels).
    pub x: u32,
    /// Y position in the atlas (pixels).
    pub y: u32,
    /// Width of the glyph (pixels).
    pub width: u32,
    /// Height of the glyph (pixels).
    pub height: u32,
}

/// The glyph atlas — a GPU texture that caches rasterized glyphs.
pub struct GlyphAtlas {
    texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
    atlas_size: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    entries: HashMap<cosmic_text::CacheKey, GlyphEntry>,
}

impl GlyphAtlas {
    /// Create a new glyph atlas with the given size (e.g. 1024×1024).
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("rye glyph atlas"),
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("rye glyph sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let atlas = Self {
            texture,
            texture_view,
            sampler,
            atlas_size: size,
            cursor_x: 1,
            cursor_y: 1,
            row_height: 1,
            entries: HashMap::new(),
        };

        // Initialize the first pixel (1×1) to white for solid rectangles.
        let white_pixel = [255u8];
        atlas.upload_glyph(0, 0, 1, 1, &white_pixel, queue);

        atlas
    }

    /// Get the texture view for binding.
    pub fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }

    /// Get the sampler for binding.
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Get a cached glyph entry, or `None` if not cached.
    pub fn get(&self, key: &cosmic_text::CacheKey) -> Option<GlyphEntry> {
        self.entries.get(key).copied()
    }

    /// Get the white pixel entry (for solid rectangles).
    pub fn white_pixel(&self) -> GlyphEntry {
        GlyphEntry {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }
    }

    /// Upload glyph pixel data to the atlas texture.
    fn upload_glyph(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        data: &[u8],
        queue: &wgpu::Queue,
    ) {
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d { x, y, z: 0 },
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Rasterize and cache a glyph. Returns the atlas entry on success.
    pub fn add_glyph(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut cosmic_text::FontSystem,
        swash_cache: &mut cosmic_text::SwashCache,
        key: cosmic_text::CacheKey,
    ) -> Option<GlyphEntry> {
        if let Some(entry) = self.entries.get(&key) {
            return Some(*entry);
        }

        let image = swash_cache.get_image(font_system, key);
        let image = match image {
            Some(img) => img,
            None => return None,
        };
        let width = image.placement.width;
        let height = image.placement.height;

        if width == 0 || height == 0 {
            return None;
        }

        // Check if we need a new row.
        if self.cursor_x + width > self.atlas_size {
            self.cursor_x = 1;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }

        if self.cursor_y + height > self.atlas_size {
            return None; // Atlas full.
        }

        let x = self.cursor_x;
        let y = self.cursor_y;

        // Convert to R8 alpha data.
        let alpha_data: Vec<u8> = match image.content {
            cosmic_text::SwashContent::Mask => image.data.clone(),
            cosmic_text::SwashContent::Color => {
                // Extract alpha channel from RGBA.
                (0..(width * height) as usize)
                    .map(|i| image.data[i * 4 + 3])
                    .collect()
            }
            cosmic_text::SwashContent::SubpixelMask => return None,
        };

        self.upload_glyph(x, y, width, height, &alpha_data, queue);

        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);

        let entry = GlyphEntry {
            x,
            y,
            width,
            height,
        };
        self.entries.insert(key, entry);
        Some(entry)
    }
}
