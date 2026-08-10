//! Avatar — image with fallback initials, sizes.

use crate::theme::{vars, Size};
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct AvatarProps {
    pub src: Option<String>,
    pub name: String,
    pub size: Size,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for AvatarProps {
    fn default() -> Self {
        Self {
            src: None,
            name: String::new(),
            size: Size::Medium,
            class: None,
            style: None,
        }
    }
}

impl AvatarProps {
    pub fn src(mut self, s: impl Into<String>) -> Self {
        self.src = Some(s.into());
        self
    }
    pub fn name(mut self, n: impl Into<String>) -> Self {
        self.name = n.into();
        self
    }
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
}

pub struct Avatar;

impl Avatar {
    pub fn render(props: AvatarProps) -> Element {
        let dim = match props.size {
            Size::Small => "24px",
            Size::Medium => "40px",
            Size::Large => "56px",
        };
        let font_size = match props.size {
            Size::Small => "10px",
            Size::Medium => "16px",
            Size::Large => "22px",
        };

        let style = format!(
            "width:{};height:{};border-radius:50%;display:inline-flex;align-items:center;\
             justify-content:center;background:{};color:{};font-size:{};font-weight:var(--rye-font-weight-semibold);\
             overflow:hidden;flex-shrink:0;{}",
            dim, dim, vars::BORDER, vars::TEXT_MUTED, font_size, props.style.as_deref().unwrap_or(""),
        );

        let children = if let Some(src) = &props.src {
            vec![Template::new_element(
                "img",
                vec![
                    ("src".to_string(), src.clone()),
                    ("alt".to_string(), props.name.clone()),
                    (
                        "style".to_string(),
                        "width:100%;height:100%;object-fit:cover;".to_string(),
                    ),
                ],
                Vec::new(),
                Vec::new(),
            )]
        } else {
            let initials = get_initials(&props.name);
            vec![Template::text(&initials)]
        };

        Element::Template(Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!("rye-avatar {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
            ],
            Vec::new(),
            children,
        ))
    }
}

fn get_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    if parts.is_empty() {
        return "?".to_string();
    }
    let first = parts.first().and_then(|p| p.chars().next()).unwrap_or('?');
    if parts.len() == 1 {
        return first.to_uppercase().to_string();
    }
    let last = parts.last().and_then(|p| p.chars().next()).unwrap_or('?');
    format!("{}{}", first.to_uppercase(), last.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avatar_default() {
        let p = AvatarProps::default();
        assert!(p.src.is_none());
        assert_eq!(p.size, Size::Medium);
    }

    #[test]
    fn test_avatar_builder() {
        let p = AvatarProps::default()
            .src("https://example.com/a.png")
            .name("Alice")
            .size(Size::Large);
        assert_eq!(p.src.as_deref(), Some("https://example.com/a.png"));
        assert_eq!(p.name, "Alice");
    }

    #[test]
    fn test_avatar_render_with_image() {
        let el = Avatar::render(AvatarProps::default().src("/img.png").name("Bob"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_avatar_render_with_initials() {
        let el = Avatar::render(AvatarProps::default().name("Alice Smith"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_get_initials() {
        assert_eq!(get_initials("Alice Smith"), "AS");
        assert_eq!(get_initials("Bob"), "B");
        assert_eq!(get_initials(""), "?");
        assert_eq!(get_initials("John Doe Smith"), "JS");
    }
}
