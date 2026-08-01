//! Performance optimization modules — Goals 104–109.
//!
//! This module groups several performance-focused features:
//!
//! - **Goal 104**: Streaming Wasm compilation (`instantiateStreaming` + preload hints)
//! - **Goal 105**: CSS-based reactive updates (`data_state!` macro, CSS attribute selectors)
//! - **Goal 106**: Wasm SIMD for layout math (SIMD-accelerated layout computations)
//! - **Goal 107**: Wasm threading via SharedArrayBuffer (Web Worker offloading)
//! - **Goal 108**: Memory profiling tools (allocation tracking, arena stats, leak detection)
//! - **Goal 109**: Bridge call counter (Wasm→JS bridge call tracking per frame)

pub mod streaming_wasm;
pub mod css_reactive;
pub mod simd;
pub mod threading;
pub mod memory_profiler;
pub mod bridge_counter;
pub mod wasm_gc;
pub mod gpu_pooling;
pub mod speculative_preload;
pub mod render_coalescing;
pub mod wasm_precompilation;
pub mod selective_aot;
