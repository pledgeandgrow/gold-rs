//! Divider — horizontal or vertical separator.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DividerOrientation { Horizontal, Vertical }

#[derive(Debug, Clone)]
pub struct DividerProps {
    pub orientation: DividerOrientation,
    pub color: String,
    pub thickness: String,
    pub width: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DividerProps {
    fn default() -> Self {
        Self { orientation: DividerOrientation::Horizontal, color: vars::BORDER.to_string(),
               thickness: "1px".to_string(), width: None, class: None, style: None }
    }
}

impl DividerProps {
    pub fn orientation(mut self, o: DividerOrientation) -> Self { self.orientation = o; self }
    pub fn color(mut self, c: impl Into<String>) -> Self { self.color = c.into(); self }
    pub fn thickness(mut self, t: impl Into<String>) -> Self { self.thickness = t.into(); self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = Some(w.into()); self }
}

pub struct Divider;

impl Divider {
    pub fn render(props: DividerProps) -> Element {
        let style = match props.orientation {
            DividerOrientation::Horizontal => {
                let mut s = format!("width:100%;border-top:{} solid {};margin:8px 0;",
                    props.thickness, props.color);
                if let Some(w) = &props.width { s = format!("width:{};border-top:{} solid {};margin:8px 0;", w, props.thickness, props.color); }
                if let Some(extra) = &props.style { s.push_str(extra); }
                s
            }
            DividerOrientation::Vertical => {
                let mut s = format!("height:100%;border-left:{} solid {};margin:0 8px;display:inline-block;",
                    props.thickness, props.color);
                if let Some(h) = &props.width { s = format!("height:{};border-left:{} solid {};margin:0 8px;display:inline-block;", h, props.thickness, props.color); }
                if let Some(extra) = &props.style { s.push_str(extra); }
                s
            }
        };

        let tag = match props.orientation {
            DividerOrientation::Horizontal => "hr",
            DividerOrientation::Vertical => "span",
        };

        Element::Template(Template::new_element(tag,
            vec![("class".to_string(), format!("rye-divider {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divider_default() {
        let p = DividerProps::default();
        assert_eq!(p.orientation, DividerOrientation::Horizontal);
        assert_eq!(p.color, vars::BORDER);
    }

    #[test]
    fn test_divider_builder() {
        let p = DividerProps::default().orientation(DividerOrientation::Vertical).color("#cbd5e1").thickness("2px");
        assert_eq!(p.orientation, DividerOrientation::Vertical);
        assert_eq!(p.thickness, "2px");
    }

    #[test]
    fn test_divider_render_horizontal() {
        let el = Divider::render(DividerProps::default());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_divider_render_vertical() {
        let el = Divider::render(DividerProps::default().orientation(DividerOrientation::Vertical));
        assert!(matches!(el, Element::Template(_)));
    }
}
