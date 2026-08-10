//! ScrollArea — custom styled scrollbar.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct ScrollAreaProps {
    pub height: String,
    pub width: Option<String>,
    pub scrollbar_color: String,
    pub scrollbar_width: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ScrollAreaProps {
    fn default() -> Self {
        Self {
            height: "300px".to_string(),
            width: None,
            scrollbar_color: vars::TEXT_SUBTLE.to_string(),
            scrollbar_width: "8px".to_string(),
            class: None,
            style: None,
        }
    }
}

impl ScrollAreaProps {
    pub fn height(mut self, h: impl Into<String>) -> Self {
        self.height = h.into();
        self
    }
    pub fn width(mut self, w: impl Into<String>) -> Self {
        self.width = Some(w.into());
        self
    }
    pub fn scrollbar_color(mut self, c: impl Into<String>) -> Self {
        self.scrollbar_color = c.into();
        self
    }
    pub fn scrollbar_width(mut self, w: impl Into<String>) -> Self {
        self.scrollbar_width = w.into();
        self
    }
}

pub struct ScrollArea;

impl ScrollArea {
    pub fn render(props: ScrollAreaProps) -> Element {
        let mut size_parts = vec![format!("height:{}", props.height)];
        if let Some(w) = &props.width {
            size_parts.push(format!("width:{}", w));
        }

        let style = format!(
            "{};overflow-y:auto;overflow-x:hidden;padding-right:8px;\
             scrollbar-width:thin;scrollbar-color:{} transparent;\
             {}",
            size_parts.join(";"),
            props.scrollbar_color,
            props.style.as_deref().unwrap_or(""),
        );

        // Webkit scrollbar styles via inline style won't work for pseudo-elements,
        // so we add a data attribute for CSS targeting
        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!("rye-scroll-area {}", props.class.as_deref().unwrap_or("")),
                ),
                (
                    "data-scrollbar-color".to_string(),
                    props.scrollbar_color.clone(),
                ),
                (
                    "data-scrollbar-width".to_string(),
                    props.scrollbar_width.clone(),
                ),
            ],
            Vec::new(),
            Vec::new(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_area_default() {
        let p = ScrollAreaProps::default();
        assert_eq!(p.height, "300px");
        assert!(p.width.is_none());
    }

    #[test]
    fn test_scroll_area_builder() {
        let p = ScrollAreaProps::default()
            .height("500px")
            .width("400px")
            .scrollbar_color("#2563eb");
        assert_eq!(p.height, "500px");
        assert_eq!(p.width.as_deref(), Some("400px"));
    }

    #[test]
    fn test_scroll_area_render() {
        let el = ScrollArea::render(ScrollAreaProps::default().height("200px"));
        assert!(matches!(el, Element::Template(_)));
    }
}
