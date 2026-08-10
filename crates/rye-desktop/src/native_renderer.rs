//! Native GPU renderer — implements the `Renderer` trait using wgpu.
//!
//! Builds a render tree via the trait methods, computes layout with taffy,
//! shapes text with cosmic-text, and renders to the GPU with wgpu.
//!
//! The GPU context is optional — when absent, the renderer still builds
//! the render tree (useful for testing). Call `attach_gpu` to initialize
//! the wgpu pipeline from a window handle.

use rye_core::renderer::{EventHandler, Renderer};
use taffy::{AvailableSpace, Layout, NodeId, Size, TaffyTree};

use crate::gpu::{GpuContext, InstanceData};
use crate::render_tree::{RenderElement, RenderNode, RenderText};

/// Native GPU renderer using wgpu for desktop platforms.
pub struct NativeRenderer {
    /// The root element of the render tree.
    root: RenderElement,
    /// Next unique ID for render nodes.
    next_id: u64,
    /// GPU context (initialized when a window is available).
    pub gpu: Option<GpuContext>,
    /// Taffy layout tree.
    taffy: TaffyTree<()>,
    /// Taffy node ID for the root.
    taffy_root: Option<NodeId>,
    /// cosmic-text font system (initialized lazily).
    font_system: Option<cosmic_text::FontSystem>,
    /// cosmic-text swash cache for glyph rasterization.
    swash_cache: Option<cosmic_text::SwashCache>,
    /// Whether the layout needs to be recomputed.
    layout_dirty: bool,
}

impl NativeRenderer {
    /// Create a new native renderer without GPU initialization.
    pub fn new() -> Self {
        let root = RenderElement::new("root", 0);
        Self {
            root,
            next_id: 1,
            gpu: None,
            taffy: TaffyTree::new(),
            taffy_root: None,
            font_system: None,
            swash_cache: None,
            layout_dirty: true,
        }
    }

    /// Attach a GPU context from a window handle.
    pub fn attach_gpu(&mut self, window: &dyn crate::gpu::WindowHandle) {
        self.gpu = Some(GpuContext::new(window));
        self.layout_dirty = true;
    }

    /// Ensure the font system and swash cache are initialized.
    fn ensure_text_resources(&mut self) {
        if self.font_system.is_none() {
            self.font_system = Some(cosmic_text::FontSystem::new());
        }
        if self.swash_cache.is_none() {
            self.swash_cache = Some(cosmic_text::SwashCache::new());
        }
    }

    /// Rebuild the taffy tree from the render tree and compute layout.
    pub fn compute_layout(&mut self, width: f32, height: f32) {
        self.taffy = TaffyTree::new();
        let root_id = self.build_taffy_node(&self.root.clone());
        self.taffy_root = Some(root_id);

        let _ = self.taffy.compute_layout(
            root_id,
            Size {
                width: AvailableSpace::Definite(width),
                height: AvailableSpace::Definite(height),
            },
        );

        self.collect_layouts(&self.root.clone(), root_id);
        self.layout_dirty = false;
    }

    /// Recursively build taffy nodes from the render tree.
    fn build_taffy_node(&mut self, element: &RenderElement) -> NodeId {
        let data = element.inner.borrow();
        let style = data.style.clone();

        if data.children.is_empty() {
            let id = self
                .taffy
                .new_leaf(style)
                .unwrap_or_else(|_| self.taffy.new_leaf(taffy::Style::default()).unwrap());
            id
        } else {
            let child_ids: Vec<NodeId> = data
                .children
                .iter()
                .map(|child| match child {
                    RenderNode::Element(el) => self.build_taffy_node(el),
                    RenderNode::Text(text) => {
                        let text_data = text.inner.borrow();
                        let estimated_width =
                            text_data.content.chars().count() as f32 * text_data.font_size * 0.5;
                        let estimated_height = text_data.font_size * 1.2;
                        let text_style = taffy::Style {
                            size: Size {
                                width: taffy::Dimension::Length(estimated_width),
                                height: taffy::Dimension::Length(estimated_height),
                            },
                            ..Default::default()
                        };
                        self.taffy.new_leaf(text_style).unwrap_or_else(|_| {
                            self.taffy.new_leaf(taffy::Style::default()).unwrap()
                        })
                    }
                })
                .collect();

            self.taffy
                .new_with_children(style, &child_ids)
                .unwrap_or_else(|_| {
                    self.taffy
                        .new_with_children(taffy::Style::default(), &child_ids)
                        .unwrap()
                })
        }
    }

