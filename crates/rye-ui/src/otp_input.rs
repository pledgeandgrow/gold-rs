//! OTPInput — one-time password digit inputs.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct OtpInputProps {
    pub length: usize,
    pub value: String,
    pub label: Option<String>,
    pub disabled: bool,
    pub error: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for OtpInputProps {
    fn default() -> Self {
        Self { length: 6, value: String::new(), label: None, disabled: false,
               error: None, class: None, style: None }
    }
}

impl OtpInputProps {
    pub fn length(mut self, l: usize) -> Self { self.length = l; self }
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = Some(l.into()); self }
    pub fn disabled(mut self, d: bool) -> Self { self.disabled = d; self }
    pub fn error(mut self, e: impl Into<String>) -> Self { self.error = Some(e.into()); self }
}

pub struct OtpInput;

impl OtpInput {
    pub fn render(props: OtpInputProps) -> Element {
        let mut children = Vec::new();

        if let Some(label) = &props.label {
            children.push(Template::new_element("label",
                vec![("style".to_string(), "display:block;font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-medium);margin-bottom:8px;".to_string())],
                Vec::new(), vec![Template::text(label)]));
        }

        let chars: Vec<char> = props.value.chars().collect();
        let inputs: Vec<Template> = (0..props.length).map(|i| {
            let val = chars.get(i).map(|c| c.to_string()).unwrap_or_default();
            let border_color = if props.error.is_some() { vars::DANGER } else { vars::INPUT_BORDER };
            let input_style = format!(
                "width:44px;height:52px;text-align:center;font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);\
                 border:1px solid {};border-radius:var(--rye-radius-lg);background:{};color:{};\
                 font-family:var(--rye-font-family);opacity:{};{}",
                border_color,
                if props.disabled { vars::BG_MUTED } else { vars::INPUT_BG },
                vars::TEXT,
                if props.disabled { "0.6" } else { "1.0" },
                props.style.as_deref().unwrap_or(""),
            );

            let mut attrs = vec![
                ("type".to_string(), "text".to_string()),
                ("maxlength".to_string(), "1".to_string()),
                ("style".to_string(), input_style),
                ("class".to_string(), "rye-otp-input".to_string()),
            ];
            if !val.is_empty() {
                attrs.push(("value".to_string(), val));
            }
            if props.disabled {
                attrs.push(("disabled".to_string(), "true".to_string()));
            }

            Template::new_element("input", attrs, Vec::new(), Vec::new())
        }).collect();

        children.push(Template::new_element("div",
            vec![("style".to_string(), "display:flex;gap:8px;".to_string()),
                 ("class".to_string(), "rye-otp-inputs".to_string())],
            Vec::new(), inputs));

        if let Some(error) = &props.error {
            children.push(Template::new_element("span",
                vec![("style".to_string(), format!("display:block;margin-top:8px;font-size:var(--rye-font-size-sm);color:{};", vars::DANGER))],
                Vec::new(), vec![Template::text(error)]));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-otp-input-wrapper {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otp_input_default() {
        let p = OtpInputProps::default();
        assert_eq!(p.length, 6);
        assert!(!p.disabled);
    }

    #[test]
    fn test_otp_input_builder() {
        let p = OtpInputProps::default().length(4).value("1234").label("Enter code").error("Invalid");
        assert_eq!(p.length, 4);
        assert_eq!(p.value, "1234");
        assert!(p.error.is_some());
    }

    #[test]
    fn test_otp_input_render() {
        let el = OtpInput::render(OtpInputProps::default().value("12").label("Verify"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_otp_input_render_error() {
        let el = OtpInput::render(OtpInputProps::default().error("Wrong code"));
        assert!(matches!(el, Element::Template(_)));
    }
}
