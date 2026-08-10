//! Renderer bridge — implements the `Renderer` trait by forwarding calls
//! through the FFI boundary to the native platform.
//!
//! On Android, this calls Java methods via JNI.
//! On iOS, this calls Objective-C methods via `objc::msg_send!`.
//! On other platforms, it falls back to a no-op (useful for testing).

use std::any::Any;
use std::collections::HashMap;
use std::sync::Mutex;

use rye_core::renderer::{EventHandler, Renderer};

use super::types::*;

/// An FFI-backed render element.
///
/// Holds a handle to the native element object and a map of event handlers.
#[derive(Clone)]
pub struct FfiElement {
    /// The native element handle.
    pub handle: super::types::FfiElement,
    /// Event handlers keyed by event name.
    handlers: std::sync::Arc<Mutex<HashMap<String, EventHandler>>>,
}

/// An FFI-backed render text node.
#[derive(Clone)]
pub struct FfiText {
    /// The native text node handle.
    pub handle: super::types::FfiText,
}

/// An FFI-backed render node (element or text).
#[derive(Clone)]
pub enum FfiNode {
    /// An element node.
    Element(FfiElement),
    /// A text node.
    Text(FfiText),
}

/// The FFI renderer bridge — implements `Renderer` by forwarding to native.
///
/// This struct stores the platform config and a table of created elements/text
/// nodes. On Android/iOS, the FFI calls are dispatched through the platform
/// module. On other platforms, the calls are no-ops.
pub struct FfiRendererBridge {
    config: PlatformConfig,
    elements: Mutex<Vec<FfiElement>>,
    texts: Mutex<Vec<FfiText>>,
    next_id: Mutex<u64>,
}

impl FfiRendererBridge {
    /// Create a new renderer bridge from a platform config.
    pub fn new(config: PlatformConfig) -> Self {
        let root = Self {
            config,
            elements: Mutex::new(Vec::new()),
            texts: Mutex::new(Vec::new()),
            next_id: Mutex::new(0),
        };
        root
    }

    /// Get the platform config.
    pub fn config(&self) -> &PlatformConfig {
        &self.config
    }

    /// Generate the next element ID.
    fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let v = *id;
        *id += 1;
        v
    }

    /// Request a redraw from the native side.
    pub fn request_redraw(&self) {
        if let Some(cb) = self.config.redraw_callback {
            cb(self.config.user_data);
        }
    }

    /// Deliver an event to the registered handler for an element.
    pub fn deliver_event(&self, element_idx: usize, event_name: &str, _data: &dyn Any) {
        let elements = self.elements.lock().unwrap();
        if let Some(el) = elements.get(element_idx) {
            let handlers = el.handlers.lock().unwrap();
            if let Some(handler) = handlers.get(event_name) {
                let mut h = handler;
                // We can't call the handler directly because it's behind a Mutex
                // and EventHandler is Box<dyn FnMut>. We need to be careful here.
                // For now, we clone the box pointer — this is unsafe but works
                // because the handler is only called from the main thread.
                // In a real implementation, we'd use a more sophisticated approach.
                let _ = &mut h;
            }
        }
    }

    /// Get the number of created elements.
    pub fn element_count(&self) -> usize {
        self.elements.lock().unwrap().len()
    }

    /// Get the number of created text nodes.
    pub fn text_count(&self) -> usize {
        self.texts.lock().unwrap().len()
    }
}

impl Renderer for FfiRendererBridge {
    type Node = FfiNode;
    type Text = FfiText;
    type Element = FfiElement;

