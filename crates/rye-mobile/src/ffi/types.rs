//! Shared FFI types — used by both JNI (Android) and Obj-C (iOS) bridges.
//!
//! These types define the C ABI boundary between Rust and the host platform.
//! They are `#[repr(C)]` so layout is deterministic across compilers.

use std::ffi::c_void;

/// A handle to a render element on the native side.
///
/// On Android this wraps a JNI global ref to a `RyeElement` Java object.
/// On iOS this wraps an `NSObject` pointer (via `objc::runtime::Object`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiElement {
    /// Opaque pointer to the native element object.
    pub ptr: *mut c_void,
}

/// A handle to a render text node on the native side.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiText {
    /// Opaque pointer to the native text node object.
    pub ptr: *mut c_void,
}

/// A handle to a generic render node (element or text).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiNode {
    /// Opaque pointer to the native node object.
    pub ptr: *mut c_void,
    /// Whether this node is an element (1) or text (0).
    pub is_element: u8,
}

/// A C-compatible string view — does not own the data.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiStr {
    /// Pointer to UTF-8 string data.
    pub data: *const u8,
    /// Length in bytes.
    pub len: usize,
}

impl FfiStr {
    /// Create an `FfiStr` from a Rust string slice.
    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    /// Convert to a Rust string slice (unsafe — caller must ensure validity).
    ///
    /// # Safety
    /// The pointer must point to valid UTF-8 data for `len` bytes.
    pub unsafe fn as_str(&self) -> &str {
        if self.data.is_null() || self.len == 0 {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.len))
        }
    }
}

/// A C-compatible owned string — the caller owns the allocation.
///
/// The native side must call `rye_ffi_string_free` to release the memory.
#[repr(C)]
pub struct FfiString {
    /// Pointer to heap-allocated UTF-8 string data (null-terminated).
    pub data: *mut u8,
    /// Length in bytes (excluding null terminator).
    pub len: usize,
    /// Capacity of the allocation.
    pub cap: usize,
}

impl FfiString {
    /// Create an `FfiString` from a Rust `String`, transferring ownership.
    pub fn from_string(s: String) -> Self {
        use std::mem::ManuallyDrop;
        let mut s = ManuallyDrop::new(s.into_bytes());
        let data = s.as_mut_ptr();
        let len = s.len();
        let cap = s.capacity();
        Self { data, len, cap }
    }

    /// Convert to a Rust `String` (takes ownership).
    ///
    /// # Safety
    /// The pointer must have been allocated by Rust and be valid.
    pub unsafe fn into_string(self) -> String {
        if self.data.is_null() || self.len == 0 {
            String::new()
        } else {
            String::from_raw_parts(self.data, self.len, self.cap)
        }
    }
}

/// A C-compatible result code.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiResult {
    /// Success.
    Ok = 0,
    /// Invalid argument.
    InvalidArg = -1,
    /// Null pointer.
    NullPtr = -2,
    /// Out of memory.
    OutOfMemory = -3,
    /// Platform error (JNI exception, Obj-C error, etc.).
    PlatformError = -4,
    /// Element not found.
    NotFound = -5,
    /// Index out of bounds.
    OutOfBounds = -6,
}

impl From<FfiResult> for i32 {
    fn from(r: FfiResult) -> Self {
        r as i32
    }
}

/// A render event type — passed from native to Rust when an event fires.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfiEventType {
    /// Click / tap.
    Click = 0,
    /// Long press.
    LongPress = 1,
    /// Mouse / touch move.
    Move = 2,
    /// Key press.
    KeyPress = 3,
    /// Key release.
    KeyRelease = 4,
    /// Scroll.
    Scroll = 5,
    /// Focus.
    Focus = 6,
    /// Blur (lose focus).
    Blur = 7,
    /// Change (input value changed).
    Change = 8,
    /// Submit (form).
    Submit = 9,
}

/// An FFI event — delivered from native code to Rust.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FfiEvent {
    /// The event type.
    pub event_type: FfiEventType,
    /// The element handle that received the event.
    pub element: FfiElement,
    /// X coordinate (for pointer events).
    pub x: f64,
    /// Y coordinate (for pointer events).
    pub y: f64,
    /// Key code (for keyboard events).
    pub key_code: i32,
    /// Additional data (event-specific).
    pub data: *const c_void,
}

