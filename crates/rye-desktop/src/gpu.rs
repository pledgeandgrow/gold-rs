//! GPU context — wgpu initialization, render pipeline, and frame rendering.
//!
//! Creates a wgpu surface from a raw window handle, initializes the device
//! and queue, builds a render pipeline with WGSL shaders, and provides a
//! `render` method that draws instanced quads (rectangles + text glyphs).

use bytemuck::cast_slice;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::glyph_atlas::GlyphAtlas;

/// Combined trait for types that have both window and display handles.
pub trait WindowHandle: HasWindowHandle + HasDisplayHandle {}
impl<T: HasWindowHandle + HasDisplayHandle> WindowHandle for T {}

/// WGSL shader source for the render pipeline.
/// Single pipeline draws both solid rectangles and text glyphs.
/// Alpha is sampled from an R8Unorm atlas texture.
const SHADER_SRC: &str = r#"
struct Uniforms {
    surface_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var glyph_texture: texture_2d<f32>;
@group(0) @binding(2) var glyph_sampler: sampler;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,
    @location(1) rect_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) uv_pos: vec2<f32>,
    @location(4) uv_size: vec2<f32>,
    @location(5) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pixel_pos = input.rect_pos + input.quad_pos * input.rect_size;
    let clip_x = (pixel_pos.x / uniforms.surface_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (pixel_pos.y / uniforms.surface_size.y) * 2.0;
    output.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.uv = input.uv_pos + input.quad_pos * input.uv_size;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(glyph_texture, glyph_sampler, input.uv).r;
    return vec4<f32>(input.color.rgb, alpha * input.color.a);
}
"#;

/// Unit quad vertices (two triangles forming a rectangle).
/// 6 vertices × 2 floats = 12 floats.
const QUAD_VERTICES: [f32; 12] = [
    0.0, 0.0, // top-left
    1.0, 0.0, // top-right
    1.0, 1.0, // bottom-right
    0.0, 0.0, // top-left (second triangle)
    1.0, 1.0, // bottom-right
    0.0, 1.0, // bottom-left
];

/// Maximum number of draw instances per frame.
const MAX_INSTANCES: usize = 8192;

/// Per-instance data: 12 floats = 48 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Pixel position (top-left corner).
    pub x: f32,
    pub y: f32,
    /// Pixel size (width, height).
    pub w: f32,
    pub h: f32,
    /// UV offset in atlas (0.0–1.0).
    pub u: f32,
    pub v: f32,
    /// UV size in atlas (0.0–1.0).
    pub uw: f32,
    pub vh: f32,
    /// Color (RGBA, 0.0–1.0).
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// GPU context — holds all wgpu resources.
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pub glyph_atlas: GlyphAtlas,
    pub width: u32,
    pub height: u32,
}

impl GpuContext {
    /// Initialize the GPU context from a window handle.
    pub fn new(window: &dyn WindowHandle) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface from raw handles (avoids lifetime issues).
        let raw_display = window.display_handle().unwrap().as_raw();
        let raw_window = window.window_handle().unwrap().as_raw();
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: raw_display,
                    raw_window_handle: raw_window,
                })
                .expect("Failed to create wgpu surface")
        };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find a suitable GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("rye GPU device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
        }, None))
        .expect("Failed to create GPU device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let (width, height) = (800, 600);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        // Create glyph atlas.
        let glyph_atlas = GlyphAtlas::new(&device, &queue, 1024);

        // Shader module.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rye render shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        // Uniform buffer (surface dimensions).
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rye uniform buffer"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Vertex buffer (unit quad).
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rye vertex buffer"),
            size: QUAD_VERTICES.len() as u64 * 4,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buffer, 0, cast_slice(&QUAD_VERTICES));

        // Instance buffer (dynamic).
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rye instance buffer"),
            size: (MAX_INSTANCES * std::mem::size_of::<InstanceData>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rye bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(8).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Bind group.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rye bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(glyph_atlas.texture_view()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(glyph_atlas.sampler()),
                },
            ],
        });

        // Pipeline layout.
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rye pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline.
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rye render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 8,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            shader_location: 0,
                            offset: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<InstanceData>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { shader_location: 1, offset: 0,  format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { shader_location: 2, offset: 8,  format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { shader_location: 3, offset: 16, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { shader_location: 4, offset: 24, format: wgpu::VertexFormat::Float32x2 },
                            wgpu::VertexAttribute { shader_location: 5, offset: 32, format: wgpu::VertexFormat::Float32x4 },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            uniform_buffer,
            vertex_buffer,
            instance_buffer,
            bind_group,
            glyph_atlas,
            width,
            height,
        }
    }

    /// Resize the surface to match the window.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Render a frame with the given instances.
    pub fn render(&mut self, instances: &[InstanceData]) {
        // Update uniform buffer (surface dimensions).
        let uniforms: [f32; 2] = [self.width as f32, self.height as f32];
        self.queue
            .write_buffer(&self.uniform_buffer, 0, cast_slice(&uniforms));

        // Update instance buffer.
        let count = instances.len().min(MAX_INSTANCES);
        if count > 0 {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                cast_slice(&instances[..count]),
            );
        }

        // Get the next frame texture.
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return;
            }
            Err(_) => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rye render encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rye render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.draw(0..6, 0..count as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
