//! Advanced rendering modules — Goals 111–120.
//!
//! - **Goal 111**: WebGPU canvas integration
//! - **Goal 112**: Virtual scrolling
//! - **Goal 113**: Intersection observer abstraction
//! - **Goal 114**: Resize observer abstraction
//! - **Goal 115**: Print/media query rendering
//! - **Goal 116**: View Transitions API
//! - **Goal 117**: Container queries
//! - **Goal 118**: Web Animations API bridge
//! - **Goal 119**: High-DPI / Retina rendering
//! - **Goal 120**: Multi-window support (web)

pub mod webgpu;
pub mod virtual_scroll;
pub mod observers;
pub mod media_query;
pub mod view_transitions;
pub mod container_queries;
pub mod web_animations;
pub mod dpi;
pub mod multi_window;
