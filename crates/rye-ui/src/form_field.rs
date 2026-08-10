//! FormField — wrapper combining Label + Input + error message.

use crate::theme::vars;
use crate::theme::Size;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct FormFieldProps {
    pub label: String,
    pub required: bool,
    pub error: Option<String>,
    pub hint: Option<String>,
    pub field_type: FormFieldType,
    pub placeholder: String,
    pub value: String,
    pub disabled: bool,
    pub size: Size,
    pub class: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldType {
    Text,
    Email,
    Password,
    Number,
    Textarea,
    Select,
}

impl Default for FormFieldProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            required: false,
            error: None,
            hint: None,
            field_type: FormFieldType::Text,
            placeholder: String::new(),
            value: String::new(),
            disabled: false,
            size: Size::Medium,
            class: None,
            style: None,
        }
    }
}

impl FormFieldProps {
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    pub fn error(mut self, e: impl Into<String>) -> Self {
        self.error = Some(e.into());
        self
    }
    pub fn hint(mut self, h: impl Into<String>) -> Self {
        self.hint = Some(h.into());
        self
    }
    pub fn field_type(mut self, t: FormFieldType) -> Self {
        self.field_type = t;
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

pub struct FormField;

impl FormField {
    pub fn render(props: FormFieldProps) -> Element {
        let mut children = Vec::new();

        // Label
        if !props.label.is_empty() {
            let label_children = {
                let mut v = vec![Template::text(&props.label)];
                if props.required {
                    v.push(Template::new_element(
                        "span",
                        vec![(
                            "style".to_string(),
                            format!("color:{};margin-left:2px;", vars::DANGER),
                        )],
                        Vec::new(),
                        vec![Template::text("*")],
                    ));
                }
                v
            };
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string()),
                     ("class".to_string(), "rye-form-field-label".to_string())],
                Vec::new(), label_children));
        }

        // Field
        let border_color = if props.error.is_some() {
            vars::DANGER
        } else {
            vars::INPUT_BORDER
        };
        let field_style = format!(
            "width:100%;padding:{};font-size:{};border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;",
            props.size.padding(), props.size.font_size(), border_color,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "text" },
        );

        let (tag, type_attr) = match props.field_type {
            FormFieldType::Textarea => ("textarea", None),
            FormFieldType::Text => ("input", Some("text")),
            FormFieldType::Email => ("input", Some("email")),
            FormFieldType::Password => ("input", Some("password")),
            FormFieldType::Number => ("input", Some("number")),
            FormFieldType::Select => ("select", None),
        };

        let mut field_attrs = vec![
            (
                "style".to_string(),
                if let Some(s) = &props.style {
                    format!("{}{}", field_style, s)
                } else {
                    field_style
                },
            ),
            (
                "class".to_string(),
                format!(
                    "rye-form-field-input {}",
                    props.class.as_deref().unwrap_or("")
                ),
            ),
        ];
        if let Some(t) = type_attr {
            field_attrs.push(("type".to_string(), t.to_string()));
        }
        if !props.placeholder.is_empty() {
            field_attrs.push(("placeholder".to_string(), props.placeholder.clone()));
        }
        if props.disabled {
            field_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let field_children = if tag == "textarea" || tag == "select" {
            if !props.value.is_empty() {
                vec![Template::text(&props.value)]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        if tag == "input" && !props.value.is_empty() {
            field_attrs.push(("value".to_string(), props.value.clone()));
        }

        children.push(Template::new_element(
            tag,
            field_attrs,
            Vec::new(),
            field_children,
        ));

        // Error or hint
        if let Some(error) = &props.error {
            children.push(Template::new_element("span",
                vec![("style".to_string(), format!("display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:{};", vars::DANGER)),
                     ("class".to_string(), "rye-form-field-error".to_string())],
                Vec::new(), vec![Template::text(error)]));
        } else if let Some(hint) = &props.hint {
            children.push(Template::new_element("span",
                vec![("style".to_string(), format!("display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:{};", vars::TEXT_MUTED)),
                     ("class".to_string(), "rye-form-field-hint".to_string())],
                Vec::new(), vec![Template::text(hint)]));
        }

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-form-field".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_default() {
        let p = FormFieldProps::default();
        assert_eq!(p.field_type, FormFieldType::Text);
        assert!(!p.required);
    }

    #[test]
    fn test_form_field_builder() {
        let p = FormFieldProps::default()
            .label("Email")
            .required(true)
            .field_type(FormFieldType::Email)
            .placeholder("you@example.com")
            .error("Invalid email");
        assert_eq!(p.label, "Email");
        assert!(p.required);
        assert_eq!(p.field_type, FormFieldType::Email);
        assert_eq!(p.error.as_deref(), Some("Invalid email"));
    }

    #[test]
    fn test_form_field_render_text() {
        let el = FormField::render(
            FormFieldProps::default()
                .label("Name")
                .placeholder("Enter name"),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_form_field_render_textarea() {
        let el = FormField::render(
            FormFieldProps::default()
                .label("Bio")
                .field_type(FormFieldType::Textarea),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_form_field_render_select() {
        let el = FormField::render(
            FormFieldProps::default()
                .label("Country")
                .field_type(FormFieldType::Select),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
