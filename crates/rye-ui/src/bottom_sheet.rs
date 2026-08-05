//! BottomSheet — mobile-friendly slide-up panel.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct BottomSheetProps {
    pub open: bool,
    pub title: Option<String>,
    pub height: String,
    pub dismissible: bool,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for BottomSheetProps {
    fn default() -> Self {
        Self { open: false, title: None, height: "auto".to_string(),
               dismissible: true, class: None, style: None }
    }
}

impl BottomSheetProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    pub fn height(mut self, h: impl Into<String>) -> Self { self.height = h.into(); self }
    pub fn dismissible(mut self, d: bool) -> Self { self.dismissible = d; self }
}

pub struct BottomSheet;

impl BottomSheet {
    pub fn render(props: BottomSheetProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let backdrop_style = format!("position:fixed;inset:0;background:{};z-index:{};", vars::OVERLAY, vars::Z_OVERLAY);

        let sheet_style = format!(
            "position:fixed;bottom:0;left:0;right:0;height:{};\
             max-height:90vh;background:{};border-radius:var(--rye-radius-xl) var(--rye-radius-xl) 0 0;\
             box-shadow:{};z-index:{};\
             display:flex;flex-direction:column;overflow:hidden;{}",
            props.height, vars::BG_ELEVATED, vars::SHADOW_LG, vars::Z_MODAL, props.style.as_deref().unwrap_or(""),
        );

        let mut children = vec![
            Template::new_element("div",
                vec![("style".to_string(), format!("width:36px;height:4px;background:{};border-radius:var(--rye-radius-sm);margin:8px auto;flex-shrink:0;", vars::BORDER_STRONG)),
                     ("class".to_string(), "rye-bottom-sheet-handle".to_string())],
                Vec::new(), Vec::new()),
        ];

        if let Some(title) = &props.title {
            children.push(Template::new_element("div",
                vec![("style".to_string(), format!("padding:12px 20px;font-size:var(--rye-font-size-xl);font-weight:var(--rye-font-weight-semibold);border-bottom:1px solid {};flex-shrink:0;", vars::BORDER)),
                     ("class".to_string(), "rye-bottom-sheet-header".to_string())],
                Vec::new(), vec![Template::text(title)]));
        }

        children.push(Template::new_element("div",
            vec![("style".to_string(), "padding:20px;overflow-y:auto;flex:1;".to_string()),
                 ("class".to_string(), "rye-bottom-sheet-body".to_string())],
            Vec::new(), Vec::new()));

        let sheet = Template::new_element("div",
            vec![("style".to_string(), sheet_style),
                 ("class".to_string(), format!("rye-bottom-sheet {}", props.class.as_deref().unwrap_or("")))],
            Vec::new(), children);

        let backdrop = Template::new_element("div",
            vec![("style".to_string(), backdrop_style.to_string()),
                 ("class".to_string(), "rye-bottom-sheet-backdrop".to_string())],
            Vec::new(), Vec::new());

        Element::Template(Template::new_element("div",
            Vec::new(), Vec::new(), vec![backdrop, sheet]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_sheet_default() {
        let p = BottomSheetProps::default();
        assert!(!p.open);
        assert!(p.dismissible);
    }

    #[test]
    fn test_bottom_sheet_builder() {
        let p = BottomSheetProps::default().open(true).title("Options").height("400px").dismissible(false);
        assert!(p.open);
        assert_eq!(p.title.as_deref(), Some("Options"));
        assert!(!p.dismissible);
    }

    #[test]
    fn test_bottom_sheet_closed() {
        let el = BottomSheet::render(BottomSheetProps::default());
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_bottom_sheet_open() {
        let el = BottomSheet::render(BottomSheetProps::default().open(true).title("Settings"));
        assert!(matches!(el, Element::Template(_)));
    }
}
