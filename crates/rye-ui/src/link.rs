//! Link — styled anchor with variants.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct LinkProps {
    pub href: String,
    pub text: String,
    pub variant: LinkVariant,
    pub disabled: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkVariant {
    Primary,
    Secondary,
    Muted,
}

impl LinkVariant {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Primary => vars::PRIMARY,
            Self::Secondary => vars::TEXT,
            Self::Muted => vars::TEXT_MUTED,
        }
    }
}

impl Default for LinkProps {
    fn default() -> Self {
        Self {
            href: String::new(),
            text: String::new(),
            variant: LinkVariant::Primary,
            disabled: false,
            class: None,
            style: None,
        }
    }
}

impl LinkProps {
    pub fn href(mut self, h: impl Into<String>) -> Self {
        self.href = h.into();
        self
    }
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.text = t.into();
        self
    }
    pub fn variant(mut self, v: LinkVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

pub struct Link;

impl Link {
    pub fn render(props: LinkProps) -> Element {
        let style = format!(
            "color:{};text-decoration:{};cursor:{};font-size:var(--rye-font-size-md);{}",
            if props.disabled {
                vars::TEXT_SUBTLE
            } else {
                props.variant.color()
            },
            if props.disabled { "none" } else { "underline" },
            if props.disabled {
                "not-allowed"
            } else {
                "pointer"
            },
            props.style.as_deref().unwrap_or(""),
        );

        let mut attrs = vec![
            ("style".to_string(), style),
            (
                "class".to_string(),
                format!("rye-link {}", props.class.as_deref().unwrap_or("")),
            ),
        ];
        if !props.href.is_empty() {
            attrs.push(("href".to_string(), props.href.clone()));
        }
        if props.disabled {
            attrs.push(("aria-disabled".to_string(), "true".to_string()));
        }

        Element::Template(Template::new_element(
            "a",
            attrs,
            Vec::new(),
            vec![Template::text(&props.text)],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_variant_color() {
        assert_eq!(LinkVariant::Primary.color(), vars::PRIMARY);
        assert_eq!(LinkVariant::Muted.color(), vars::TEXT_MUTED);
    }

    #[test]
    fn test_link_default() {
        let p = LinkProps::default();
        assert_eq!(p.variant, LinkVariant::Primary);
        assert!(!p.disabled);
    }

    #[test]
    fn test_link_builder() {
        let p = LinkProps::default()
            .href("/about")
            .text("About Us")
            .variant(LinkVariant::Muted)
            .disabled(true);
        assert_eq!(p.href, "/about");
        assert!(p.disabled);
    }

    #[test]
    fn test_link_render() {
        let el = Link::render(LinkProps::default().href("/").text("Home"));
        assert!(matches!(el, Element::Template(_)));
    }
}
