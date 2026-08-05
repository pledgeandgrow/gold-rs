//! Badge — small status indicator.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::Variant;

#[derive(Debug, Clone)]
pub struct BadgeProps {
    pub text: String,
    pub variant: Variant,
    pub dot: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self { text: String::new(), variant: Variant::Secondary, dot: false, class: None, style: None }
    }
}

impl BadgeProps {
    pub fn text(mut self, t: impl Into<String>) -> Self { self.text = t.into(); self }
    pub fn variant(mut self, v: Variant) -> Self { self.variant = v; self }
    pub fn dot(mut self, d: bool) -> Self { self.dot = d; self }
}

pub struct Badge;

impl Badge {
    pub fn render(props: BadgeProps) -> Element {
        let style = format!(
            "display:inline-flex;align-items:center;gap:4px;padding:2px 8px;font-size:12px;\
             font-weight:500;border-radius:9999px;background:{};color:{};{}",
            props.variant.background(), props.variant.color(),
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = Vec::new();
        if props.dot {
            children.push(Template::new_element("span",
                vec![("style".to_string(), "width:6px;height:6px;border-radius:50%;background:currentColor;".to_string())],
                Vec::new(), Vec::new()));
        }
        children.push(Template::text(&props.text));

        Element::Template(Template::new_element("span",
            vec![("class".to_string(), format!("rye-badge {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_default() {
        let p = BadgeProps::default();
        assert_eq!(p.variant, Variant::Secondary);
        assert!(!p.dot);
    }

    #[test]
    fn test_badge_builder() {
        let p = BadgeProps::default().text("New").variant(Variant::Success).dot(true);
        assert_eq!(p.text, "New");
        assert!(p.dot);
    }

    #[test]
    fn test_badge_render() {
        let el = Badge::render(BadgeProps::default().text("5").variant(Variant::Primary));
        assert!(matches!(el, Element::Template(_)));
    }
}
