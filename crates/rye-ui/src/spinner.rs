//! Spinner — animated loading indicator.

use crate::theme::vars;
use crate::theme::Size;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct SpinnerProps {
    pub size: Size,
    pub color: String,
    pub label: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            size: Size::Medium,
            color: vars::PRIMARY.to_string(),
            label: None,
            class: None,
            style: None,
        }
    }
}

impl SpinnerProps {
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: impl Into<String>) -> Self {
        self.color = c.into();
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
}

pub struct Spinner;

impl Spinner {
    pub fn render(props: SpinnerProps) -> Element {
        let dim = match props.size {
            Size::Small => "16px",
            Size::Medium => "24px",
            Size::Large => "36px",
        };
        let border_width = match props.size {
            Size::Small => "2px",
            Size::Medium => "3px",
            Size::Large => "4px",
        };
        let spinner_style = format!(
            "width:{};height:{};border:{} solid {};border-top-color:{};border-radius:50%;\
             animation:rye-spin 0.8s linear infinite;{}",
            dim,
            dim,
            border_width,
            vars::BORDER,
            props.color,
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!("rye-spinner {}", props.class.as_deref().unwrap_or("")),
                ),
                ("style".to_string(), spinner_style),
            ],
            Vec::new(),
            Vec::new(),
        )];

        if let Some(label) = &props.label {
            children.push(Template::new_element(
                "span",
                vec![(
                    "style".to_string(),
                    format!(
                        "font-size:var(--rye-font-size-md);color:{};",
                        vars::TEXT_MUTED
                    ),
                )],
                Vec::new(),
                vec![Template::text(label)],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![(
                "style".to_string(),
                "display:inline-flex;align-items:center;gap:8px;".to_string(),
            )],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_default() {
        let p = SpinnerProps::default();
        assert_eq!(p.size, Size::Medium);
        assert_eq!(p.color, vars::PRIMARY);
    }

    #[test]
    fn test_spinner_builder() {
        let p = SpinnerProps::default()
            .size(Size::Large)
            .color("#dc2626")
            .label("Loading...");
        assert_eq!(p.size, Size::Large);
        assert_eq!(p.label.as_deref(), Some("Loading..."));
    }

    #[test]
    fn test_spinner_render() {
        let el = Spinner::render(SpinnerProps::default());
        assert!(matches!(el, Element::Template(_)));
    }
}
