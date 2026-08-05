//! FFI bindings — JNI for Android, Obj-C for iOS.
//!
//! This module provides the C ABI boundary between Rust and the host mobile
//! platform. On Android, the `jni` submodule uses the JNI crate to communicate
//! with Java/Kotlin. On iOS, the `objc` submodule uses the objc crate to
//! communicate with Objective-C/Swift.
//!
//! The `bridge` submodule provides a renderer bridge that translates `Renderer`
//! trait calls into FFI calls, and the `types` submodule defines shared
//! C-compatible types.

pub mod types;
pub mod bridge;
pub mod bindings_gen;

#[cfg(target_os = "android")]
pub mod jni;

#[cfg(target_os = "ios")]
pub mod objc;

pub use types::*;

/// Initialize the FFI layer with a platform configuration.
///
/// This is called from native code (via `rye_ffi_init`) after the surface
/// is created. It sets up the renderer bridge and stores the callbacks.
pub fn init(config: PlatformConfig) -> Result<bridge::FfiRendererBridge, FfiResult> {
    if config.surface_handle.is_null() {
        return Err(FfiResult::NullPtr);
    }
    if config.width == 0 || config.height == 0 {
        return Err(FfiResult::InvalidArg);
    }
    Ok(bridge::FfiRendererBridge::new(config))
}
