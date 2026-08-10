//! DatePicker — calendar picker.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct DatePickerProps {
    pub value: Option<(u32, u32, u32)>, // (year, month 1-12, day)
    pub label: Option<String>,
    pub min: Option<(u32, u32, u32)>,
    pub max: Option<(u32, u32, u32)>,
    pub disabled: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DatePickerProps {
    fn default() -> Self {
        Self {
            value: None,
            label: None,
            min: None,
            max: None,
            disabled: false,
            class: None,
            style: None,
        }
    }
}

impl DatePickerProps {
    pub fn value(mut self, y: u32, m: u32, d: u32) -> Self {
        self.value = Some((y, m, d));
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn min(mut self, y: u32, m: u32, d: u32) -> Self {
        self.min = Some((y, m, d));
        self
    }
    pub fn max(mut self, y: u32, m: u32, d: u32) -> Self {
        self.max = Some((y, m, d));
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
}

fn format_date(y: u32, m: u32, d: u32) -> String {
    format!("{:04}-{:02}-{:02}", y, m, d)
}

pub struct DatePicker;

impl DatePicker {
    pub fn render(props: DatePickerProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let style = format!(
            "width:100%;padding:8px 16px;font-size:var(--rye-font-size-md);border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;{}",
            vars::INPUT_BORDER,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "pointer" },
            props.style.as_deref().unwrap_or(""),
        );

        let mut attrs = vec![
            ("type".to_string(), "date".to_string()),
            ("style".to_string(), style),
            (
                "class".to_string(),
                format!("rye-date-picker {}", props.class.as_deref().unwrap_or("")),
            ),
        ];

        if let Some((y, m, d)) = props.value {
            attrs.push(("value".to_string(), format_date(y, m, d)));
        }
        if let Some((y, m, d)) = props.min {
            attrs.push(("min".to_string(), format_date(y, m, d)));
        }
        if let Some((y, m, d)) = props.max {
            attrs.push(("max".to_string(), format_date(y, m, d)));
        }
        if props.disabled {
            attrs.push(("disabled".to_string(), "true".to_string()));
        }

        children.push(Template::new_element(
            "input",
            attrs,
            Vec::new(),
            Vec::new(),
        ));

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-date-picker-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_picker_default() {
        let p = DatePickerProps::default();
        assert!(p.value.is_none());
        assert!(!p.disabled);
    }

    #[test]
    fn test_date_picker_builder() {
        let p = DatePickerProps::default()
            .value(2025, 1, 15)
            .label("Birth Date")
            .min(1900, 1, 1)
            .max(2025, 12, 31);
        assert_eq!(p.value, Some((2025, 1, 15)));
        assert_eq!(p.min, Some((1900, 1, 1)));
    }

    #[test]
    fn test_format_date() {
        assert_eq!(format_date(2025, 1, 5), "2025-01-05");
        assert_eq!(format_date(2025, 12, 31), "2025-12-31");
    }

    #[test]
    fn test_date_picker_render() {
        let el = DatePicker::render(DatePickerProps::default().label("Date").value(2025, 6, 15));
        assert!(matches!(el, Element::Template(_)));
    }
}
