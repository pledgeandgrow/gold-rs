//! Objective-C FFI bindings for iOS.
//!
//! This module provides the bridge between Rust and Objective-C/Swift via
//! the `objc` crate. It exposes `#[no_mangle]` `extern "C"` functions that
//! Swift can call directly, and uses `objc::msg_send!` to call back into
//! Objective-C.
//!
//! ## Swift-side class
//! The Swift side must define a class `RyeNative` (inheriting from NSObject)
//! with class methods matching the FFI signatures. The generated Swift
//! bindings (`bindings_gen`) produce the correct class structure.

use std::ffi::c_void;
use std::ffi::CString;

use objc::class;
use objc::msg_send;
use objc::runtime::{Class, Object, Sel};
use objc::sel;
use objc::sel_impl;

use super::types::*;

// ---------------------------------------------------------------------------
// Objective-C runtime helpers
// ---------------------------------------------------------------------------

/// Send a message to the `RyeNative` Objective-C class.
///
/// This is a convenience wrapper around `msg_send!` for class methods.
macro_rules! rye_msg_send {
    ($sel:ident, $arg:expr) => {
        msg_send![$class, sel!($sel), $arg]
    };
}

/// Get the `RyeNative` Objective-C class.
fn rye_class() -> Option<&'static Class> {
    Class::get("RyeNative")
}

/// Call a no-argument class method on `RyeNative`.
fn call_rye_void(sel_str: &str) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel] };
    Ok(())
}

/// Call a class method on `RyeNative` with a string argument.
fn call_rye_with_string(sel_str: &str, arg: &str) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let c_str = CString::new(arg).map_err(|_| FfiResult::InvalidArg)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, c_str.as_ptr()] };
    Ok(())
}

/// Call a class method on `RyeNative` with two string arguments.
fn call_rye_with_two_strings(sel_str: &str, arg1: &str, arg2: &str) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let c1 = CString::new(arg1).map_err(|_| FfiResult::InvalidArg)?;
    let c2 = CString::new(arg2).map_err(|_| FfiResult::InvalidArg)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, c1.as_ptr(), c2.as_ptr()] };
    Ok(())
}

/// Call a class method on `RyeNative` with an integer argument.
fn call_rye_with_int(sel_str: &str, arg: i64) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, arg] };
    Ok(())
}

/// Call a class method on `RyeNative` with an integer and a string.
fn call_rye_with_int_string(sel_str: &str, int_arg: i64, str_arg: &str) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let c_str = CString::new(str_arg).map_err(|_| FfiResult::InvalidArg)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, int_arg, c_str.as_ptr()] };
    Ok(())
}

/// Call a class method on `RyeNative` with two integers.
fn call_rye_with_two_ints(sel_str: &str, arg1: i64, arg2: i64) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, arg1, arg2] };
    Ok(())
}

/// Call a class method on `RyeNative` with three integers.
fn call_rye_with_three_ints(
    sel_str: &str,
    arg1: i64,
    arg2: i64,
    arg3: i64,
) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let sel_name = CString::new(sel_str).unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, arg1, arg2, arg3] };
    Ok(())
}

// ---------------------------------------------------------------------------
// C ABI functions called from Swift
// ---------------------------------------------------------------------------

