//! Card — container with header, body, footer.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct CardProps {
    pub shadow: bool,
    pub border: bool,
    pub padding: String,
    pub border_radius: String,
    pub background: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CardProps {
    fn default() -> Self {
        Self {
            shadow: true,
            border: true,
            padding: "16px".to_string(),
            border_radius: "var(--rye-radius-lg)".to_string(),
            background: vars::BG.to_string(),
            class: None,
            style: None,
        }
    }
}

impl CardProps {
    pub fn shadow(mut self, s: bool) -> Self {
        self.shadow = s;
        self
    }
    pub fn border(mut self, b: bool) -> Self {
        self.border = b;
        self
    }
    pub fn padding(mut self, p: impl Into<String>) -> Self {
        self.padding = p.into();
        self
    }
    pub fn border_radius(mut self, r: impl Into<String>) -> Self {
        self.border_radius = r.into();
        self
    }
    pub fn background(mut self, b: impl Into<String>) -> Self {
        self.background = b.into();
        self
    }
}

pub struct Card;

impl Card {
    pub fn render(props: CardProps) -> Element {
        let mut parts = vec![
            format!("padding:{}", props.padding),
            format!("border-radius:{}", props.border_radius),
            format!("background:{}", props.background),
        ];
        if props.border {
            parts.push(format!("border:1px solid {}", vars::BORDER));
        }
        if props.shadow {
            parts.push(format!("box-shadow:{}", vars::SHADOW_MD));
        }
        if let Some(s) = &props.style {
            parts.push(s.clone());
        }
        let style = parts.join(";");

        Element::Template(Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!("rye-card {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
            ],
            Vec::new(),
            Vec::new(),
        ))
    }
}

pub struct CardHeader;

impl CardHeader {
    pub fn render(title: &str, subtitle: Option<&str>) -> Element {
        let mut children = vec![Template::new_element("div",
            vec![("style".to_string(), format!("font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);color:{};", vars::TEXT))],
            Vec::new(), vec![Template::text(title)])];
        if let Some(sub) = subtitle {
            children.push(Template::new_element(
                "div",
                vec![(
                    "style".to_string(),
                    format!(
                        "font-size:var(--rye-font-size-md);color:{};margin-top:4px;",
                        vars::TEXT_MUTED
                    ),
                )],
                Vec::new(),
                vec![Template::text(sub)],
            ));
        }
        Element::Template(Template::new_element(
            "div",
            vec![
                ("class".to_string(), "rye-card-header".to_string()),
                ("style".to_string(), "margin-bottom:12px;".to_string()),
            ],
            Vec::new(),
            children,
        ))
    }
}

pub struct CardBody;

impl CardBody {
    pub fn render() -> Element {
        Element::Template(Template::new_element(
            "div",
            vec![
                ("class".to_string(), "rye-card-body".to_string()),
                (
                    "style".to_string(),
                    format!("font-size:var(--rye-font-size-md);color:{};", vars::TEXT),
                ),
            ],
            Vec::new(),
            Vec::new(),
        ))
    }
}

pub struct CardFooter;

impl CardFooter {
    pub fn render() -> Element {
        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-card-footer".to_string()),
                 ("style".to_string(), format!("margin-top:12px;padding-top:12px;border-top:1px solid {};display:flex;justify-content:flex-end;gap:8px;", vars::BORDER))],
            Vec::new(), Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_default() {
        let p = CardProps::default();
        assert!(p.shadow);
        assert!(p.border);
    }

    #[test]
    fn test_card_builder() {
        let p = CardProps::default()
            .shadow(false)
            .padding("24px")
            .background("#f8fafc");
        assert!(!p.shadow);
        assert_eq!(p.padding, "24px");
    }

    #[test]
    fn test_card_render() {
        let el = Card::render(CardProps::default());
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_card_header_render() {
        let el = CardHeader::render("Title", Some("Subtitle"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_card_body_render() {
        let el = CardBody::render();
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_card_footer_render() {
        let el = CardFooter::render();
        assert!(matches!(el, Element::Template(_)));
    }
}
