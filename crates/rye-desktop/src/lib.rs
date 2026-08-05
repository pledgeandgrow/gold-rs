//! # rye-desktop
//!
//! Native desktop renderer for rye — wgpu + winit + taffy, no WebView.
//!
//! Implements a real GPU rendering pipeline:
//! - **wgpu**: Surface, device, queue, render pipeline with WGSL shaders
//! - **taffy**: Flexbox layout engine for the render tree
//! - **cosmic-text**: Font system, text shaping, and glyph rasterization
//! - **winit**: Window creation and event loop

pub mod native_renderer;
pub mod render_tree;
pub mod glyph_atlas;
pub mod gpu;
pub mod window;
pub mod input;

pub use native_renderer::NativeRenderer;
pub use render_tree::{RenderElement, RenderText, RenderNode};
pub use window::run;
