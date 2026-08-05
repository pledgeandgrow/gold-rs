//! Textarea component — multi-line text input.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::{Size, vars};

#[derive(Debug, Clone)]
pub struct TextareaProps {
    pub placeholder: String,
    pub value: String,
    pub label: Option<String>,
    pub error: Option<String>,
    pub disabled: bool,
    pub size: Size,
    pub rows: usize,
    pub resize: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TextareaProps {
    fn default() -> Self {
        Self {
            placeholder: String::new(), value: String::new(), label: None,
            error: None, disabled: false, size: Size::Medium, rows: 4,
            resize: true, class: None, style: None,
        }
    }
}

impl TextareaProps {
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = p.into(); self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn error(mut self, e: impl Into<String>) -> Self { self.error = Some(e.into()); self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn rows(mut self, r: usize) -> Self { self.rows = r; self }
    pub fn no_resize(mut self) -> Self { self.resize = false; self }
}

pub struct Textarea;

impl Textarea {
    pub fn render(props: TextareaProps) -> Element {
        let border_color = if props.error.is_some() { vars::DANGER } else { vars::INPUT_BORDER };
        let style = format!(
            "width:100%;padding:{};font-size:{};border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;resize:{};",
            props.size.padding(), props.size.font_size(), border_color,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "text" },
            if props.resize { "vertical" } else { "none" },
        );

        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;margin-bottom:4px;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let mut attrs = vec![
            ("style".to_string(), if let Some(extra) = &props.style { format!("{}{}", style, extra) } else { style }),
            ("class".to_string(), format!("rye-textarea {}", props.class.as_deref().unwrap_or(""))),
            ("rows".to_string(), props.rows.to_string()),
        ];
        if !props.placeholder.is_empty() {
            attrs.push(("placeholder".to_string(), props.placeholder.clone()));
        }
        if props.disabled {
            attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut ta_children = Vec::new();
        if !props.value.is_empty() {
            ta_children.push(Template::text(&props.value));
        }
        children.push(Template::new_element("textarea", attrs, Vec::new(), ta_children));

        if let Some(error) = &props.error {
            children.push(Template::new_element("span",
                vec![("style".to_string(), "display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:var(--rye-danger);".to_string())],
                Vec::new(), vec![Template::text(error)]));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-textarea-wrapper".to_string())], Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_default() {
        let p = TextareaProps::default();
        assert_eq!(p.rows, 4);
        assert!(p.resize);
    }

    #[test]
    fn test_textarea_builder() {
        let p = TextareaProps::default().rows(8).no_resize().label("Bio").placeholder("Tell us about yourself");
        assert_eq!(p.rows, 8);
        assert!(!p.resize);
        assert_eq!(p.label.as_deref(), Some("Bio"));
    }

    #[test]
    fn test_textarea_render() {
        let el = Textarea::render(TextareaProps::default().value("Hello world"));
        assert!(matches!(el, Element::Template(_)));
    }
}
