//! DOM batch protocol — queue DOM mutations and apply them in a single JS call.
//!
//! Instead of N individual Wasm→JS bridge crossings per render cycle,
//! mutations are serialized into a flat JS array and applied by a single
//! JS function call. Target: <5 bridge calls per render cycle.
//!
//! ## How it works
//!
//! 1. `DomRenderer` queues mutations as `DomMutation` enums while batching.
//! 2. On `flush_batch`, mutations are encoded into a `js_sys::Array`.
//! 3. A single JS function (`applyDomMutations`) iterates the array in JS land.
//! 4. Exactly **one** Wasm→JS bridge crossing per flush, regardless of mutation count.

use std::sync::OnceLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{Element, Node, Text};

// Operation codes for the JS batch apply function.
const OP_SET_ATTRIBUTE: u32 = 0;
const OP_REMOVE_ATTRIBUTE: u32 = 1;
const OP_SET_TEXT: u32 = 2;
const OP_INSERT_CHILD: u32 = 3;
const OP_REMOVE_CHILD: u32 = 4;
const OP_REPLACE_CHILD: u32 = 5;
const OP_MOVE_CHILD: u32 = 6;

/// A single DOM mutation queued for batch application.
#[derive(Clone)]
pub enum DomMutation {
    /// Set an attribute on an element.
    SetAttribute {
        el: Element,
        name: String,
        value: String,
    },
    /// Remove an attribute from an element.
    RemoveAttribute {
        el: Element,
        name: String,
    },
    /// Set text content on a text node.
    SetText {
        node: Text,
        content: String,
    },
    /// Insert a child node at a specific index.
    InsertChild {
        parent: Element,
        child: Node,
        index: usize,
    },
    /// Remove the child at a given index.
    RemoveChild {
        parent: Element,
        index: usize,
    },
    /// Replace the child at a given index with a new node.
    ReplaceChild {
        parent: Element,
        new: Node,
        index: usize,
    },
    /// Move a child from one index to another within the same parent.
    MoveChild {
        parent: Element,
        from: usize,
        to: usize,
    },
}

/// Lazily-created JS function that applies an array of DOM mutations in a single call.
///
/// The function body iterates through a flat array of operations and applies
/// each one directly in JS land, avoiding repeated Wasm→JS bridge crossings.
static BATCH_FN: OnceLock<js_sys::Function> = OnceLock::new();

fn get_batch_fn() -> &'static js_sys::Function {
    BATCH_FN.get_or_init(|| {
        js_sys::Function::new_with_args(
            "ops",
            r#"
var len = ops.length;
for (var i = 0; i < len; i++) {
    var op = ops[i];
    switch (op[0]) {
        case 0: op[1].setAttribute(op[2], op[3]); break;
        case 1: op[1].removeAttribute(op[2]); break;
        case 2: op[1].data = op[2]; break;
        case 3:
            var ref3 = op[1].childNodes[op[3]];
            if (ref3) op[1].insertBefore(op[2], ref3);
            else op[1].appendChild(op[2]);
            break;
        case 4:
            var child4 = op[1].childNodes[op[2]];
            if (child4) op[1].removeChild(child4);
            break;
        case 5:
            var old5 = op[1].childNodes[op[3]];
            if (old5) op[1].replaceChild(op[2], old5);
            break;
        case 6:
            var parent6 = op[1];
            var child6 = parent6.childNodes[op[2]];
            if (child6) {
                parent6.removeChild(child6);
                var ref6 = parent6.childNodes[op[3]];
                if (ref6) parent6.insertBefore(child6, ref6);
                else parent6.appendChild(child6);
            }
            break;
    }
}
"#,
        )
    })
}

