//! ConfirmationDialog — confirm/cancel modal.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmVariant {
    Default,
    Danger,
    Warning,
}

impl ConfirmVariant {
    pub fn confirm_color(&self) -> &'static str {
        match self {
            Self::Default => vars::PRIMARY,
            Self::Danger => vars::DANGER,
            Self::Warning => vars::WARNING,
        }
    }
    pub fn confirm_label(&self) -> &'static str {
        match self {
            Self::Default => "Confirm",
            Self::Danger => "Delete",
            Self::Warning => "Proceed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfirmationDialogProps {
    pub open: bool,
    pub title: String,
    pub message: String,
    pub variant: ConfirmVariant,
    pub confirm_label: Option<String>,
    pub cancel_label: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ConfirmationDialogProps {
    fn default() -> Self {
        Self { open: false, title: String::new(), message: String::new(),
               variant: ConfirmVariant::Default, confirm_label: None,
               cancel_label: "Cancel".to_string(), class: None, style: None }
    }
}

impl ConfirmationDialogProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn message(mut self, m: impl Into<String>) -> Self { self.message = m.into(); self }
    pub fn variant(mut self, v: ConfirmVariant) -> Self { self.variant = v; self }
    pub fn confirm_label(mut self, l: impl Into<String>) -> Self { self.confirm_label = Some(l.into()); self }
    pub fn cancel_label(mut self, l: impl Into<String>) -> Self { self.cancel_label = l.into(); self }
}

pub struct ConfirmationDialog;

impl ConfirmationDialog {
    pub fn render(props: ConfirmationDialogProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let backdrop_style = format!("position:fixed;inset:0;background:{};display:flex;align-items:center;justify-content:center;z-index:{};", vars::OVERLAY, vars::Z_MODAL);

        let modal_style = format!(
            "width:420px;max-width:90vw;background:{};border-radius:var(--rye-radius-lg);\
             box-shadow:{};padding:24px;{}",
            vars::BG_ELEVATED, vars::SHADOW_XL, props.style.as_deref().unwrap_or(""),
        );

        let confirm_label = props.confirm_label.as_deref().unwrap_or_else(|| props.variant.confirm_label());
        let confirm_color = props.variant.confirm_color();

        let children = vec![
            Template::new_element("h2",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);margin:0 0 12px 0;color:{};", vars::TEXT))],
                Vec::new(), vec![Template::text(&props.title)]),
            Template::new_element("p",
                vec![("style".to_string(), format!("font-size:var(--rye-font-size-md);color:{};margin:0 0 24px 0;line-height:var(--rye-line-height);", vars::TEXT_MUTED))],
                Vec::new(), vec![Template::text(&props.message)]),
            Template::new_element("div",
                vec![("style".to_string(), "display:flex;justify-content:flex-end;gap:8px;".to_string()),
                     ("class".to_string(), "rye-confirm-dialog-actions".to_string())],
                Vec::new(), vec![
                    Template::new_element("button",
                        vec![("style".to_string(), format!("padding:8px 16px;border:1px solid {};border-radius:var(--rye-radius-md);background:{};color:{};font-size:var(--rye-font-size-md);cursor:pointer;font-family:var(--rye-font-family);", vars::BORDER_STRONG, vars::BG, vars::TEXT)),
                             ("class".to_string(), "rye-confirm-dialog-cancel".to_string())],
                        Vec::new(), vec![Template::text(&props.cancel_label)]),
                    Template::new_element("button",
                        vec![("style".to_string(), format!("padding:8px 16px;border:none;border-radius:var(--rye-radius-md);background:{};color:{};font-size:var(--rye-font-size-md);cursor:pointer;font-family:var(--rye-font-family);", confirm_color, vars::PRIMARY_FG)),
                             ("class".to_string(), "rye-confirm-dialog-confirm".to_string())],
                        Vec::new(), vec![Template::text(confirm_label)]),
                ]),
        ];

        let modal = Template::new_element("div",
            vec![("style".to_string(), modal_style),
                 ("class".to_string(), format!("rye-confirm-dialog {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children);

        Element::Template(Template::new_element("div",
            vec![("style".to_string(), backdrop_style.to_string()),
                 ("class".to_string(), "rye-confirm-dialog-backdrop".to_string())],
            Vec::new(), vec![modal]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_variant() {
        assert_eq!(ConfirmVariant::Danger.confirm_color(), vars::DANGER);
        assert_eq!(ConfirmVariant::Danger.confirm_label(), "Delete");
        assert_eq!(ConfirmVariant::Warning.confirm_label(), "Proceed");
    }

    #[test]
    fn test_confirmation_dialog_default() {
        let p = ConfirmationDialogProps::default();
        assert!(!p.open);
        assert_eq!(p.variant, ConfirmVariant::Default);
    }

    #[test]
    fn test_confirmation_dialog_builder() {
        let p = ConfirmationDialogProps::default()
            .open(true)
            .title("Delete file?")
            .message("This action cannot be undone.")
            .variant(ConfirmVariant::Danger)
            .confirm_label("Delete forever");
        assert!(p.open);
        assert_eq!(p.variant, ConfirmVariant::Danger);
        assert_eq!(p.confirm_label.as_deref(), Some("Delete forever"));
    }

    #[test]
    fn test_confirmation_dialog_closed() {
        let el = ConfirmationDialog::render(ConfirmationDialogProps::default().title("Test"));
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_confirmation_dialog_open() {
        let el = ConfirmationDialog::render(ConfirmationDialogProps::default()
            .open(true).title("Delete?").message("Are you sure?").variant(ConfirmVariant::Danger));
        assert!(matches!(el, Element::Template(_)));
    }
}