    /// Recursively collect layout results from taffy and store them in the render tree.
    fn collect_layouts(&self, element: &RenderElement, node_id: NodeId) {
        if let Ok(layout) = self.taffy.layout(node_id) {
            element.inner.borrow_mut().layout = Some(*layout);
        }

        let children: Vec<RenderNode> = element.inner.borrow().children.clone();
        let child_ids: Vec<NodeId> = self.taffy.children(node_id).unwrap_or_default().to_vec();

        for (child, child_id) in children.iter().zip(child_ids.iter()) {
            match child {
                RenderNode::Element(el) => {
                    self.collect_layouts(el, *child_id);
                }
                RenderNode::Text(text) => {
                    if let Ok(layout) = self.taffy.layout(*child_id) {
                        text.inner.borrow_mut().layout = Some(*layout);
                    }
                }
            }
        }
    }

    /// Collect draw instances from the render tree for GPU rendering.
    fn collect_instances(&mut self) -> Vec<InstanceData> {
        let mut instances = Vec::new();
        let atlas_size = 1024.0f32;
        let white = self.gpu.as_ref().map(|g| g.glyph_atlas.white_pixel());

        if let Some(white) = white {
            self.collect_element_instances(&self.root.clone(), &mut instances, white, atlas_size);
        }

        instances
    }

    /// Recursively collect draw instances from an element and its children.
    fn collect_element_instances(
        &mut self,
        element: &RenderElement,
        instances: &mut Vec<InstanceData>,
        white: crate::glyph_atlas::GlyphEntry,
        atlas_size: f32,
    ) {
        let data = element.inner.borrow();
        let layout = data.layout;
        let bg = data.background_color;
        let children = data.children.clone();
        drop(data);

        if let Some(layout) = layout {
            if bg[3] > 0.0 {
                instances.push(InstanceData {
                    x: layout.location.x,
                    y: layout.location.y,
                    w: layout.size.width,
                    h: layout.size.height,
                    u: white.x as f32 / atlas_size,
                    v: white.y as f32 / atlas_size,
                    uw: 1.0 / atlas_size,
                    vh: 1.0 / atlas_size,
                    r: bg[0],
                    g: bg[1],
                    b: bg[2],
                    a: bg[3],
                });
            }

            // Collect text instances for children.
            let child_layouts: Vec<Option<Layout>> = children
                .iter()
                .map(|c| match c {
                    RenderNode::Element(el) => el.inner.borrow().layout,
                    RenderNode::Text(t) => t.inner.borrow().layout,
                })
                .collect();

            for (child, child_layout) in children.iter().zip(child_layouts.iter()) {
                if let RenderNode::Text(text) = child {
                    if let Some(cl) = child_layout {
                        let text_data = text.inner.borrow();
                        self.collect_text_instances(
                            &text_data.content,
                            text_data.font_size,
                            text_data.color,
                            layout.location.x + cl.location.x,
                            layout.location.y + cl.location.y,
                            cl.size.width,
                            instances,
                            atlas_size,
                        );
                    }
                }
            }
        }

        // Recurse into element children.
        for child in &children {
            if let RenderNode::Element(el) = child {
                self.collect_element_instances(el, instances, white, atlas_size);
            }
        }
    }

    /// Shape text with cosmic-text and collect glyph instances.
    #[allow(clippy::too_many_arguments)]
    fn collect_text_instances(
        &mut self,
        content: &str,
        font_size: f32,
        color: [f32; 4],
        x: f32,
        y: f32,
        max_width: f32,
        instances: &mut Vec<InstanceData>,
        atlas_size: f32,
    ) {
        if content.is_empty() {
            return;
        }

        self.ensure_text_resources();

        let font_system = self.font_system.as_mut().unwrap();
        let swash_cache = self.swash_cache.as_mut().unwrap();
        let gpu = self.gpu.as_mut().unwrap();

        let mut buffer = cosmic_text::Buffer::new(
            font_system,
            cosmic_text::Metrics::new(font_size, font_size * 1.2),
        );
        buffer.set_size(font_system, Some(max_width), None);
        buffer.set_text(
            font_system,
            content,
            cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(font_system, false);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, run.line_y), 1.0);
                let entry = gpu.glyph_atlas.add_glyph(
                    &gpu.device,
                    &gpu.queue,
                    font_system,
                    swash_cache,
                    physical.cache_key,
                );

