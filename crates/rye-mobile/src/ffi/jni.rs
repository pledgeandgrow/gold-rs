//! JNI FFI bindings for Android.
//!
//! This module provides the bridge between Rust and Java/Kotlin via JNI.
//! It exposes `#[no_mangle]` `extern "C"` functions that Java can call
//! directly, and provides helper functions for Rust to call back into Java.
//!
//! ## Java-side class
//! The Kotlin/Java side must define a class `rye.RyeNative` with static
//! methods matching the FFI signatures. The generated Kotlin bindings
//! (`bindings_gen`) produce the correct class structure.

use std::ffi::c_void;
use std::sync::OnceLock;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jlong, jfloat, jdouble, jobject, jstring};
use jni::JNIEnv;

use super::types::*;

/// The global JNI environment, stored after `JNI_OnLoad`.
static JNI_ENV: OnceLock<JniContext> = OnceLock::new();

/// Stored JNI context — the Java VM and class references.
pub struct JniContext {
    /// The Java VM pointer.
    java_vm: *mut jni::sys::JavaVM,
    /// The `rye.RyeNative` class reference (global ref).
    rye_class: jobject,
}

unsafe impl Send for JniContext {}
unsafe impl Sync for JniContext {}

impl JniContext {
    /// Get the Java VM.
    pub fn java_vm(&self) -> *mut jni::sys::JavaVM {
        self.java_vm
    }

    /// Get the rye class reference.
    pub fn rye_class(&self) -> jobject {
        self.rye_class
    }
}

/// Called by the JVM when the native library is loaded.
///
/// Stores the Java VM pointer for later use.
#[no_mangle]
pub extern "C" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint {
    let mut env_ptr: *mut jni::sys::JNIEnv = std::ptr::null_mut();
    unsafe {
        let get_env_fn = (*vm).GetEnv;
        let ret = get_env_fn(vm, &mut env_ptr as *mut *mut jni::sys::JNIEnv, jni::sys::JNI_VERSION_1_8);
        if ret != jni::sys::JNI_OK {
            return jni::sys::JNI_ERR;
        }

        let env = JNIEnv::from_raw(env_ptr);
        let env = match env {
            Ok(e) => e,
            Err(_) => return jni::sys::JNI_ERR,
        };

        // Find the rye.RyeNative class.
        let class = match env.find_class("rye/RyeNative") {
            Ok(c) => c,
            Err(_) => return jni::sys::JNI_ERR,
        };

        // Create a global reference to the class.
        let global_ref = env.new_global_ref(class.as_ref()).unwrap();
        let raw_ref = global_ref.as_raw();

        let ctx = JniContext {
            java_vm: vm,
            rye_class: raw_ref,
        };

        let _ = JNI_ENV.set(ctx);
    }

    jni::sys::JNI_VERSION_1_8
}

/// Called by the JVM when the native library is unloaded.
#[no_mangle]
pub extern "C" fn JNI_OnUnload(vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) {
    // The OnceLock value will be dropped when the process exits.
    let _ = vm;
}

// ---------------------------------------------------------------------------
// Native methods called from Java/Kotlin
// ---------------------------------------------------------------------------

/// Initialize the rye renderer from Java.
///
/// Called from Kotlin as: `external fun nativeInit(width: Int, height: Int, scale: Float, surface: Long): Long`
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_nativeInit(
    mut env: JNIEnv,
    _class: JClass,
    width: jint,
    height: jint,
    scale: jfloat,
    surface: jlong,
) -> jlong {
    let config = PlatformConfig {
        width: width as u32,
        height: height as u32,
        scale_factor: scale,
        surface_handle: surface as *mut c_void,
        event_callback: None,
        redraw_callback: None,
        user_data: std::ptr::null_mut(),
    };

    match super::init(config) {
        Ok(bridge) => {
            let boxed = Box::new(bridge);
            Box::into_raw(boxed) as jlong
        }
        Err(_) => 0,
    }
}

/// Destroy the renderer bridge.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
) {
    if bridge_ptr != 0 {
        unsafe {
            let _ = Box::from_raw(bridge_ptr as *mut super::bridge::FfiRendererBridge);
        }
    }
}

/// Create an element. Returns a handle (index) to the element.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_createElement(
    mut env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    tag: JString,
) -> jlong {
    if bridge_ptr == 0 {
        return -1;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let tag_str: String = env.get_string(&tag).unwrap_or_default().into();
    let el = bridge.create_element(&tag_str);
    // Store the element and return its index as a handle.
    // The bridge internally tracks elements, so we use the count as the handle.
    (bridge.element_count() - 1) as jlong
}

/// Create a text node. Returns a handle (index) to the text node.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_createText(
    mut env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    content: JString,
) -> jlong {
    if bridge_ptr == 0 {
        return -1;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let content_str: String = env.get_string(&content).unwrap_or_default().into();
    let _text = bridge.create_text(&content_str);
    (bridge.text_count() - 1) as jlong
}

/// Set text content on a text node.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_setText(
    mut env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    text_handle: jlong,
    content: JString,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let content_str: String = env.get_string(&content).unwrap_or_default().into();
    let texts = bridge.texts_lock();
    if let Some(text) = texts.get(text_handle as usize) {
        bridge.set_text(text, &content_str);
    }
}