/// Encode a single `DomMutation` into a `js_sys::Array` for the JS batch function.
fn encode_mutation(mutation: &DomMutation) -> js_sys::Array {
    match mutation {
        DomMutation::SetAttribute { el, name, value } => {
            let arr = js_sys::Array::new_with_length(4);
            arr.set(0, JsValue::from(OP_SET_ATTRIBUTE));
            arr.set(1, el.clone().into());
            arr.set(2, JsValue::from_str(name));
            arr.set(3, JsValue::from_str(value));
            arr
        }
        DomMutation::RemoveAttribute { el, name } => {
            let arr = js_sys::Array::new_with_length(3);
            arr.set(0, JsValue::from(OP_REMOVE_ATTRIBUTE));
            arr.set(1, el.clone().into());
            arr.set(2, JsValue::from_str(name));
            arr
        }
        DomMutation::SetText { node, content } => {
            let arr = js_sys::Array::new_with_length(3);
            arr.set(0, JsValue::from(OP_SET_TEXT));
            arr.set(1, node.clone().into());
            arr.set(2, JsValue::from_str(content));
            arr
        }
        DomMutation::InsertChild {
            parent,
            child,
            index,
        } => {
            let arr = js_sys::Array::new_with_length(4);
            arr.set(0, JsValue::from(OP_INSERT_CHILD));
            arr.set(1, parent.clone().into());
            arr.set(2, child.clone().into());
            arr.set(3, JsValue::from(*index as u32));
            arr
        }
        DomMutation::RemoveChild { parent, index } => {
            let arr = js_sys::Array::new_with_length(3);
            arr.set(0, JsValue::from(OP_REMOVE_CHILD));
            arr.set(1, parent.clone().into());
            arr.set(2, JsValue::from(*index as u32));
            arr
        }
        DomMutation::ReplaceChild {
            parent,
            new,
            index,
        } => {
            let arr = js_sys::Array::new_with_length(4);
            arr.set(0, JsValue::from(OP_REPLACE_CHILD));
            arr.set(1, parent.clone().into());
            arr.set(2, new.clone().into());
            arr.set(3, JsValue::from(*index as u32));
            arr
        }
        DomMutation::MoveChild { parent, from, to } => {
            let arr = js_sys::Array::new_with_length(4);
            arr.set(0, JsValue::from(OP_MOVE_CHILD));
            arr.set(1, parent.clone().into());
            arr.set(2, JsValue::from(*from as u32));
            arr.set(3, JsValue::from(*to as u32));
            arr
        }
    }
}

/// Apply a batch of DOM mutations via a single JS function call.
///
/// This is the core of the batch protocol: all queued mutations are encoded
/// into a flat `js_sys::Array` and passed to `applyDomMutations(ops)` in one
/// Wasm→JS bridge crossing. The JS function iterates the array and applies
/// each operation directly in JS land.
pub fn apply_mutations(mutations: &[DomMutation]) {
    if mutations.is_empty() {
        return;
    }

    let array = js_sys::Array::new_with_length(mutations.len() as u32);

    for (i, mutation) in mutations.iter().enumerate() {
        let encoded = encode_mutation(mutation);
        array.set(i as u32, encoded.into());
    }

    let batch_fn = get_batch_fn();
    let _ = batch_fn.call1(&JsValue::undefined(), &array);
}

/// Apply mutations directly (non-batched fallback).
///
/// Used when batching is disabled — applies each mutation immediately
/// via individual web-sys calls.
pub fn apply_mutation_direct(mutation: &DomMutation) {
    match mutation {
        DomMutation::SetAttribute { el, name, value } => {
            let _ = el.set_attribute(name, value);
        }
        DomMutation::RemoveAttribute { el, name } => {
            let _ = el.remove_attribute(name);
        }
        DomMutation::SetText { node, content } => {
            node.set_data(content);
        }
        DomMutation::InsertChild {
            parent,
            child,
            index,
        } => {
            let node: &web_sys::Node = parent.as_ref();
            let children = node.child_nodes();
            if *index >= children.length() as usize {
                let _ = parent.append_child(child);
            } else if let Some(reference) = children.item(*index as u32) {
                let _ = parent.insert_before(child, Some(&reference));
            }
        }
        DomMutation::RemoveChild { parent, index } => {
            let node: &web_sys::Node = parent.as_ref();
            let children = node.child_nodes();
            if let Some(child) = children.item(*index as u32) {
                let _ = parent.remove_child(&child);
            }
        }
        DomMutation::ReplaceChild {
            parent,
            new,
            index,
        } => {
            let node: &web_sys::Node = parent.as_ref();
            let children = node.child_nodes();
            if let Some(old) = children.item(*index as u32) {
                let _ = parent.replace_child(new, &old);
            }
        }
        DomMutation::MoveChild { parent, from, to } => {
            let node: &web_sys::Node = parent.as_ref();
            let children = node.child_nodes();
            if let Some(child) = children.item(*from as u32) {
                let _ = parent.remove_child(&child);
                let children = node.child_nodes();
                if *to >= children.length() as usize {
                    let _ = parent.append_child(&child);
                } else if let Some(reference) = children.item(*to as u32) {
                    let _ = parent.insert_before(&child, Some(&reference));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_clone() {
        // Ensure DomMutation variants are cloneable (needed for batching)
        let mutation = DomMutation::RemoveAttribute {
            el: web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("div")
                .unwrap(),
            name: "class".to_string(),
        };
        let _cloned = mutation.clone();
    }
}
