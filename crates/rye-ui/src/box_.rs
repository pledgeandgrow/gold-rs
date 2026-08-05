//! Box — generic container with padding, border, background.

use rye_core::Element;
use rye_core::template::Template;

#[derive(Debug, Clone)]
pub struct BoxProps {
    pub padding: Option<String>,
    pub margin: Option<String>,
    pub background: Option<String>,
    pub border: Option<String>,
    pub border_radius: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for BoxProps {
    fn default() -> Self {
        Self { padding: None, margin: None, background: None, border: None,
               border_radius: None, width: None, height: None, class: None, style: None }
    }
}

impl BoxProps {
    pub fn padding(mut self, p: impl Into<String>) -> Self { self.padding = Some(p.into()); self }
    pub fn margin(mut self, m: impl Into<String>) -> Self { self.margin = Some(m.into()); self }
    pub fn background(mut self, b: impl Into<String>) -> Self { self.background = Some(b.into()); self }
    pub fn border(mut self, b: impl Into<String>) -> Self { self.border = Some(b.into()); self }
    pub fn border_radius(mut self, r: impl Into<String>) -> Self { self.border_radius = Some(r.into()); self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = Some(w.into()); self }
    pub fn height(mut self, h: impl Into<String>) -> Self { self.height = Some(h.into()); self }
}

pub struct Box;

impl Box {
    pub fn render(props: BoxProps) -> Element {
        let mut parts = Vec::new();
        if let Some(p) = &props.padding { parts.push(format!("padding:{}", p)); }
        if let Some(m) = &props.margin { parts.push(format!("margin:{}", m)); }
        if let Some(b) = &props.background { parts.push(format!("background:{}", b)); }
        if let Some(b) = &props.border { parts.push(format!("border:{}", b)); }
        if let Some(r) = &props.border_radius { parts.push(format!("border-radius:{}", r)); }
        if let Some(w) = &props.width { parts.push(format!("width:{}", w)); }
        if let Some(h) = &props.height { parts.push(format!("height:{}", h)); }
        if let Some(s) = &props.style { parts.push(s.clone()); }
        let style = parts.join(";");

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-box {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_default() {
        let p = BoxProps::default();
        assert!(p.padding.is_none());
    }

    #[test]
    fn test_box_builder() {
        let p = BoxProps::default().padding("16px").background("#f8fafc").border_radius("8px");
        assert_eq!(p.padding.as_deref(), Some("16px"));
        assert_eq!(p.background.as_deref(), Some("#f8fafc"));
    }

    #[test]
    fn test_box_render() {
        let el = Box::render(BoxProps::default().padding("12px"));
        assert!(matches!(el, Element::Template(_)));
    }
}
