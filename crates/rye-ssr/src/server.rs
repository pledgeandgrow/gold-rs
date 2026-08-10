//! Server & full-stack modules — Goals 123, 125, 127–130, 186–195.
//!
//! - **Goal 123**: Edge rendering support
//! - **Goal 125**: Progressive / partial hydration
//! - **Goal 127**: Server-side data prefetching
//! - **Goal 128**: WebSocket / SSE integration
//! - **Goal 129**: Server middleware pipeline
//! - **Goal 130**: Static site generation (SSG)
//! - **Goal 186**: Route loaders (data loading patterns)
//! - **Goal 187**: API routes with OpenAPI generation
//! - **Goal 188**: Typed SSE channels
//! - **Goal 189**: Distributed SSR with session affinity
//! - **Goal 190**: Partial SSR re-rendering
//! - **Goal 191**: Server-side signal hydration
//! - **Goal 192**: Request-scoped context
//! - **Goal 193**: SSR compression with Brotli/Zstd
//! - **Goal 195**: Cron / scheduled tasks

pub mod api_routes;
pub mod compression;
pub mod cron;
pub mod edge;
pub mod loader;
pub mod middleware;
pub mod partial_rerender;
pub mod prefetch;
pub mod progressive_hydration;
pub mod realtime;
pub mod request_context;
pub mod session_affinity;
pub mod signal_hydration;
pub mod ssg;
pub mod typed_sse;
