//! Render tree types for the native renderer.
//!
//! These types form a tree of elements and text nodes that mirrors
//! the component tree. The tree is built via the `Renderer` trait methods
//! and then used for layout (taffy) and rendering (wgpu).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rye_core::renderer::EventHandler;
use taffy::{Layout, Style};

/// Shared mutable element data.
type SharedElement = Rc<RefCell<RenderElementData>>;
/// Shared mutable text data.
type SharedText = Rc<RefCell<RenderTextData>>;

/// A render element — a node in the render tree with a tag, attributes, and children.
#[derive(Clone)]
pub struct RenderElement {
    /// Shared inner data.
    pub inner: SharedElement,
}

/// A text node in the render tree.
#[derive(Clone)]
pub struct RenderText {
    /// Shared inner data.
    pub inner: SharedText,
}

/// A generic render node — either an element or text.
#[derive(Clone)]
pub enum RenderNode {
    /// An element node.
    Element(RenderElement),
    /// A text node.
    Text(RenderText),
}

/// Internal data for a render element.
pub struct RenderElementData {
    /// Unique ID.
    pub id: u64,
    /// Tag name (e.g. "div", "span", "p").
    pub tag: String,
    /// HTML attributes.
    pub attributes: HashMap<String, String>,
    /// Child nodes.
    pub children: Vec<RenderNode>,
    /// Taffy layout style (flexbox properties).
    pub style: Style,
    /// Computed layout result (set after `compute_layout`).
    pub layout: Option<Layout>,
    /// Background color (RGBA, 0.0–1.0).
    pub background_color: [f32; 4],
    /// Text color for children (RGBA, 0.0–1.0).
    pub text_color: [f32; 4],
    /// Font size in pixels.
    pub font_size: f32,
    /// Event handlers keyed by event name.
    pub event_handlers: HashMap<String, EventHandler>,
}

/// Internal data for a text node.
pub struct RenderTextData {
    /// Unique ID.
    pub id: u64,
    /// Text content.
    pub content: String,
    /// Computed layout result.
    pub layout: Option<Layout>,
    /// Text color (RGBA, 0.0–1.0).
    pub color: [f32; 4],
    /// Font size in pixels.
    pub font_size: f32,
}

impl RenderElement {
    /// Create a new render element with the given tag and ID.
    pub fn new(tag: &str, id: u64) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RenderElementData {
                id,
                tag: tag.to_string(),
                attributes: HashMap::new(),
                children: Vec::new(),
                style: Style {
                    display: taffy::Display::Flex,
                    flex_direction: taffy::FlexDirection::Column,
                    ..Default::default()
                },
                layout: None,
                background_color: [0.0, 0.0, 0.0, 0.0],
                text_color: [0.0, 0.0, 0.0, 1.0],
                font_size: 16.0,
                event_handlers: HashMap::new(),
            })),
        }
    }
}

impl RenderText {
    /// Create a new text node with the given content and ID.
    pub fn new(content: &str, id: u64) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RenderTextData {
                id,
                content: content.to_string(),
                layout: None,
                color: [0.0, 0.0, 0.0, 1.0],
                font_size: 16.0,
            })),
        }
    }
}

impl Default for RenderElement {
    fn default() -> Self {
        Self::new("div", 0)
    }
}

impl Default for RenderText {
    fn default() -> Self {
        Self::new("", 0)
    }
}

impl Default for RenderNode {
    fn default() -> Self {
        RenderNode::Element(RenderElement::default())
    }
}