/// Initialize the rye renderer from Swift.
///
/// Called from Swift as: `rye_native_init(width, height, scale, surface)`
#[no_mangle]
pub extern "C" fn rye_native_init(
    width: u32,
    height: u32,
    scale: f32,
    surface: *mut c_void,
) -> *mut super::bridge::FfiRendererBridge {
    if surface.is_null() || width == 0 || height == 0 {
        return std::ptr::null_mut();
    }

    let config = PlatformConfig {
        width,
        height,
        scale_factor: scale,
        surface_handle: surface,
        event_callback: None,
        redraw_callback: None,
        user_data: std::ptr::null_mut(),
    };

    match super::init(config) {
        Ok(bridge) => Box::into_raw(Box::new(bridge)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Destroy the renderer bridge.
#[no_mangle]
pub extern "C" fn rye_native_destroy(bridge: *mut super::bridge::FfiRendererBridge) {
    if !bridge.is_null() {
        unsafe {
            let _ = Box::from_raw(bridge);
        }
    }
}

/// Create an element. Returns the element index as a handle.
#[no_mangle]
pub extern "C" fn rye_create_element(
    bridge: *mut super::bridge::FfiRendererBridge,
    tag: *const std::os::raw::c_char,
) -> i64 {
    if bridge.is_null() || tag.is_null() {
        return -1;
    }
    let bridge = unsafe { &mut *bridge };
    let tag_str = unsafe { std::ffi::CStr::from_ptr(tag).to_str().unwrap_or("") };
    let _el = bridge.create_element(tag_str);
    (bridge.element_count() - 1) as i64
}

/// Create a text node. Returns the text index as a handle.
#[no_mangle]
pub extern "C" fn rye_create_text(
    bridge: *mut super::bridge::FfiRendererBridge,
    content: *const std::os::raw::c_char,
) -> i64 {
    if bridge.is_null() || content.is_null() {
        return -1;
    }
    let bridge = unsafe { &mut *bridge };
    let content_str = unsafe { std::ffi::CStr::from_ptr(content).to_str().unwrap_or("") };
    let _text = bridge.create_text(content_str);
    (bridge.text_count() - 1) as i64
}

/// Set text content on a text node.
#[no_mangle]
pub extern "C" fn rye_set_text(
    bridge: *mut super::bridge::FfiRendererBridge,
    text_handle: i64,
    content: *const std::os::raw::c_char,
) {
    if bridge.is_null() || content.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let content_str = unsafe { std::ffi::CStr::from_ptr(content).to_str().unwrap_or("") };
    let texts = bridge.texts_lock();
    if let Some(text) = texts.get(text_handle as usize) {
        bridge.set_text(text, content_str);
    }
}

/// Set an attribute on an element.
#[no_mangle]
pub extern "C" fn rye_set_attribute(
    bridge: *mut super::bridge::FfiRendererBridge,
    element_handle: i64,
    name: *const std::os::raw::c_char,
    value: *const std::os::raw::c_char,
) {
    if bridge.is_null() || name.is_null() || value.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let name_str = unsafe { std::ffi::CStr::from_ptr(name).to_str().unwrap_or("") };
    let value_str = unsafe { std::ffi::CStr::from_ptr(value).to_str().unwrap_or("") };
    let elements = bridge.elements_lock();
    if let Some(el) = elements.get(element_handle as usize) {
        bridge.set_attribute(el, name_str, value_str);
    }
}

/// Remove an attribute from an element.
#[no_mangle]
pub extern "C" fn rye_remove_attribute(
    bridge: *mut super::bridge::FfiRendererBridge,
    element_handle: i64,
    name: *const std::os::raw::c_char,
) {
    if bridge.is_null() || name.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let name_str = unsafe { std::ffi::CStr::from_ptr(name).to_str().unwrap_or("") };
    let elements = bridge.elements_lock();
    if let Some(el) = elements.get(element_handle as usize) {
        bridge.remove_attribute(el, name_str);
    }
}

/// Insert a child node into a parent element at the given index.
#[no_mangle]
pub extern "C" fn rye_insert_child(
    bridge: *mut super::bridge::FfiRendererBridge,
    parent_handle: i64,
    child_handle: i64,
    child_is_element: i32,
    index: i32,
) {
    if bridge.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let elements = bridge.elements_lock();
    let texts = bridge.texts_lock();
    let parent = match elements.get(parent_handle as usize) {
        Some(p) => p,
        None => return,
    };

    let child_node = if child_is_element != 0 {
        match elements.get(child_handle as usize) {
            Some(el) => bridge.element_to_node(el),
            None => return,
        }
    } else {
        match texts.get(child_handle as usize) {
            Some(t) => bridge.text_to_node(t),
            None => return,
        }
    };

    bridge.insert_child(parent, &child_node, index as usize);
}

/// Remove a child at the given index.
#[no_mangle]
pub extern "C" fn rye_remove_child(
    bridge: *mut super::bridge::FfiRendererBridge,
    parent_handle: i64,
    index: i32,
) {
    if bridge.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let elements = bridge.elements_lock();
    if let Some(parent) = elements.get(parent_handle as usize) {
        bridge.remove_child(parent, index as usize);
    }
}

/// Replace a child at the given index.
#[no_mangle]
pub extern "C" fn rye_replace_child(
    bridge: *mut super::bridge::FfiRendererBridge,
    parent_handle: i64,
    child_handle: i64,
    child_is_element: i32,
    index: i32,
) {
    if bridge.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let elements = bridge.elements_lock();
    let texts = bridge.texts_lock();
    let parent = match elements.get(parent_handle as usize) {
        Some(p) => p,
        None => return,
    };

    let child_node = if child_is_element != 0 {
        match elements.get(child_handle as usize) {
            Some(el) => bridge.element_to_node(el),
            None => return,
        }
    } else {
        match texts.get(child_handle as usize) {
            Some(t) => bridge.text_to_node(t),
            None => return,
        }
    };

    bridge.replace_child(parent, &child_node, index as usize);
}

/// Move a child from one index to another.
#[no_mangle]
pub extern "C" fn rye_move_child(
    bridge: *mut super::bridge::FfiRendererBridge,
    parent_handle: i64,
    from: i32,
    to: i32,
) {
    if bridge.is_null() {
        return;
    }
    let bridge = unsafe { &mut *bridge };
    let elements = bridge.elements_lock();
    if let Some(parent) = elements.get(parent_handle as usize) {
        bridge.move_child(parent, from as usize, to as usize);
    }
}

/// Request a redraw.
#[no_mangle]
pub extern "C" fn rye_request_redraw(bridge: *mut super::bridge::FfiRendererBridge) {
    if bridge.is_null() {
        return;
    }
    let bridge = unsafe { &*bridge };
    bridge.request_redraw();
}

/// Get the element count (for testing).
#[no_mangle]
pub extern "C" fn rye_element_count(bridge: *mut super::bridge::FfiRendererBridge) -> i32 {
    if bridge.is_null() {
        return 0;
    }
    let bridge = unsafe { &*bridge };
    bridge.element_count() as i32
}

// ---------------------------------------------------------------------------
// Call-back into Objective-C from Rust
// ---------------------------------------------------------------------------

/// Call the `RyeNative` Objective-C class to create a native element.
pub fn objc_create_element(tag: &str) -> Result<(), FfiResult> {
    call_rye_with_string("createElementWithTag:", tag)
}

/// Call the `RyeNative` Objective-C class to create a native text node.
pub fn objc_create_text(content: &str) -> Result<(), FfiResult> {
    call_rye_with_string("createTextWithContent:", content)
}

/// Call the `RyeNative` Objective-C class to set text on a node.
pub fn objc_set_text(handle: i64, content: &str) -> Result<(), FfiResult> {
    call_rye_with_int_string("setText:forNode:", handle, content)
}

/// Call the `RyeNative` Objective-C class to set an attribute.
pub fn objc_set_attribute(handle: i64, name: &str, value: &str) -> Result<(), FfiResult> {
    let cls = rye_class().ok_or(FfiResult::PlatformError)?;
    let c_name = CString::new(name).map_err(|_| FfiResult::InvalidArg)?;
    let c_value = CString::new(value).map_err(|_| FfiResult::InvalidArg)?;
    let sel_name = CString::new("setAttribute:value:forElement:").unwrap();
    let sel = Sel::register(sel_name.as_ptr());
    let _: () = unsafe { msg_send![cls, sel, handle, c_name.as_ptr(), c_value.as_ptr()] };
    Ok(())
}

/// Call the `RyeNative` Objective-C class to remove an attribute.
pub fn objc_remove_attribute(handle: i64, name: &str) -> Result<(), FfiResult> {
    call_rye_with_int_string("removeAttribute:forElement:", handle, name)
}

/// Call the `RyeNative` Objective-C class to insert a child.
pub fn objc_insert_child(parent: i64, child: i64, index: i32) -> Result<(), FfiResult> {
    call_rye_with_three_ints("insertChild:atIndex:element:", parent, child, index as i64)
}

/// Call the `RyeNative` Objective-C class to remove a child.
pub fn objc_remove_child(parent: i64, index: i32) -> Result<(), FfiResult> {
    call_rye_with_two_ints("removeChildAtIndex:element:", parent, index as i64)
}

/// Call the `RyeNative` Objective-C class to replace a child.
pub fn objc_replace_child(parent: i64, child: i64, index: i32) -> Result<(), FfiResult> {
    call_rye_with_three_ints("replaceChild:atIndex:element:", parent, child, index as i64)
}

/// Call the `RyeNative` Objective-C class to move a child.
pub fn objc_move_child(parent: i64, from: i32, to: i32) -> Result<(), FfiResult> {
    call_rye_with_three_ints(
        "moveChildFromIndex:toIndex:element:",
        parent,
        from as i64,
        to as i64,
    )
}

/// Call the `RyeNative` Objective-C class to request a redraw.
pub fn objc_request_redraw() -> Result<(), FfiResult> {
    call_rye_void("requestRedraw")
}

// ---------------------------------------------------------------------------
// Extension trait for bridge to expose internal locks (iOS version)
// ---------------------------------------------------------------------------

impl super::bridge::FfiRendererBridge {
    /// Get a snapshot of elements for Obj-C lookup.
    pub(crate) fn elements_lock(
        &self,
    ) -> std::sync::MutexGuard<'_, Vec<super::bridge::FfiElement>> {
        self.elements.lock().unwrap()
    }

    /// Get a snapshot of text nodes for Obj-C lookup.
    pub(crate) fn texts_lock(&self) -> std::sync::MutexGuard<'_, Vec<super::bridge::FfiText>> {
        self.texts.lock().unwrap()
    }
}