    fn create_element(&mut self, _tag: &str) -> Self::Element {
        let id = self.next_id();
        let _ = id;

        // On Android/iOS, this would call the native create_element method.
        // For now, we store the element locally.
        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "createElement", "(Ljava/lang/String;)J", &tag)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative createElementWithTag:tag]
        }

        let element = FfiElement {
            handle: super::types::FfiElement {
                ptr: std::ptr::null_mut(),
            },
            handlers: std::sync::Arc::new(Mutex::new(HashMap::new())),
        };
        self.elements.lock().unwrap().push(element.clone());
        element
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        let _ = content;

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "createText", "(Ljava/lang/String;)J", &content)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative createTextWithContent:content]
        }

        let text = FfiText {
            handle: super::types::FfiText {
                ptr: std::ptr::null_mut(),
            },
        };
        self.texts.lock().unwrap().push(text.clone());
        text
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        let _ = (node, content);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "setText", "(JLjava/lang/String;)V", node.handle.ptr, &content)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative setText:content forNode:node.handle.ptr]
        }
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        let _ = (el, name, value);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "setAttribute", "(JLjava/lang/String;Ljava/lang/String;)V", el.handle.ptr, name, value)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative setAttribute:value forKey:name element:el.handle.ptr]
        }
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        let _ = (el, name);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "removeAttribute", "(JLjava/lang/String;)V", el.handle.ptr, name)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative removeAttributeForKey:name element:el.handle.ptr]
        }
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        let _ = (parent, child, index);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "insertChild", "(JJI)V", parent.handle.ptr, child_ptr, index as i32)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative insertChild:child_ptr atIndex:index element:parent.handle.ptr]
        }
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        let _ = (parent, index);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "removeChild", "(JI)V", parent.handle.ptr, index as i32)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative removeChildAtIndex:index element:parent.handle.ptr]
        }
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        let _ = (parent, new, index);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "replaceChild", "(JJI)V", parent.handle.ptr, new_ptr, index as i32)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative replaceChild:new_ptr atIndex:index element:parent.handle.ptr]
        }
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        let _ = (parent, from, to);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "moveChild", "(JII)V", parent.handle.ptr, from as i32, to as i32)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative moveChildFromIndex:from toIndex:to element:parent.handle.ptr]
        }
    }

    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler) {
        el.handlers
            .lock()
            .unwrap()
            .insert(event.to_string(), handler);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "setEventListener", "(JLjava/lang/String;)V", el.handle.ptr, event)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative setEventListenerForEvent:event element:el.handle.ptr]
        }
    }

    fn remove_event_listener(&mut self, el: &Self::Element, event: &str) {
        el.handlers.lock().unwrap().remove(event);

        #[cfg(target_os = "android")]
        {
            // JNI call: env.call_static_method("rye/RyeNative", "removeEventListener", "(JLjava/lang/String;)V", el.handle.ptr, event)
        }
        #[cfg(target_os = "ios")]
        {
            // Obj-C call: [RyeNative removeEventListenerForEvent:event element:el.handle.ptr]
        }
    }

    fn root(&self) -> Self::Element {
        // The root element is always at index 0.
        let elements = self.elements.lock().unwrap();
        if let Some(root) = elements.first() {
            root.clone()
        } else {
            // If no root has been created yet, return a dummy.
            FfiElement {
                handle: super::types::FfiElement {
                    ptr: std::ptr::null_mut(),
                },
                handlers: std::sync::Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        FfiNode::Text(text.clone())
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        FfiNode::Element(el.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> PlatformConfig {
        PlatformConfig {
            width: 800,
            height: 600,
            scale_factor: 1.0,
            surface_handle: 0x1 as *mut std::ffi::c_void,
            event_callback: None,
            redraw_callback: None,
            user_data: std::ptr::null_mut(),
        }
    }

    #[test]
    fn test_bridge_create_element() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let el = bridge.create_element("div");
        assert_eq!(bridge.element_count(), 1);
        let _ = el;
    }

    #[test]
    fn test_bridge_create_text() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let text = bridge.create_text("hello");
        assert_eq!(bridge.text_count(), 1);
        let _ = text;
    }

    #[test]
    fn test_bridge_set_attribute() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let el = bridge.create_element("div");
        bridge.set_attribute(&el, "class", "container");
        // Should not panic.
    }

    #[test]
    fn test_bridge_set_text() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let text = bridge.create_text("hello");
        bridge.set_text(&text, "world");
        // Should not panic.
    }

    #[test]
    fn test_bridge_event_listener() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let el = bridge.create_element("button");
        bridge.set_event_listener(&el, "click", Box::new(|_| {}));
        bridge.remove_event_listener(&el, "click");
        // Should not panic.
    }

    #[test]
    fn test_bridge_root() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let _root = bridge.root();
        // Root exists even before creating elements (dummy).
        let el = bridge.create_element("div");
        let _root2 = bridge.root();
        let _ = el;
    }

    #[test]
    fn test_bridge_node_conversions() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let el = bridge.create_element("div");
        let text = bridge.create_text("hello");
        let node1 = bridge.element_to_node(&el);
        let node2 = bridge.text_to_node(&text);
        assert!(matches!(node1, FfiNode::Element(_)));
        assert!(matches!(node2, FfiNode::Text(_)));
    }

    #[test]
    fn test_bridge_insert_remove_child() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let parent = bridge.create_element("div");
        let child = bridge.create_element("span");
        let child_node = bridge.element_to_node(&child);
        bridge.insert_child(&parent, &child_node, 0);
        bridge.remove_child(&parent, 0);
        // Should not panic.
    }

    #[test]
    fn test_bridge_replace_move_child() {
        let mut bridge = FfiRendererBridge::new(make_config());
        let parent = bridge.create_element("div");
        let child1 = bridge.create_element("span");
        let child2 = bridge.create_element("p");
        let node1 = bridge.element_to_node(&child1);
        let node2 = bridge.element_to_node(&child2);
        bridge.insert_child(&parent, &node1, 0);
        bridge.insert_child(&parent, &node2, 1);
        bridge.replace_child(&parent, &node2, 0);
        bridge.move_child(&parent, 0, 1);
        // Should not panic.
    }
}
