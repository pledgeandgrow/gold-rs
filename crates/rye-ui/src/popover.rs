//! Popover — click-triggered floating content.

use rye_core::Element;
use rye_core::template::Template;
use crate::theme::vars;

#[derive(Debug, Clone)]
pub struct PopoverProps {
    pub open: bool,
    pub trigger_text: String,
    pub width: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self { open: false, trigger_text: String::new(), width: "240px".to_string(),
               class: None, style: None }
    }
}

impl PopoverProps {
    pub fn open(mut self, o: bool) -> Self { self.open = o; self }
    pub fn trigger(mut self, t: impl Into<String>) -> Self { self.trigger_text = t.into(); self }
    pub fn width(mut self, w: impl Into<String>) -> Self { self.width = w.into(); self }
}

pub struct Popover;

impl Popover {
    pub fn render(props: PopoverProps) -> Element {
        let container_style = "position:relative;display:inline-block;";

        let trigger_style = format!("padding:8px 12px;border:1px solid {};border-radius:var(--rye-radius-md);background:{};cursor:pointer;font-size:var(--rye-font-size-md);", vars::INPUT_BORDER, vars::INPUT_BG);

        let mut children = vec![
            Template::new_element("button",
                vec![("style".to_string(), trigger_style.to_string()),
                     ("class".to_string(), "rye-popover-trigger".to_string())],
                Vec::new(), vec![Template::text(&props.trigger_text)]),
        ];

        if props.open {
            let content_style = format!(
                "position:absolute;top:100%;left:0;margin-top:6px;width:{};\
                 background:{};border:1px solid {};border-radius:var(--rye-radius-md);\
                 box-shadow:{};padding:12px;z-index:{};{}",
                props.width, vars::BG_ELEVATED, vars::BORDER, vars::SHADOW_MD, vars::Z_DROPDOWN, props.style.as_deref().unwrap_or(""),
            );
            children.push(Template::new_element("div",
                vec![("class".to_string(), format!("rye-popover-content {}", props.class.as_deref().unwrap_or(""))),
                     ("style".to_string(), content_style)],
                Vec::new(), Vec::new()));
        }

        Element::Template(Template::new_element("div",
            vec![("class".to_string(), "rye-popover".to_string()),
                 ("style".to_string(), container_style.to_string())],
            Vec::new(), children))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_popover_default() {
        let p = PopoverProps::default();
        assert!(!p.open);
        assert_eq!(p.width, "240px");
    }

    #[test]
    fn test_popover_builder() {
        let p = PopoverProps::default().open(true).trigger("Menu").width("300px");
        assert!(p.open);
        assert_eq!(p.trigger_text, "Menu");
    }

    #[test]
    fn test_popover_render_closed() {
        let el = Popover::render(PopoverProps::default().trigger("Click"));
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_popover_render_open() {
        let el = Popover::render(PopoverProps::default().trigger("Click").open(true));
        assert!(matches!(el, Element::Template(_)));
    }
}
