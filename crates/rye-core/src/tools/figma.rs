//! Goal 133: Figma import (design-to-code).
//!
//! Import Figma frames as rye components. Uses the Figma REST API to fetch
//! node trees and convert them to rye template syntax.

use std::collections::HashMap;

/// Figma node types.
#[derive(Debug, Clone, PartialEq)]
pub enum FigmaNodeType {
    /// Document root.
    Document,
    /// Canvas/page.
    Canvas,
    /// Frame (container).
    Frame,
    /// Group.
    Group,
    /// Text node.
    Text,
    /// Rectangle.
    Rectangle,
    /// Ellipse.
    Ellipse,
    /// Vector.
    Vector,
    /// Component.
    Component,
    /// Instance of a component.
    Instance,
    /// Other/unknown.
    Other(String),
}

impl FigmaNodeType {
    /// Parse from Figma API string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "DOCUMENT" => Self::Document,
            "CANVAS" => Self::Canvas,
            "FRAME" => Self::Frame,
            "GROUP" => Self::Group,
            "TEXT" => Self::Text,
            "RECTANGLE" => Self::Rectangle,
            "ELLIPSE" => Self::Ellipse,
            "VECTOR" => Self::Vector,
            "COMPONENT" => Self::Component,
            "INSTANCE" => Self::Instance,
            other => Self::Other(other.to_string()),
        }
    }
}

/// A Figma node imported from the API.
#[derive(Debug, Clone)]
pub struct FigmaNode {
    /// Node ID.
    pub id: String,
    /// Node name.
    pub name: String,
    /// Node type.
    pub node_type: FigmaNodeType,
    /// Children nodes.
    pub children: Vec<FigmaNode>,
    /// Layout properties.
    pub layout: FigmaLayout,
    /// Style properties.
    pub style: FigmaStyle,
    /// Text content (for text nodes).
    pub text_content: Option<String>,
}

/// Figma layout properties.
#[derive(Debug, Clone, Default)]
pub struct FigmaLayout {
    /// Absolute position X.
    pub x: f64,
    /// Absolute position Y.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
    /// Layout mode (NONE, HORIZONTAL, VERTICAL).
    pub layout_mode: String,
    /// Padding.
    pub padding: (f64, f64, f64, f64), // top, right, bottom, left
    /// Gap between children.
    pub gap: f64,
    /// Corner radius.
    pub corner_radius: f64,
}

/// Figma style properties.
#[derive(Debug, Clone, Default)]
pub struct FigmaStyle {
    /// Background color (hex).
    pub background: Option<String>,
    /// Text color (hex).
    pub color: Option<String>,
    /// Font size.
    pub font_size: Option<f64>,
    /// Font weight.
    pub font_weight: Option<f64>,
    /// Font family.
    pub font_family: Option<String>,
    /// Border color (hex).
    pub border_color: Option<String>,
    /// Border width.
    pub border_width: Option<f64>,
    /// Opacity (0.0 to 1.0).
    pub opacity: Option<f64>,
}

/// Figma import configuration.
#[derive(Debug, Clone)]
pub struct FigmaImportConfig {
    /// Figma file key.
    pub file_key: String,
    /// Node IDs to import.
    pub node_ids: Vec<String>,
    /// Whether to flatten groups.
    pub flatten_groups: bool,
    /// Whether to extract components.
    pub extract_components: bool,
}

impl FigmaImportConfig {
    /// Create a new Figma import config.
    pub fn new(file_key: impl Into<String>) -> Self {
        Self {
            file_key: file_key.into(),
            node_ids: Vec::new(),
            flatten_groups: false,
            extract_components: true,
        }
    }

    /// Add a node ID to import.
    pub fn node(mut self, id: impl Into<String>) -> Self {
        self.node_ids.push(id.into());
        self
    }
}