/// Set an attribute on an element.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_setAttribute(
    mut env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    element_handle: jlong,
    name: JString,
    value: JString,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let name_str: String = env.get_string(&name).unwrap_or_default().into();
    let value_str: String = env.get_string(&value).unwrap_or_default().into();
    let elements = bridge.elements_lock();
    if let Some(el) = elements.get(element_handle as usize) {
        bridge.set_attribute(el, &name_str, &value_str);
    }
}

/// Remove an attribute from an element.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_removeAttribute(
    mut env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    element_handle: jlong,
    name: JString,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let name_str: String = env.get_string(&name).unwrap_or_default().into();
    let elements = bridge.elements_lock();
    if let Some(el) = elements.get(element_handle as usize) {
        bridge.remove_attribute(el, &name_str);
    }
}

/// Insert a child node into a parent element at the given index.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_insertChild(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    parent_handle: jlong,
    child_handle: jlong,
    child_is_element: jint,
    index: jint,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let elements = bridge.elements_lock();
    let texts = bridge.texts_lock();
    let parent = elements.get(parent_handle as usize);
    if parent.is_none() {
        return;
    }
    let parent = parent.unwrap();

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

/// Remove a child at the given index from a parent element.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_removeChild(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    parent_handle: jlong,
    index: jint,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let elements = bridge.elements_lock();
    if let Some(parent) = elements.get(parent_handle as usize) {
        bridge.remove_child(parent, index as usize);
    }
}

/// Replace a child at the given index.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_replaceChild(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    parent_handle: jlong,
    child_handle: jlong,
    child_is_element: jint,
    index: jint,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let elements = bridge.elements_lock();
    let texts = bridge.texts_lock();
    let parent = elements.get(parent_handle as usize);
    if parent.is_none() {
        return;
    }
    let parent = parent.unwrap();

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
pub extern "C" fn Java_rye_RyeNative_moveChild(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
    parent_handle: jlong,
    from: jint,
    to: jint,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &mut *(bridge_ptr as *mut super::bridge::FfiRendererBridge) };
    let elements = bridge.elements_lock();
    if let Some(parent) = elements.get(parent_handle as usize) {
        bridge.move_child(parent, from as usize, to as usize);
    }
}

/// Request a redraw.
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_requestRedraw(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
) {
    if bridge_ptr == 0 {
        return;
    }
    let bridge = unsafe { &*(bridge_ptr as *const super::bridge::FfiRendererBridge) };
    bridge.request_redraw();
}

/// Get the element count (for testing).
#[no_mangle]
pub extern "C" fn Java_rye_RyeNative_elementCount(
    _env: JNIEnv,
    _class: JClass,
    bridge_ptr: jlong,
) -> jint {
    if bridge_ptr == 0 {
        return 0;
    }
    let bridge = unsafe { &*(bridge_ptr as *const super::bridge::FfiRendererBridge) };
    bridge.element_count() as jint
}

// ---------------------------------------------------------------------------
// Helper: call back into Java from Rust
// ---------------------------------------------------------------------------

/// Call a static method on `rye.RyeNative` from Rust.
///
/// This attaches the current thread to the JVM if needed and calls the
/// specified static method.
pub fn call_java_static(
    method_name: &str,
    signature: &str,
    args: &[JValue],
) -> Result<(), jni::errors::Error> {
    let ctx = JNI_ENV.get().ok_or(jni::errors::Error::JavaVMNotFound)?;
    let vm = unsafe { jni::JavaVM::from_raw(ctx.java_vm())? };
    let mut env = vm.attach_current_thread()?;

    let class = unsafe { JClass::from_raw(ctx.rye_class()) };
    env.call_static_method(&class, method_name, signature, args)?;
    Ok(())
}

/// Call a static method that returns a string.
pub fn call_java_string(
    method_name: &str,
    signature: &str,
    args: &[JValue],
) -> Result<String, jni::errors::Error> {
    let ctx = JNI_ENV.get().ok_or(jni::errors::Error::JavaVMNotFound)?;
    let vm = unsafe { jni::JavaVM::from_raw(ctx.java_vm())? };
    let mut env = vm.attach_current_thread()?;

    let class = unsafe { JClass::from_raw(ctx.rye_class()) };
    let result = env.call_static_method(&class, method_name, signature, args)?;
    let string_obj: JString = result.l().into();
    let string: String = env.get_string(&string_obj)?.into();
    Ok(string)
}

// ---------------------------------------------------------------------------
// Extension trait for bridge to expose internal locks
// ---------------------------------------------------------------------------

/// Internal helper — provides access to the bridge's element/text vectors.
impl super::bridge::FfiRendererBridge {
    /// Get a snapshot of elements for JNI lookup.
    pub(crate) fn elements_lock(&self) -> std::sync::MutexGuard<'_, Vec<super::bridge::FfiElement>> {
        self.elements.lock().unwrap()
    }

    /// Get a snapshot of text nodes for JNI lookup.
    pub(crate) fn texts_lock(&self) -> std::sync::MutexGuard<'_, Vec<super::bridge::FfiText>> {
        self.texts.lock().unwrap()
    }
}
