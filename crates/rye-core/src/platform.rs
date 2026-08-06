//! Platform abstraction — trait for cross-platform system APIs.
//!
//! The `Platform` trait abstracts over platform-specific capabilities like
//! filesystem access, notifications, clipboard, and networking. Each platform
//! (web, desktop, mobile) provides its own implementation.
//!
//! App code should depend on `Platform`, not on `web_sys` or `std::fs` directly.
//! This keeps your app logic portable across all renderers.
//!
//! # Example
//!
//! ```ignore
//! use rye_core::Platform;
//!
//! fn save_file(platform: &dyn Platform, name: &str, contents: &str) {
//!     platform.write_file(name, contents);
//! }
//! ```
//!
//! ## Available implementations
//!
//! - **Web/WebView**: `WebPlatform` (in `rye-html`) — uses browser APIs
//! - **Desktop**: `NativePlatform` (in `rye-desktop`) — uses OS APIs
//! - **Mobile**: `MobilePlatform` (in `rye-mobile`) — uses platform bridges

use std::fmt;

/// Result type for platform operations.
pub type PlatformResult<T> = Result<T, PlatformError>;

/// Error type for platform operations.
#[derive(Debug, Clone)]
pub enum PlatformError {
    /// Operation not supported on this platform.
    Unsupported,
    /// Permission denied.
    PermissionDenied,
    /// Resource not found.
    NotFound,
    /// I/O error with message.
    Io(String),
    /// Platform-specific error.
    Other(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(f, "operation not supported on this platform"),
            Self::PermissionDenied => write!(f, "permission denied"),
            Self::NotFound => write!(f, "resource not found"),
            Self::Io(msg) => write!(f, "I/O error: {}", msg),
            Self::Other(msg) => write!(f, "platform error: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {}

/// The rendering backend selected for this build.
///
/// Determined at compile time via feature flags:
/// - `webview` (default): DOM-based rendering via WebView (web, mobile, desktop WebView)
/// - `native`: GPU rendering via wgpu (desktop, mobile native)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    /// WebView-based rendering (WASM + DOM).
    WebView,
    /// Native GPU rendering (wgpu + taffy).
    Native,
}

/// Platform abstraction trait.
///
/// Provides access to system capabilities that differ between platforms.
/// App code should use this trait instead of platform-specific APIs.
///
/// Each backend provides its own implementation:
/// - `rye_html::WebPlatform` for WASM/WebView
/// - `rye_desktop::NativePlatform` for desktop
/// - `rye_mobile::MobilePlatform` for iOS/Android
pub trait Platform: 'static {
    /// Which rendering backend this platform uses.
    fn backend(&self) -> RenderBackend;

    // ── Filesystem ───────────────────────────────────────────

    /// Read a file as a UTF-8 string.
    fn read_file(&self, path: &str) -> PlatformResult<String>;

    /// Write a UTF-8 string to a file.
    fn write_file(&self, path: &str, contents: &str) -> PlatformResult<()>;

    /// Check if a file exists.
    fn file_exists(&self, path: &str) -> bool;

    // ── Clipboard ────────────────────────────────────────────

    /// Read text from the clipboard.
    fn clipboard_read(&self) -> PlatformResult<String>;

    /// Write text to the clipboard.
    fn clipboard_write(&self, text: &str) -> PlatformResult<()>;

    // ── Notifications ─────────────────────────────────────────

    /// Show a notification.
    fn notify(&self, title: &str, body: &str) -> PlatformResult<()>;

    // ── Networking ────────────────────────────────────────────

    /// Make an HTTP GET request and return the response body.
    fn http_get(&self, url: &str) -> PlatformResult<String>;

    /// Make an HTTP POST request with a body and return the response body.
    fn http_post(&self, url: &str, body: &str) -> PlatformResult<String>;

    // ── System info ───────────────────────────────────────────

    /// Get the platform name (e.g. "web", "windows", "macos", "android", "ios").
    fn platform_name(&self) -> &str;

    /// Get the screen dimensions in logical pixels.
    fn screen_size(&self) -> (f64, f64);
}

/// A no-op platform implementation for testing or headless environments.
pub struct NoopPlatform {
    backend: RenderBackend,
}

impl NoopPlatform {
    /// Create a no-op platform with the given backend.
    pub fn new(backend: RenderBackend) -> Self {
        Self { backend }
    }
}

impl Platform for NoopPlatform {
    fn backend(&self) -> RenderBackend {
        self.backend
    }

    fn read_file(&self, _path: &str) -> PlatformResult<String> {
        Err(PlatformError::Unsupported)
    }

    fn write_file(&self, _path: &str, _contents: &str) -> PlatformResult<()> {
        Err(PlatformError::Unsupported)
    }

    fn file_exists(&self, _path: &str) -> bool {
        false
    }

    fn clipboard_read(&self) -> PlatformResult<String> {
        Err(PlatformError::Unsupported)
    }

    fn clipboard_write(&self, _text: &str) -> PlatformResult<()> {
        Err(PlatformError::Unsupported)
    }

    fn notify(&self, _title: &str, _body: &str) -> PlatformResult<()> {
        Err(PlatformError::Unsupported)
    }

    fn http_get(&self, _url: &str) -> PlatformResult<String> {
        Err(PlatformError::Unsupported)
    }

    fn http_post(&self, _url: &str, _body: &str) -> PlatformResult<String> {
        Err(PlatformError::Unsupported)
    }

    fn platform_name(&self) -> &str {
        "noop"
    }

    fn screen_size(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_platform() {
        let p = NoopPlatform::new(RenderBackend::WebView);
        assert_eq!(p.backend(), RenderBackend::WebView);
        assert_eq!(p.platform_name(), "noop");
        assert_eq!(p.screen_size(), (0.0, 0.0));
        assert!(!p.file_exists("foo"));
        assert!(p.read_file("foo").is_err());
        assert!(p.write_file("foo", "bar").is_err());
        assert!(p.clipboard_read().is_err());
        assert!(p.clipboard_write("test").is_err());
        assert!(p.notify("title", "body").is_err());
        assert!(p.http_get("http://example.com").is_err());
        assert!(p.http_post("http://example.com", "body").is_err());
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            PlatformError::Unsupported.to_string(),
            "operation not supported on this platform"
        );
        assert_eq!(
            PlatformError::PermissionDenied.to_string(),
            "permission denied"
        );
        assert_eq!(PlatformError::NotFound.to_string(), "resource not found");
        assert_eq!(
            PlatformError::Io("disk full".to_string()).to_string(),
            "I/O error: disk full"
        );
        assert_eq!(
            PlatformError::Other("custom".to_string()).to_string(),
            "platform error: custom"
        );
    }

    #[test]
    fn test_backend_equality() {
        assert_eq!(RenderBackend::WebView, RenderBackend::WebView);
        assert_ne!(RenderBackend::WebView, RenderBackend::Native);
    }
}