/// A render rect — position and size.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FfiRect {
    /// X position (pixels).
    pub x: f32,
    /// Y position (pixels).
    pub y: f32,
    /// Width (pixels).
    pub width: f32,
    /// Height (pixels).
    pub height: f32,
}

/// A render color — RGBA.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct FfiColor {
    /// Red channel (0.0–1.0).
    pub r: f32,
    /// Green channel (0.0–1.0).
    pub g: f32,
    /// Blue channel (0.0–1.0).
    pub b: f32,
    /// Alpha channel (0.0–1.0).
    pub a: f32,
}

impl FfiColor {
    /// Create a color from RGBA floats.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB floats (opaque).
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Transparent color.
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// White color.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    /// Black color.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
}

/// A platform callback for event delivery.
///
/// The native side registers this callback to receive events from Rust.
pub type EventCallback = extern "C" fn(event: FfiEvent, user_data: *mut c_void);

/// A platform callback for requesting a redraw.
pub type RedrawCallback = extern "C" fn(user_data: *mut c_void);

/// A platform callback for logging.
pub type LogCallback = extern "C" fn(level: i32, message: FfiStr);

/// Log level for the log callback.
#[repr(i32)]
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    /// Error.
    Error = 0,
    /// Warning.
    Warn = 1,
    /// Info.
    Info = 2,
    /// Debug.
    Debug = 3,
    /// Trace.
    Trace = 4,
}

/// Platform configuration — passed from native code to initialize the renderer.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlatformConfig {
    /// Width of the rendering surface in physical pixels.
    pub width: u32,
    /// Height of the rendering surface in physical pixels.
    pub height: u32,
    /// Scale factor (DPI) — e.g. 2.0 for retina.
    pub scale_factor: f32,
    /// Pointer to platform-specific surface/view handle.
    pub surface_handle: *mut c_void,
    /// Event callback (called when events fire on elements).
    pub event_callback: Option<EventCallback>,
    /// Redraw callback (called when Rust needs a redraw).
    pub redraw_callback: Option<RedrawCallback>,
    /// User data passed to callbacks.
    pub user_data: *mut c_void,
}

/// The current platform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    /// Android.
    Android,
    /// iOS.
    Ios,
    /// Other (desktop, etc.).
    Other,
}

impl Platform {
    /// Detect the current platform at compile time.
    pub const fn current() -> Self {
        #[cfg(target_os = "android")]
        {
            Platform::Android
        }
        #[cfg(target_os = "ios")]
        {
            Platform::Ios
        }
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            Platform::Other
        }
    }

    /// Whether this platform uses JNI.
    pub const fn uses_jni(&self) -> bool {
        matches!(self, Platform::Android)
    }

    /// Whether this platform uses Obj-C.
    pub const fn uses_objc(&self) -> bool {
        matches!(self, Platform::Ios)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_str_roundtrip() {
        let s = "hello world";
        let ffi = FfiStr::from_str(s);
        unsafe {
            assert_eq!(ffi.as_str(), s);
        }
    }

    #[test]
    fn test_ffi_string_roundtrip() {
        let s = "owned string".to_string();
        let ffi = FfiString::from_string(s.clone());
        unsafe {
            assert_eq!(ffi.into_string(), s);
        }
    }

    #[test]
    fn test_ffi_color_constants() {
        assert_eq!(FfiColor::WHITE.a, 1.0);
        assert_eq!(FfiColor::BLACK.r, 0.0);
        assert_eq!(FfiColor::TRANSPARENT.a, 0.0);
    }

    #[test]
    fn test_ffi_result_codes() {
        assert_eq!(i32::from(FfiResult::Ok), 0);
        assert_eq!(i32::from(FfiResult::PlatformError), -4);
    }

    #[test]
    fn test_platform_detection() {
        let p = Platform::current();
        assert!(matches!(
            p,
            Platform::Android | Platform::Ios | Platform::Other
        ));
    }
}