/// Convert a Figma node to a rye component template string.
pub fn figma_to_rye(node: &FigmaNode) -> String {
    let mut output = String::new();
    figma_to_rye_recursive(node, &mut output, 0);
    output
}

fn figma_to_rye_recursive(node: &FigmaNode, output: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);

    match &node.node_type {
        FigmaNodeType::Text => {
            let text = node.text_content.as_deref().unwrap_or("");
            let style_str = format_figma_style(&node.style);
            output.push_str(&format!("{}<span style=\"{}\">{}</span>\n", pad, style_str, text));
        }
        FigmaNodeType::Rectangle | FigmaNodeType::Frame | FigmaNodeType::Component | FigmaNodeType::Instance => {
            let tag = if matches!(node.node_type, FigmaNodeType::Rectangle) { "div" } else { "div" };
            let style_str = format_figma_layout_style(&node.layout, &node.style);
            output.push_str(&format!("{}<{} style=\"{}\">\n", pad, tag, style_str));

            for child in &node.children {
                figma_to_rye_recursive(child, output, indent + 1);
            }

            output.push_str(&format!("{}</{}>\n", pad, tag));
        }
        FigmaNodeType::Group => {
            for child in &node.children {
                figma_to_rye_recursive(child, output, indent);
            }
        }
        FigmaNodeType::Canvas | FigmaNodeType::Document => {
            for child in &node.children {
                figma_to_rye_recursive(child, output, indent);
            }
        }
        FigmaNodeType::Ellipse => {
            let style_str = format!("{}; border-radius: 50%", format_figma_layout_style(&node.layout, &node.style));
            output.push_str(&format!("{}<div style=\"{}\"></div>\n", pad, style_str));
        }
        FigmaNodeType::Vector => {
            output.push_str(&format!("{}<svg width=\"{}\" height=\"{}\"></svg>\n", pad, node.layout.width, node.layout.height));
        }
        FigmaNodeType::Other(_) => {
            // Skip unknown node types
        }
    }
}

fn format_figma_style(style: &FigmaStyle) -> String {
    let mut parts = Vec::new();
    if let Some(color) = &style.color {
        parts.push(format!("color: {}", color));
    }
    if let Some(size) = style.font_size {
        parts.push(format!("font-size: {}px", size));
    }
    if let Some(weight) = style.font_weight {
        parts.push(format!("font-weight: {}", weight));
    }
    if let Some(family) = &style.font_family {
        parts.push(format!("font-family: {}", family));
    }
    if let Some(opacity) = style.opacity {
        if opacity < 1.0 {
            parts.push(format!("opacity: {}", opacity));
        }
    }
    parts.join("; ")
}

fn format_figma_layout_style(layout: &FigmaLayout, style: &FigmaStyle) -> String {
    let mut parts = vec![
        format!("width: {}px", layout.width),
        format!("height: {}px", layout.height),
    ];

    if layout.corner_radius > 0.0 {
        parts.push(format!("border-radius: {}px", layout.corner_radius));
    }

    if layout.layout_mode == "HORIZONTAL" {
        parts.push("display: flex".to_string());
        parts.push("flex-direction: row".to_string());
        if layout.gap > 0.0 {
            parts.push(format!("gap: {}px", layout.gap));
        }
    } else if layout.layout_mode == "VERTICAL" {
        parts.push("display: flex".to_string());
        parts.push("flex-direction: column".to_string());
        if layout.gap > 0.0 {
            parts.push(format!("gap: {}px", layout.gap));
        }
    }

    if let Some(bg) = &style.background {
        parts.push(format!("background: {}", bg));
    }
    if let Some(border) = &style.border_color {
        let width = style.border_width.unwrap_or(1.0);
        parts.push(format!("border: {}px solid {}", width, border));
    }

    parts.join("; ")
}

