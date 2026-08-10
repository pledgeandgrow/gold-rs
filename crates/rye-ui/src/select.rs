//! Select component — dropdown with options.

use crate::theme::{vars, Size};
use rye_core::template::Template;
use rye_core::Element;

/// A select option.
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SelectProps {
    pub options: Vec<SelectOption>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub label: Option<String>,
    pub error: Option<String>,
    pub disabled: bool,
    pub size: Size,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            value: None,
            placeholder: None,
            label: None,
            error: None,
            disabled: false,
            size: Size::Medium,
            class: None,
            style: None,
        }
    }
}

impl SelectProps {
    pub fn options(mut self, opts: Vec<SelectOption>) -> Self {
        self.options = opts;
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = Some(v.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn error(mut self, e: impl Into<String>) -> Self {
        self.error = Some(e.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

pub struct Select;

impl Select {
    pub fn render(props: SelectProps) -> Element {
        let border_color = if props.error.is_some() {
            vars::DANGER
        } else {
            vars::INPUT_BORDER
        };
        let style = format!(
            "width:100%;padding:{};font-size:{};border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;",
            props.size.padding(), props.size.font_size(), border_color,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "pointer" },
        );

        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;margin-bottom:4px;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let mut select_attrs = vec![
            (
                "style".to_string(),
                if let Some(extra) = &props.style {
                    format!("{}{}", style, extra)
                } else {
                    style
                },
            ),
            (
                "class".to_string(),
                format!("rye-select {}", props.class.as_deref().unwrap_or("")),
            ),
        ];
        if props.disabled {
            select_attrs.push(("disabled".to_string(), "true".to_string()));
        }

        let mut opt_children = Vec::new();
        if let Some(ph) = &props.placeholder {
            opt_children.push(Template::new_element(
                "option",
                vec![
                    ("value".to_string(), "".to_string()),
                    ("disabled".to_string(), "true".to_string()),
                    ("selected".to_string(), "true".to_string()),
                ],
                Vec::new(),
                vec![Template::text(ph)],
            ));
        }
        for opt in &props.options {
            let mut o_attrs = vec![("value".to_string(), opt.value.clone())];
            if Some(&opt.value) == props.value.as_ref() {
                o_attrs.push(("selected".to_string(), "true".to_string()));
            }
            if opt.disabled {
                o_attrs.push(("disabled".to_string(), "true".to_string()));
            }
            opt_children.push(Template::new_element(
                "option",
                o_attrs,
                Vec::new(),
                vec![Template::text(&opt.label)],
            ));
        }
        children.push(Template::new_element(
            "select",
            select_attrs,
            Vec::new(),
            opt_children,
        ));

        if let Some(error) = &props.error {
            children.push(Template::new_element("span",
                vec![("style".to_string(), "display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:var(--rye-danger);".to_string())],
                Vec::new(), vec![Template::text(error)]));
        }

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-select-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_option_new() {
        let opt = SelectOption::new("us", "United States");
        assert_eq!(opt.value, "us");
        assert_eq!(opt.label, "United States");
        assert!(!opt.disabled);
    }

    #[test]
    fn test_select_option_disabled() {
        let opt = SelectOption::new("xx", "Disabled").disabled();
        assert!(opt.disabled);
    }

    #[test]
    fn test_select_props_builder() {
        let props = SelectProps::default()
            .options(vec![
                SelectOption::new("a", "Alpha"),
                SelectOption::new("b", "Beta"),
            ])
            .value("a")
            .placeholder("Choose...")
            .label("Letter");
        assert_eq!(props.options.len(), 2);
        assert_eq!(props.value.as_deref(), Some("a"));
    }

    #[test]
    fn test_select_render() {
        let el = Select::render(
            SelectProps::default()
                .options(vec![SelectOption::new("1", "One")])
                .placeholder("Pick"),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