                if let Some(entry) = entry {
                    instances.push(InstanceData {
                        x: x + physical.x as f32,
                        y: y + physical.y as f32,
                        w: entry.width as f32,
                        h: entry.height as f32,
                        u: entry.x as f32 / atlas_size,
                        v: entry.y as f32 / atlas_size,
                        uw: entry.width as f32 / atlas_size,
                        vh: entry.height as f32 / atlas_size,
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: color[3],
                    });
                }
            }
        }
    }

    /// Render a frame to the GPU.
    pub fn render_frame(&mut self) {
        let (width, height) = match &self.gpu {
            Some(gpu) => (gpu.width as f32, gpu.height as f32),
            None => return,
        };

        if self.layout_dirty {
            self.compute_layout(width, height);
        }

        let instances = self.collect_instances();

        if let Some(gpu) = self.gpu.as_mut() {
            gpu.render(&instances);
        }
    }

    /// Resize the GPU surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.resize(width, height);
            self.layout_dirty = true;
        }
    }
}

impl Default for NativeRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for NativeRenderer {
    type Node = RenderNode;
    type Text = RenderText;
    type Element = RenderElement;

    fn create_element(&mut self, tag: &str) -> Self::Element {
        let el = RenderElement::new(tag, self.next_id);
        self.next_id += 1;
        el
    }

    fn create_text(&mut self, content: &str) -> Self::Text {
        let text = RenderText::new(content, self.next_id);
        self.next_id += 1;
        text
    }

    fn set_text(&mut self, node: &Self::Text, content: &str) {
        node.inner.borrow_mut().content = content.to_string();
        self.layout_dirty = true;
    }

    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str) {
        let mut data = el.inner.borrow_mut();
        data.attributes.insert(name.to_string(), value.to_string());

        match name {
            "style" => {
                data.style = parse_style(value, data.style.clone());
            }
            "bg" | "background" => {
                data.background_color = parse_color(value, data.background_color);
            }
            "color" => {
                data.text_color = parse_color(value, data.text_color);
            }
            "font-size" => {
                if let Ok(size) = value.parse::<f32>() {
                    data.font_size = size;
                }
            }
            _ => {}
        }
        self.layout_dirty = true;
    }

    fn remove_attribute(&mut self, el: &Self::Element, name: &str) {
        el.inner.borrow_mut().attributes.remove(name);
        self.layout_dirty = true;
    }

    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize) {
        let mut data = parent.inner.borrow_mut();
        let clamped = index.min(data.children.len());
        data.children.insert(clamped, child.clone());
        self.layout_dirty = true;
    }

    fn remove_child(&mut self, parent: &Self::Element, index: usize) {
        let mut data = parent.inner.borrow_mut();
        if index < data.children.len() {
            data.children.remove(index);
            self.layout_dirty = true;
        }
    }

    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize) {
        let mut data = parent.inner.borrow_mut();
        if index < data.children.len() {
            data.children[index] = new.clone();
            self.layout_dirty = true;
        }
    }

    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler) {
        el.inner
            .borrow_mut()
            .event_handlers
            .insert(event.to_string(), handler);
    }

    fn remove_event_listener(&mut self, el: &Self::Element, event: &str) {
        el.inner.borrow_mut().event_handlers.remove(event);
    }

    fn root(&self) -> Self::Element {
        self.root.clone()
    }

    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize) {
        let mut data = parent.inner.borrow_mut();
        if from < data.children.len() {
            let child = data.children.remove(from);
            let to = to.min(data.children.len());
            data.children.insert(to, child);
            self.layout_dirty = true;
        }
    }

    fn text_to_node(&self, text: &Self::Text) -> Self::Node {
        RenderNode::Text(text.clone())
    }

    fn element_to_node(&self, el: &Self::Element) -> Self::Node {
        RenderNode::Element(el.clone())
    }
}

