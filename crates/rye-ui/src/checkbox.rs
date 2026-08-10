//! Checkbox component.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct CheckboxProps {
    pub label: Option<String>,
    pub checked: bool,
    pub indeterminate: bool,
    pub disabled: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for CheckboxProps {
    fn default() -> Self {
        Self {
            label: None,
            checked: false,
            indeterminate: false,
            disabled: false,
            class: None,
            style: None,
        }
    }
}

impl CheckboxProps {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn checked(mut self, c: bool) -> Self {
        self.checked = c;
        self
    }
    pub fn indeterminate(mut self, i: bool) -> Self {
        self.indeterminate = i;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

pub struct Checkbox;

impl Checkbox {
    pub fn render(props: CheckboxProps) -> Element {
        let label_style =
            format!(
            "display:inline-flex;align-items:center;gap:8px;cursor:{};opacity:{};font-size:14px;",
            if props.disabled { "not-allowed" } else { "pointer" },
            if props.disabled { "0.6" } else { "1.0" },
        );

        let mut input_attrs = vec![
            ("type".to_string(), "checkbox".to_string()),
            (
                "style".to_string(),
                format!("width:16px;height:16px;accent-color:{};", vars::PRIMARY),
            ),
            (
                "class".to_string(),
                format!("rye-checkbox {}", props.class.as_deref().unwrap_or("")),
            ),
        ];
        if props.checked {
            input_attrs.push(("checked".to_string(), "true".to_string()));
        }
        if props.disabled {
            input_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut children = vec![Template::new_element(
            "input",
            input_attrs,
            Vec::new(),
            Vec::new(),
        )];

        if let Some(label) = &props.label {
            children.push(Template::text(label));
        }

        let style = if let Some(extra) = &props.style {
            format!("{}{}", label_style, extra)
        } else {
            label_style
        };
        Element::Template(Template::new_element(
            "label",
            vec![("style".to_string(), style)],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_default() {
        let p = CheckboxProps::default();
        assert!(!p.checked);
        assert!(!p.indeterminate);
    }

    #[test]
    fn test_checkbox_builder() {
        let p = CheckboxProps::default()
            .label("Accept terms")
            .checked(true)
            .disabled(true);
        assert_eq!(p.label.as_deref(), Some("Accept terms"));
        assert!(p.checked);
        assert!(p.disabled);
    }

    #[test]
    fn test_checkbox_render() {
        let el = Checkbox::render(CheckboxProps::default().label("Remember me").checked(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
