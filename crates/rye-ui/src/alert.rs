//! Alert — inline banner (success/error/warning/info).

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Success,
    Error,
    Warning,
    Info,
}

impl AlertVariant {
    pub fn bg(&self) -> &'static str {
        match self {
            Self::Success => "color-mix(in srgb, var(--rye-success) 15%, transparent)",
            Self::Error => "color-mix(in srgb, var(--rye-danger) 15%, transparent)",
            Self::Warning => "color-mix(in srgb, var(--rye-warning) 15%, transparent)",
            Self::Info => "color-mix(in srgb, var(--rye-info) 15%, transparent)",
        }
    }
    pub fn border(&self) -> &'static str {
        match self {
            Self::Success => vars::SUCCESS,
            Self::Error => vars::DANGER,
            Self::Warning => vars::WARNING,
            Self::Info => vars::INFO,
        }
    }
    pub fn text(&self) -> &'static str {
        match self {
            Self::Success => vars::SUCCESS,
            Self::Error => vars::DANGER,
            Self::Warning => vars::WARNING,
            Self::Info => vars::INFO,
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✓", Self::Error => "✕",
            Self::Warning => "⚠", Self::Info => "ℹ",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertProps {
    pub title: Option<String>,
    pub message: String,
    pub variant: AlertVariant,
    pub dismissible: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for AlertProps {
    fn default() -> Self {
        Self { title: None, message: String::new(), variant: AlertVariant::Info,
               dismissible: false, class: None, style: None }
    }
}

impl AlertProps {
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    pub fn message(mut self, m: impl Into<String>) -> Self { self.message = m.into(); self }
    pub fn variant(mut self, v: AlertVariant) -> Self { self.variant = v; self }
    pub fn dismissible(mut self, d: bool) -> Self { self.dismissible = d; self }
}

pub struct Alert;

impl Alert {
    pub fn render(props: AlertProps) -> Element {
        let style = format!(
            "display:flex;align-items:flex-start;gap:8px;padding:12px 16px;border-radius:var(--rye-radius-md);\
             border:1px solid {};background:{};color:{};font-size:var(--rye-font-size-md);{}",
            props.variant.border(), props.variant.bg(), props.variant.text(),
            props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![
            Template::new_element("span",
                vec![("style".to_string(), "font-size:18px;flex-shrink:0;".to_string())],
                Vec::new(), vec![Template::text(props.variant.icon())]),
        ];

        let mut content_children = Vec::new();
        if let Some(title) = &props.title {
            content_children.push(Template::new_element("div",
                vec![("style".to_string(), "font-weight:600;margin-bottom:4px;".to_string())],
                Vec::new(), vec![Template::text(title)]));
        }
        content_children.push(Template::new_element("div",
            vec![("style".to_string(), "flex:1;".to_string())],
            Vec::new(), vec![Template::text(&props.message)]));

        children.push(Template::new_element("div",
            vec![("style".to_string(), "flex:1;".to_string())],
            Vec::new(), content_children));

        if props.dismissible {
            children.push(Template::new_element("button",
                vec![("style".to_string(), "border:none;background:none;font-size:20px;cursor:pointer;color:inherit;padding:0;flex-shrink:0;".to_string()),
                     ("aria-label".to_string(), "Dismiss".to_string())],
                Vec::new(), vec![Template::text("×")]));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-alert {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_variant_colors() {
        assert_eq!(AlertVariant::Success.border(), vars::SUCCESS);
        assert_eq!(AlertVariant::Error.border(), vars::DANGER);
        assert_eq!(AlertVariant::Warning.text(), vars::WARNING);
    }

    #[test]
    fn test_alert_default() {
        let p = AlertProps::default();
        assert_eq!(p.variant, AlertVariant::Info);
        assert!(!p.dismissible);
    }

    #[test]
    fn test_alert_builder() {
        let p = AlertProps::default().title("Warning!").message("Check your input").variant(AlertVariant::Warning).dismissible(true);
        assert_eq!(p.title.as_deref(), Some("Warning!"));
        assert!(p.dismissible);
    }

    #[test]
    fn test_alert_render() {
        let el = Alert::render(AlertProps::default().message("All good").variant(AlertVariant::Success));
        assert!(matches!(el, Element::Template(_)));
    }
}
