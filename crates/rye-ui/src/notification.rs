//! Notification — desktop-style notification banner.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationVariant {
    Success,
    Error,
    Warning,
    Info,
}

impl NotificationVariant {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Success => vars::SUCCESS, Self::Error => vars::DANGER,
            Self::Warning => vars::WARNING, Self::Info => vars::INFO,
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
pub struct NotificationProps {
    pub title: String,
    pub body: Option<String>,
    pub variant: NotificationVariant,
    pub dismissible: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for NotificationProps {
    fn default() -> Self {
        Self { title: String::new(), body: None, variant: NotificationVariant::Info,
               dismissible: true, class: None, style: None }
    }
}

impl NotificationProps {
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn body(mut self, b: impl Into<String>) -> Self { self.body = Some(b.into()); self }
    pub fn variant(mut self, v: NotificationVariant) -> Self { self.variant = v; self }
    pub fn dismissible(mut self, d: bool) -> Self { self.dismissible = d; self }
}

pub struct Notification;

impl Notification {
    pub fn render(props: NotificationProps) -> Element {
        let style = format!(
            "display:flex;align-items:flex-start;gap:12px;padding:16px;\
             background:{};border-left:4px solid {};border-radius:var(--rye-radius-md);\
             box-shadow:{};min-width:320px;max-width:480px;{}",
            vars::BG, props.variant.color(), vars::SHADOW_MD, props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![
            Template::new_element("div",
                vec![("style".to_string(), format!("width:32px;height:32px;border-radius:50%;background:{};color:{};display:flex;align-items:center;justify-content:center;font-size:var(--rye-font-size-lg);flex-shrink:0;", props.variant.color(), vars::BG))],
                Vec::new(), vec![Template::text(props.variant.icon())]),
        ];

        let mut content_children = vec![
            Template::new_element("div",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);font-weight:var(--rye-font-weight-semibold);color:{};", vars::TEXT))],
                Vec::new(), vec![Template::text(&props.title)]),
        ];

        if let Some(body) = &props.body {
            content_children.push(Template::new_element("div",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-sm);color:{};margin-top:4px;", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(body)]));
        }

        children.push(Template::new_element("div",
            vec![("style".to_string(), "flex:1;".to_string())],
            Vec::new(), content_children));

        if props.dismissible {
            children.push(Template::new_element("button",
                vec![("style".to_string(), format!("border:none;background:none;font-size:var(--rye-font-size-lg);cursor:pointer;color:{};padding:0;flex-shrink:0;", vars::TEXT_SUBTLE)),
                     ("aria-label".to_string(), "Dismiss".to_string()),
                     ("class".to_string(), "rye-notification-dismiss".to_string())],
                Vec::new(), vec![Template::text("×")]));
        }

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), style),
                 ("class".to_string(), format!("rye-notification {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_variant() {
        assert_eq!(NotificationVariant::Success.color(), vars::SUCCESS);
        assert_eq!(NotificationVariant::Error.icon(), "✕");
    }

    #[test]
    fn test_notification_default() {
        let p = NotificationProps::default();
        assert_eq!(p.variant, NotificationVariant::Info);
        assert!(p.dismissible);
    }

    #[test]
    fn test_notification_builder() {
        let p = NotificationProps::default()
            .title("Update available")
            .body("Version 2.0 is ready to install")
            .variant(NotificationVariant::Warning)
            .dismissible(false);
        assert_eq!(p.title, "Update available");
        assert!(!p.dismissible);
    }

    #[test]
    fn test_notification_render() {
        let el = Notification::render(NotificationProps::default()
            .title("Success!").body("Your changes have been saved").variant(NotificationVariant::Success));
        assert!(matches!(el, Element::Template(_)));
    }
}
