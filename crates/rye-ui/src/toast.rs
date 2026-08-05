//! Toast — auto-dismiss notification.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastVariant {
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
pub struct ToastProps {
    pub message: String,
    pub variant: ToastVariant,
    pub duration_ms: u64,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ToastProps {
    fn default() -> Self {
        Self { message: String::new(), variant: ToastVariant::Info,
               duration_ms: 3000, class: None, style: None }
    }
}

impl ToastProps {
    pub fn message(mut self, m: impl Into<String>) -> Self { self.message = m.into(); self }
    pub fn variant(mut self, v: ToastVariant) -> Self { self.variant = v; self }
    pub fn duration(mut self, d: u64) -> Self { self.duration_ms = d; self }
}

pub struct Toast;

impl Toast {
    pub fn render(props: ToastProps) -> Element {
        let style = format!(
            "display:flex;align-items:center;gap:8px;padding:12px 16px;border-radius:var(--rye-radius-md);\
             background:{};color:{};font-size:var(--rye-font-size-md);box-shadow:{};\
             min-width:300px;{}",
            props.variant.color(), vars::BG, vars::SHADOW_MD,
            props.style.as_deref().unwrap_or(""),
        );

        let children = vec![
            Template::new_element("span",
                vec![("style".to_string(), "font-size:18px;flex-shrink:0;".to_string())],
                Vec::new(), vec![Template::text(props.variant.icon())]),
            Template::new_element("span",
                vec![("style".to_string(), "flex:1;".to_string())],
                Vec::new(), vec![Template::text(&props.message)]),
        ];

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), format!("rye-toast {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), style)],
            Vec::new(), children))
    }
}

/// Toast manager — tracks active toasts.
#[derive(Debug, Clone)]
pub struct ToastManager {
    toasts: Vec<(String, ToastVariant)>,
}

impl ToastManager {
    pub fn new() -> Self { Self { toasts: Vec::new() } }

    pub fn success(&mut self, msg: impl Into<String>) { self.toasts.push((msg.into(), ToastVariant::Success)); }
    pub fn error(&mut self, msg: impl Into<String>) { self.toasts.push((msg.into(), ToastVariant::Error)); }
    pub fn warning(&mut self, msg: impl Into<String>) { self.toasts.push((msg.into(), ToastVariant::Warning)); }
    pub fn info(&mut self, msg: impl Into<String>) { self.toasts.push((msg.into(), ToastVariant::Info)); }

    pub fn count(&self) -> usize { self.toasts.len() }
    pub fn is_empty(&self) -> bool { self.toasts.is_empty() }

    pub fn clear(&mut self) { self.toasts.clear(); }

    pub fn render_all(&self) -> Element {
        if self.toasts.is_empty() {
            return Element::None;
        }
        let toasts: Vec<Template> = self.toasts.iter().map(|(msg, variant)| {
            let props = ToastProps { message: msg.clone(), variant: *variant, duration_ms: 3000, class: None, style: None };
            if let Element::Template(t) = Toast::render(props) { t } else { Template::empty() }
        }).collect();

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-toast-container".to_string()),
                 ("style".to_string(), format!("position:fixed;top:16px;right:16px;display:flex;flex-direction:column;gap:8px;z-index:{};", vars::Z_TOAST))],
            Vec::new(), toasts))
    }
}

impl Default for ToastManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_variant_color() {
        assert_eq!(ToastVariant::Success.color(), vars::SUCCESS);
        assert_eq!(ToastVariant::Error.color(), vars::DANGER);
    }

    #[test]
    fn test_toast_variant_icon() {
        assert_eq!(ToastVariant::Success.icon(), "✓");
        assert_eq!(ToastVariant::Warning.icon(), "⚠");
    }

    #[test]
    fn test_toast_props_builder() {
        let p = ToastProps::default().message("Saved!").variant(ToastVariant::Success).duration(5000);
        assert_eq!(p.message, "Saved!");
        assert_eq!(p.variant, ToastVariant::Success);
        assert_eq!(p.duration_ms, 5000);
    }

    #[test]
    fn test_toast_render() {
        let el = Toast::render(ToastProps::default().message("Hello").variant(ToastVariant::Info));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_toast_manager() {
        let mut mgr = ToastManager::new();
        assert!(mgr.is_empty());
        mgr.success("Done");
        mgr.error("Failed");
        assert_eq!(mgr.count(), 2);
        let el = mgr.render_all();
        assert!(matches!(el, Element::Template(_)));
        mgr.clear();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_toast_manager_empty_render() {
        let mgr = ToastManager::new();
        assert!(matches!(mgr.render_all(), Element::None));
    }
}
