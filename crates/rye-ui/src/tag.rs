//! Tag (Chip) — removable tag with variants.

use crate::theme::Variant;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct TagProps {
    pub text: String,
    pub variant: Variant,
    pub removable: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TagProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            variant: Variant::Secondary,
            removable: false,
            class: None,
            style: None,
        }
    }
}

impl TagProps {
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.text = t.into();
        self
    }
    pub fn variant(mut self, v: Variant) -> Self {
        self.variant = v;
        self
    }
    pub fn removable(mut self, r: bool) -> Self {
        self.removable = r;
        self
    }
}

pub struct Tag;

impl Tag {
    pub fn render(props: TagProps) -> Element {
        let style = format!(
            "display:inline-flex;align-items:center;gap:4px;padding:4px 10px;font-size:12px;\
             border-radius:9999px;background:{};color:{};{}",
            props.variant.background(),
            props.variant.color(),
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![Template::text(&props.text)];

        if props.removable {
            children.push(Template::new_element("button",
                vec![("style".to_string(), "border:none;background:none;color:inherit;cursor:pointer;font-size:14px;padding:0;line-height:1;".to_string()),
                     ("aria-label".to_string(), format!("Remove {}", props.text)),
                     ("class".to_string(), "rye-tag-remove".to_string())],
                Vec::new(), vec![Template::text("×")]));
        }

        Element::Template(Template::new_element(
            "span",
            vec![
                (
                    "class".to_string(),
                    format!("rye-tag {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), style),
            ],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_default() {
        let p = TagProps::default();
        assert_eq!(p.variant, Variant::Secondary);
        assert!(!p.removable);
    }

    #[test]
    fn test_tag_builder() {
        let p = TagProps::default()
            .text("Rust")
            .variant(Variant::Primary)
            .removable(true);
        assert_eq!(p.text, "Rust");
        assert!(p.removable);
    }

    #[test]
    fn test_tag_render() {
        let el = Tag::render(TagProps::default().text("New").removable(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