/// Figma API URL for fetching a node.
pub fn figma_api_url(file_key: &str, node_id: &str) -> String {
    format!("https://api.figma.com/v1/files/{}/nodes?id={}", file_key, node_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_text_node(name: &str, text: &str) -> FigmaNode {
        FigmaNode {
            id: "1".to_string(),
            name: name.to_string(),
            node_type: FigmaNodeType::Text,
            children: Vec::new(),
            layout: FigmaLayout::default(),
            style: FigmaStyle {
                color: Some("#333".to_string()),
                font_size: Some(16.0),
                ..Default::default()
            },
            text_content: Some(text.to_string()),
        }
    }

    fn make_frame_node(name: &str, children: Vec<FigmaNode>) -> FigmaNode {
        FigmaNode {
            id: "0".to_string(),
            name: name.to_string(),
            node_type: FigmaNodeType::Frame,
            children,
            layout: FigmaLayout {
                width: 400.0,
                height: 300.0,
                layout_mode: "VERTICAL".to_string(),
                corner_radius: 8.0,
                ..Default::default()
            },
            style: FigmaStyle {
                background: Some("#fff".to_string()),
                ..Default::default()
            },
            text_content: None,
        }
    }

    #[test]
    fn test_figma_node_type_from_str() {
        assert_eq!(FigmaNodeType::from_str("FRAME"), FigmaNodeType::Frame);
        assert_eq!(FigmaNodeType::from_str("TEXT"), FigmaNodeType::Text);
        assert!(matches!(FigmaNodeType::from_str("CUSTOM"), FigmaNodeType::Other(_)));
    }

    #[test]
    fn test_figma_to_rye_text() {
        let node = make_text_node("Label", "Hello World");
        let html = figma_to_rye(&node);
        assert!(html.contains("Hello World"));
        assert!(html.contains("color: #333"));
        assert!(html.contains("font-size: 16px"));
    }

    #[test]
    fn test_figma_to_rye_frame() {
        let frame = make_frame_node("Card", vec![make_text_node("Title", "Card Title")]);
        let html = figma_to_rye(&frame);
        assert!(html.contains("<div"));
        assert!(html.contains("width: 400px"));
        assert!(html.contains("height: 300px"));
        assert!(html.contains("Card Title"));
        assert!(html.contains("background: #fff"));
        assert!(html.contains("border-radius: 8px"));
    }

    #[test]
    fn test_figma_to_rye_flex() {
        let frame = FigmaNode {
            id: "0".to_string(),
            name: "Row".to_string(),
            node_type: FigmaNodeType::Frame,
            children: Vec::new(),
            layout: FigmaLayout {
                width: 100.0,
                height: 50.0,
                layout_mode: "HORIZONTAL".to_string(),
                gap: 10.0,
                ..Default::default()
            },
            style: FigmaStyle::default(),
            text_content: None,
        };
        let html = figma_to_rye(&frame);
        assert!(html.contains("display: flex"));
        assert!(html.contains("flex-direction: row"));
        assert!(html.contains("gap: 10px"));
    }

    #[test]
    fn test_figma_to_rye_ellipse() {
        let node = FigmaNode {
            id: "1".to_string(),
            name: "Circle".to_string(),
            node_type: FigmaNodeType::Ellipse,
            children: Vec::new(),
            layout: FigmaLayout {
                width: 50.0,
                height: 50.0,
                ..Default::default()
            },
            style: FigmaStyle::default(),
            text_content: None,
        };
        let html = figma_to_rye(&node);
        assert!(html.contains("border-radius: 50%"));
    }

    #[test]
    fn test_figma_api_url() {
        let url = figma_api_url("abc123", "1:2");
        assert!(url.contains("abc123"));
        assert!(url.contains("1:2"));
    }

    #[test]
    fn test_figma_import_config() {
        let config = FigmaImportConfig::new("file123")
            .node("1:2")
            .node("3:4");
        assert_eq!(config.file_key, "file123");
        assert_eq!(config.node_ids, vec!["1:2", "3:4"]);
    }
}