/// Parse a CSS-like style string and update the taffy style.
fn parse_style(value: &str, current: taffy::Style) -> taffy::Style {
    let mut style = current;
    for decl in value.split(';') {
        let decl = decl.trim();
        if let Some((prop, val)) = decl.split_once(':') {
            let prop = prop.trim();
            let val = val.trim();
            match prop {
                "display" => match val {
                    "flex" => style.display = taffy::Display::Flex,
                    "block" => style.display = taffy::Display::Block,
                    "none" => style.display = taffy::Display::None,
                    _ => {}
                },
                "flex-direction" => match val {
                    "row" => style.flex_direction = taffy::FlexDirection::Row,
                    "column" => style.flex_direction = taffy::FlexDirection::Column,
                    "row-reverse" => style.flex_direction = taffy::FlexDirection::RowReverse,
                    "column-reverse" => style.flex_direction = taffy::FlexDirection::ColumnReverse,
                    _ => {}
                },
                "justify-content" => match val {
                    "center" => style.justify_content = Some(taffy::JustifyContent::Center),
                    "flex-start" => style.justify_content = Some(taffy::JustifyContent::FlexStart),
                    "flex-end" => style.justify_content = Some(taffy::JustifyContent::FlexEnd),
                    "space-between" => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceBetween)
                    }
                    "space-around" => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceAround)
                    }
                    "space-evenly" => {
                        style.justify_content = Some(taffy::JustifyContent::SpaceEvenly)
                    }
                    _ => {}
                },
                "align-items" => match val {
                    "center" => style.align_items = Some(taffy::AlignItems::Center),
                    "flex-start" => style.align_items = Some(taffy::AlignItems::FlexStart),
                    "flex-end" => style.align_items = Some(taffy::AlignItems::FlexEnd),
                    "stretch" => style.align_items = Some(taffy::AlignItems::Stretch),
                    _ => {}
                },
                "width" => {
                    if let Ok(px) = val.trim_end_matches("px").parse::<f32>() {
                        style.size.width = taffy::Dimension::Length(px);
                    } else if val == "auto" {
                        style.size.width = taffy::Dimension::Auto;
                    }
                }
                "height" => {
                    if let Ok(px) = val.trim_end_matches("px").parse::<f32>() {
                        style.size.height = taffy::Dimension::Length(px);
                    } else if val == "auto" {
                        style.size.height = taffy::Dimension::Auto;
                    }
                }
                "padding" => {
                    if let Ok(px) = val.trim_end_matches("px").parse::<f32>() {
                        style.padding = taffy::Rect::length(px);
                    }
                }
                "gap" | "row-gap" => {
                    if let Ok(px) = val.trim_end_matches("px").parse::<f32>() {
                        style.gap.width = taffy::LengthPercentage::Length(px);
                    }
                }
                "column-gap" => {
                    if let Ok(px) = val.trim_end_matches("px").parse::<f32>() {
                        style.gap.height = taffy::LengthPercentage::Length(px);
                    }
                }
                _ => {}
            }
        }
    }
    style
}

