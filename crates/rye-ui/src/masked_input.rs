//! MaskedInput — phone/date/SSN formatted input.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct MaskPattern {
    pub mask: String,
    pub placeholder_char: char,
}

impl MaskPattern {
    pub fn phone() -> Self {
        Self {
            mask: "(###) ###-####".to_string(),
            placeholder_char: '#',
        }
    }
    pub fn date() -> Self {
        Self {
            mask: "##/##/####".to_string(),
            placeholder_char: '#',
        }
    }
    pub fn ssn() -> Self {
        Self {
            mask: "###-##-####".to_string(),
            placeholder_char: '#',
        }
    }
    pub fn zip() -> Self {
        Self {
            mask: "#####".to_string(),
            placeholder_char: '#',
        }
    }
    pub fn custom(mask: impl Into<String>) -> Self {
        Self {
            mask: mask.into(),
            placeholder_char: '#',
        }
    }

    pub fn apply(&self, value: &str) -> String {
        let digits: Vec<char> = value.chars().filter(|c| c.is_ascii_digit()).collect();
        let mut result = String::new();
        let mut di = 0;
        for mc in self.mask.chars() {
            if mc == self.placeholder_char {
                if di < digits.len() {
                    result.push(digits[di]);
                    di += 1;
                } else {
                    break;
                }
            } else {
                if di < digits.len() || result.len() < self.mask.len() {
                    result.push(mc);
                }
            }
        }
        result
    }

    pub fn placeholder_text(&self) -> String {
        self.mask.replace(self.placeholder_char, "_")
    }
}

#[derive(Debug, Clone)]
pub struct MaskedInputProps {
    pub pattern: MaskPattern,
    pub value: String,
    pub label: Option<String>,
    pub disabled: bool,
    pub error: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for MaskedInputProps {
    fn default() -> Self {
        Self {
            pattern: MaskPattern::phone(),
            value: String::new(),
            label: None,
            disabled: false,
            error: None,
            class: None,
            style: None,
        }
    }
}

impl MaskedInputProps {
    pub fn pattern(mut self, p: MaskPattern) -> Self {
        self.pattern = p;
        self
    }
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn error(mut self, e: impl Into<String>) -> Self {
        self.error = Some(e.into());
        self
    }
}

pub struct MaskedInput;

impl MaskedInput {
    pub fn render(props: MaskedInputProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let border_color = if props.error.is_some() {
            vars::DANGER
        } else {
            vars::INPUT_BORDER
        };
        let input_style = format!(
            "width:100%;padding:8px 16px;font-size:var(--rye-font-size-md);border:1px solid {};border-radius:var(--rye-radius-md);\
             background:{};opacity:{};cursor:{};font-family:var(--rye-font-family);box-sizing:border-box;{}",
            border_color,
            if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
            if props.disabled { "0.6" } else { "1.0" },
            if props.disabled { "not-allowed" } else { "text" },
            props.style.as_deref().unwrap_or(""),
        );

        let masked_value = props.pattern.apply(&props.value);
        let placeholder = props.pattern.placeholder_text();

        let mut attrs = vec![
            ("type".to_string(), "text".to_string()),
            ("style".to_string(), input_style),
            ("placeholder".to_string(), placeholder),
            (
                "class".to_string(),
                format!("rye-masked-input {}", props.class.as_deref().unwrap_or("")),
            ),
        ];
        if !masked_value.is_empty() {
            attrs.push(("value".to_string(), masked_value));
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

        if let Some(error) = &props.error {
            children.push(Template::new_element(
                "span",
                vec![(
                    "style".to_string(),
                    format!(
                        "display:block;margin-top:4px;font-size:var(--rye-font-size-sm);color:{};",
                        vars::DANGER
                    ),
                )],
                Vec::new(),
                vec![Template::text(error)],
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![("class".to_string(), "rye-masked-input-wrapper".to_string())],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_phone_apply() {
        let p = MaskPattern::phone();
        assert_eq!(p.apply("1234567890"), "(123) 456-7890");
        assert_eq!(p.apply("123"), "(123) ");
    }

    #[test]
    fn test_mask_date_apply() {
        let p = MaskPattern::date();
        assert_eq!(p.apply("12252025"), "12/25/2025");
    }

    #[test]
    fn test_mask_ssn_apply() {
        let p = MaskPattern::ssn();
        assert_eq!(p.apply("123456789"), "123-45-6789");
    }

    #[test]
    fn test_mask_placeholder() {
        let p = MaskPattern::phone();
        assert_eq!(p.placeholder_text(), "(___) ___-____");
    }

    #[test]
    fn test_masked_input_default() {
        let p = MaskedInputProps::default();
        assert_eq!(p.pattern.mask, "(###) ###-####");
    }

    #[test]
    fn test_masked_input_builder() {
        let p = MaskedInputProps::default()
            .pattern(MaskPattern::date())
            .value("12252025")
            .label("Birth Date");
        assert_eq!(p.pattern.mask, "##/##/####");
    }

    #[test]
    fn test_masked_input_render() {
        let el = MaskedInput::render(
            MaskedInputProps::default()
                .value("5551234567")
                .label("Phone"),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
