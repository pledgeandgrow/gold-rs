//! Label component — form label with required indicator.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct LabelProps {
    pub text: String,
    pub required: bool,
    pub disabled: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for LabelProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            required: false,
            disabled: false,
            class: None,
            style: None,
        }
    }
}

impl LabelProps {
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.text = t.into();
        self
    }
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

pub struct Label;

impl Label {
    pub fn render(props: LabelProps) -> Element {
        let style = format!(
            "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);color:{};margin-bottom:4px;{}",
            if props.disabled { vars::TEXT_SUBTLE } else { vars::TEXT },
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![Template::text(&props.text)];
        if props.required {
            children.push(Template::new_element(
                "span",
                vec![
                    (
                        "style".to_string(),
                        format!("color:{};margin-left:2px;", vars::DANGER),
                    ),
                    ("aria-hidden".to_string(), "true".to_string()),
                ],
                Vec::new(),
                vec![Template::text("*")],
            ));
        }

        Element::Template(Template::new_element(
            "label",
            vec![
                ("style".to_string(), style),
                (
                    "class".to_string(),
                    format!("rye-label {}", props.class.as_deref().unwrap_or("")),
                ),
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
    fn test_label_default() {
        let p = LabelProps::default();
        assert!(!p.required);
    }

    #[test]
    fn test_label_builder() {
        let p = LabelProps::default().text("Email").required(true);
        assert_eq!(p.text, "Email");
        assert!(p.required);
    }

    #[test]
    fn test_label_render() {
        let el = Label::render(LabelProps::default().text("Name").required(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
