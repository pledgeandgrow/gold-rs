//! Dialog (Modal) — overlay with backdrop, close on escape.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct DialogProps {
    pub open: bool,
    pub title: Option<String>,
    pub width: String,
    pub close_on_backdrop: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DialogProps {
    fn default() -> Self {
        Self { open: false, title: None, width: "500px".to_string(),
               close_on_backdrop: true, class: None, style: None }
    }
}

impl DialogProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = w.into(); self }
    pub fn close_on_backdrop(mut self, c: bool) -> Self { self.close_on_backdrop = c; self }
}

pub struct Dialog;

impl Dialog {
    pub fn render(props: DialogProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let backdrop_style = format!("position:fixed;inset:0;background:{};display:flex;align-items:center;justify-content:center;z-index:{};", vars::OVERLAY, vars::Z_MODAL);

        let mut modal_parts = vec![
            format!("width:{};max-width:90vw;max-height:90vh;overflow:auto;background:{};border-radius:var(--rye-radius-lg);box-shadow:{};", props.width, vars::BG_ELEVATED, vars::SHADOW_XL),
        ];
        if let Some(s) = &props.style { modal_parts.push(s.clone()); }
        let modal_style = modal_parts.join(";");

        let mut children = Vec::new();

        if let Some(title) = &props.title {
            children.push(Template::new_element("div",
                vec![("style".to_string(), format!("display:flex;align-items:center;justify-content:space-between;padding:16px 20px;border-bottom:1px solid {};", vars::BORDER)),
                     ("class".to_string(), "rye-dialog-header".to_string())],
                Vec::new(), vec![
                    Template::new_element("h2",
                        vec![("style".to_string(), "font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);margin:0;color:var(--rye-text);".to_string())],
                        Vec::new(), vec![Template::text(title)]),
                    Template::new_element("button",
                        vec![("style".to_string(), format!("border:none;background:none;font-size:24px;cursor:pointer;color:{};padding:0;", vars::TEXT_MUTED)),
                             ("aria-label".to_string(), "Close".to_string())],
                        Vec::new(), vec![Template::text("×")]),
                ]));
        }

        children.push(Template::new_element("div",
            vec![("class".to_string(), "rye-dialog-body".to_string()),
                 ("style".to_string(), "padding:20px;".to_string())],
            Vec::new(), Vec::new()));

        let modal = Template::new_element("div",
            vec![("class".to_string(), format!("rye-dialog {}", props.class.as_deref().unwrap_or(""))),
                 ("style".to_string(), modal_style)],
            Vec::new(), children);

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-dialog-backdrop".to_string()),
                 ("style".to_string(), backdrop_style.to_string())],
            Vec::new(), vec![modal]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_default() {
        let p = DialogProps::default();
        assert!(!p.open);
        assert_eq!(p.width, "500px");
        assert!(p.close_on_backdrop);
    }

    #[test]
    fn test_dialog_builder() {
        let p = DialogProps::default().open(true).title("Confirm").width("600px");
        assert!(p.open);
        assert_eq!(p.title.as_deref(), Some("Confirm"));
        assert_eq!(p.width, "600px");
    }

    #[test]
    fn test_dialog_render_closed() {
        let el = Dialog::render(DialogProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_dialog_render_open() {
        let el = Dialog::render(DialogProps::default().open(true).title("Test"));
        assert!(matches!(el, Element::Template(_)));
    }
}