/// Parse a color string (hex or named) into an RGBA array.
fn parse_color(value: &str, fallback: [f32; 4]) -> [f32; 4] {
    let value = value.trim();
    if value.starts_with('#') {
        let hex = &value[1..];
        match hex.len() {
            6 => {
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    let r = ((n >> 16) & 0xFF) as f32 / 255.0;
                    let g = ((n >> 8) & 0xFF) as f32 / 255.0;
                    let b = (n & 0xFF) as f32 / 255.0;
                    return [r, g, b, 1.0];
                }
            }
            8 => {
                if let Ok(n) = u64::from_str_radix(hex, 16) {
                    let r = ((n >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((n >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((n >> 8) & 0xFF) as f32 / 255.0;
                    let a = (n & 0xFF) as f32 / 255.0;
                    return [r, g, b, a];
                }
            }
            _ => {}
        }
    }
    match value {
        "black" => [0.0, 0.0, 0.0, 1.0],
        "white" => [1.0, 1.0, 1.0, 1.0],
        "red" => [1.0, 0.0, 0.0, 1.0],
        "green" => [0.0, 1.0, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0, 1.0],
        "transparent" => [0.0, 0.0, 0.0, 0.0],
        _ => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rye_core::renderer::Renderer;

    #[test]
    fn test_render_tree_building() {
        let mut renderer = NativeRenderer::new();
        let root = renderer.root();

        // Create a div element
        let div = renderer.create_element("div");
        renderer.set_attribute(
            &div,
            "style",
            "display:flex;flex-direction:column;padding:10px",
        );
        renderer.set_attribute(&div, "bg", "#3b82f6");

        // Create a text child
        let text = renderer.create_text("Hello, rye!");
        let text_node = renderer.text_to_node(&text);
        renderer.insert_child(&div, &text_node, 0);

        // Insert div into root
        let div_node = renderer.element_to_node(&div);
        renderer.insert_child(&root, &div_node, 0);

        // Verify the tree structure
        let root_data = root.inner.borrow();
        assert_eq!(root_data.children.len(), 1);

        match &root_data.children[0] {
            RenderNode::Element(el) => {
                let el_data = el.inner.borrow();
                assert_eq!(el_data.tag, "div");
                let bg = el_data.background_color;
                assert!((bg[0] - 0.231).abs() < 0.01, "red: {}", bg[0]);
                assert!((bg[1] - 0.510).abs() < 0.01, "green: {}", bg[1]);
                assert!((bg[2] - 0.965).abs() < 0.01, "blue: {}", bg[2]);
                assert_eq!(bg[3], 1.0);
                assert_eq!(el_data.children.len(), 1);
                assert_eq!(el_data.style.padding, taffy::Rect::length(10.0));

                match &el_data.children[0] {
                    RenderNode::Text(t) => {
                        assert_eq!(t.inner.borrow().content, "Hello, rye!");
                    }
                    _ => panic!("Expected text child"),
                }
            }
            _ => panic!("Expected element child"),
        }
    }

    #[test]
    fn test_layout_computation() {
        let mut renderer = NativeRenderer::new();
        let root = renderer.root();

        // Create a flex column container with two children
        let container = renderer.create_element("div");
        renderer.set_attribute(
            &container,
            "style",
            "display:flex;flex-direction:column;width:300px;height:200px",
        );

        let child1 = renderer.create_element("div");
        renderer.set_attribute(&child1, "style", "width:100px;height:50px");
        let child2 = renderer.create_element("div");
        renderer.set_attribute(&child2, "style", "width:200px;height:30px");

        let c1_node = renderer.element_to_node(&child1);
        let c2_node = renderer.element_to_node(&child2);
        renderer.insert_child(&container, &c1_node, 0);
        renderer.insert_child(&container, &c2_node, 1);

        let container_node = renderer.element_to_node(&container);
        renderer.insert_child(&root, &container_node, 0);

        // Compute layout at 800x600
        renderer.compute_layout(800.0, 600.0);

        // Verify container layout
        let container_data = container.inner.borrow();
        let container_layout = container_data.layout.expect("container should have layout");
        assert_eq!(container_layout.size.width, 300.0);
        assert_eq!(container_layout.size.height, 200.0);

        // Verify children are stacked vertically (column direction)
        let child1_data = child1.inner.borrow();
        let child1_layout = child1_data.layout.expect("child1 should have layout");
        assert_eq!(child1_layout.size.width, 100.0);
        assert_eq!(child1_layout.size.height, 50.0);
        // First child should be at y=0 (top of container)
        assert_eq!(child1_layout.location.y, 0.0);

        let child2_data = child2.inner.borrow();
        let child2_layout = child2_data.layout.expect("child2 should have layout");
        assert_eq!(child2_layout.size.width, 200.0);
        assert_eq!(child2_layout.size.height, 30.0);
        // Second child should be below first child (y = 50)
        assert_eq!(child2_layout.location.y, 50.0);
    }

    #[test]
    fn test_style_parsing() {
        let style = parse_style("display:flex;flex-direction:row;justify-content:center;align-items:center;padding:20px", taffy::Style::default());
        assert_eq!(style.display, taffy::Display::Flex);
        assert_eq!(style.flex_direction, taffy::FlexDirection::Row);
        assert_eq!(style.justify_content, Some(taffy::JustifyContent::Center));
        assert_eq!(style.align_items, Some(taffy::AlignItems::Center));
        assert_eq!(style.padding, taffy::Rect::length(20.0));
    }

    #[test]
    fn test_color_parsing() {
        assert_eq!(parse_color("#ff0000", [0.0; 4]), [1.0, 0.0, 0.0, 1.0]);
        let alpha_color = parse_color("#00ff0080", [0.0; 4]);
        assert!(
            (alpha_color[3] - 0.502).abs() < 0.01,
            "alpha: {}",
            alpha_color[3]
        );
        assert_eq!(parse_color("blue", [0.0; 4]), [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(parse_color("transparent", [0.0; 4]), [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(parse_color("unknown", [0.5; 4]), [0.5; 4]); // fallback
    }

    #[test]
    fn test_text_node_creation() {
        let mut renderer = NativeRenderer::new();
        let text = renderer.create_text("Hello World");
        assert_eq!(text.inner.borrow().content, "Hello World");
        assert_eq!(text.inner.borrow().font_size, 16.0); // default

        renderer.set_text(&text, "Updated");
        assert_eq!(text.inner.borrow().content, "Updated");
    }

    #[test]
    fn test_event_handler_storage() {
        let mut renderer = NativeRenderer::new();
        let button = renderer.create_element("button");

        let handler: EventHandler = Box::new(|_| {});
        renderer.set_event_listener(&button, "click", handler);

        assert!(button.inner.borrow().event_handlers.contains_key("click"));

        renderer.remove_event_listener(&button, "click");
        assert!(!button.inner.borrow().event_handlers.contains_key("click"));
    }

    #[test]
    fn test_child_manipulation() {
        let mut renderer = NativeRenderer::new();
        let parent = renderer.create_element("div");

        let child1 = renderer.create_element("span");
        let child2 = renderer.create_element("span");
        let child3 = renderer.create_element("span");

        renderer.insert_child(&parent, &renderer.element_to_node(&child1), 0);
        renderer.insert_child(&parent, &renderer.element_to_node(&child2), 1);
        renderer.insert_child(&parent, &renderer.element_to_node(&child3), 2);

        assert_eq!(parent.inner.borrow().children.len(), 3);

        // Remove middle child
        renderer.remove_child(&parent, 1);
        assert_eq!(parent.inner.borrow().children.len(), 2);

        // Move first child to end
        renderer.move_child(&parent, 0, 1);
        assert_eq!(parent.inner.borrow().children.len(), 2);
    }

    #[test]
    fn test_layout_dirty_flag() {
        let mut renderer = NativeRenderer::new();
        assert!(renderer.layout_dirty); // starts dirty

        renderer.compute_layout(800.0, 600.0);
        assert!(!renderer.layout_dirty); // clean after layout

        // Adding a child should mark dirty
        let root = renderer.root();
        let child = renderer.create_element("div");
        renderer.insert_child(&root, &renderer.element_to_node(&child), 0);
        assert!(renderer.layout_dirty);
    }

    #[test]
    fn test_full_pipeline_render_tree_to_layout() {
        // Prove the full desktop pipeline: build render tree → compute layout
        // → verify geometry. This is what happens every frame in the desktop app.
        let mut renderer = NativeRenderer::new();
        let root = renderer.root();

        // Build a simple card layout: container > [header, body]
        let container = renderer.create_element("div");
        renderer.set_attribute(
            &container,
            "style",
            "display:flex;flex-direction:column;width:400px;height:300px;padding:20px",
        );
        renderer.set_attribute(&container, "bg", "#ffffff");

        let header = renderer.create_element("div");
        renderer.set_attribute(&header, "style", "width:100%;height:60px");
        renderer.set_attribute(&header, "bg", "#3b82f6");
        let header_text = renderer.create_text("Dashboard");
        renderer.insert_child(&header, &renderer.text_to_node(&header_text), 0);

        let body = renderer.create_element("div");
        renderer.set_attribute(&body, "style", "width:100%;height:200px");
        renderer.set_attribute(&body, "bg", "#f3f4f6");
        let body_text = renderer.create_text("Content here");
        renderer.insert_child(&body, &renderer.text_to_node(&body_text), 0);

        renderer.insert_child(&container, &renderer.element_to_node(&header), 0);
        renderer.insert_child(&container, &renderer.element_to_node(&body), 1);
        renderer.insert_child(&root, &renderer.element_to_node(&container), 0);

        // Compute layout
        renderer.compute_layout(800.0, 600.0);

        // Verify container has correct size
        let container_data = container.inner.borrow();
        let container_layout = container_data.layout.expect("container layout");
        assert_eq!(container_layout.size.width, 400.0);
        assert_eq!(container_layout.size.height, 300.0);

        // Header should be at top (after padding)
        let header_data = header.inner.borrow();
        let header_layout = header_data.layout.expect("header layout");
        assert_eq!(header_layout.size.height, 60.0);
        // With 20px padding, header starts at y=20
        assert!(
            header_layout.location.y >= 20.0,
            "header y should account for padding: got {}",
            header_layout.location.y
        );

        // Body should be below header
        let body_data = body.inner.borrow();
        let body_layout = body_data.layout.expect("body layout");
        assert_eq!(body_layout.size.height, 200.0);
        assert!(
            body_layout.location.y > header_layout.location.y,
            "body should be below header"
        );

        // Text nodes should have layout
        let header_text_data = header_text.inner.borrow();
        assert!(
            header_text_data.layout.is_some(),
            "header text should have layout"
        );
    }
}
