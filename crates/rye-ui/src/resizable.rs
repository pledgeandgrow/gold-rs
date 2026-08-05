//! Resizable — user-resizable panel.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeDirection {
    Horizontal,
    Vertical,
    Both,
}

impl ResizeDirection {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Horizontal => "horizontal", Self::Vertical => "vertical", Self::Both => "both" }
    }
}

#[derive(Debug, Clone)]
pub struct ResizableProps {
    pub direction: ResizeDirection,
    pub width: String,
    pub height: String,
    pub min_width: String,
    pub min_height: String,
    pub max_width: String,
    pub max_height: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ResizableProps {
    fn default() -> Self {
        Self { direction: ResizeDirection::Both, width: "300px".to_string(), height: "200px".to_string(),
               min_width: "100px".to_string(), min_height: "80px".to_string(),
               max_width: "800px".to_string(), max_height: "600px".to_string(),
               class: None, style: None }
    }
}

impl ResizableProps {
    pub fn direction(mut self, d: ResizeDirection) -> Self { self.direction = d; self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = w.into(); self }
    pub fn height(mut self, h: impl Into<String>) -> Self { self.height = h.into(); self }
    pub fn min_width(mut self, w: impl Into<String>) -> Self { self.min_width = w.into(); self }
    pub fn min_height(mut self, h: impl Into<String>) -> Self { self.min_height = h.into(); self }
}

pub struct Resizable;

impl Resizable {
    pub fn render(props: ResizableProps) -> Element {
        let resize_val = match props.direction {
            ResizeDirection::Horizontal => "horizontal",
            ResizeDirection::Vertical => "vertical",
            ResizeDirection::Both => "both",
        };

        let style = format!(
            "width:{};height:{};min-width:{};min-height:{};max-width:{};max-height:{};\
             resize:{};overflow:auto;border:1px solid {};border-radius:var(--rye-radius-md);padding:16px;\
             background:{};{}",
            props.width, props.height, props.min_width, props.min_height,
            props.max_width, props.max_height, resize_val,
            vars::BORDER, vars::BG,
            props.style.as_deref().unwrap_or(""),
        );

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-resizable rye-resizable-{} {}", props.direction.as_str(), props.class.as_deref().unwrap_or("")))],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_direction_as_str() {
        assert_eq!(ResizeDirection::Horizontal.as_str(), "horizontal");
        assert_eq!(ResizeDirection::Vertical.as_str(), "vertical");
    }

    #[test]
    fn test_resizable_default() {
        let p = ResizableProps::default();
        assert_eq!(p.direction, ResizeDirection::Both);
        assert_eq!(p.width, "300px");
    }

    #[test]
    fn test_resizable_builder() {
        let p = ResizableProps::default().direction(ResizeDirection::Horizontal).width("500px").min_width("200px");
        assert_eq!(p.direction, ResizeDirection::Horizontal);
        assert_eq!(p.width, "500px");
    }

    #[test]
    fn test_resizable_render() {
        let el = Resizable::render(ResizableProps::default().direction(ResizeDirection::Vertical));
        assert!(matches!(el, Element::Template(_)));
    }
}
