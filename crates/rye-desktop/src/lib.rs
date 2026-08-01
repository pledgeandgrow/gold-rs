//! # rye-desktop
//!
//! Native desktop renderer for rye — wgpu + winit + taffy, no WebView.

pub mod native_renderer;
pub mod window;
pub mod input;

pub use native_renderer::NativeRenderer;
pub use window::Window;
