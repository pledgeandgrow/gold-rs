//! ContextMenu — right-click menu.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }
    pub fn icon(mut self, i: impl Into<String>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ContextMenuProps {
    pub items: Vec<ContextMenuItem>,
    pub open: bool,
    pub x: u32,
    pub y: u32,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for ContextMenuProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            open: false,
            x: 0,
            y: 0,
            class: None,
            style: None,
        }
    }
}

impl ContextMenuProps {
    pub fn items(mut self, i: Vec<ContextMenuItem>) -> Self {
        self.items = i;
        self
    }
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    pub fn position(mut self, x: u32, y: u32) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

pub struct ContextMenu;

impl ContextMenu {
    pub fn render(props: ContextMenuProps) -> Element {
        if !props.open {
            return Element::None;
        }

        let menu_style = format!(
            "position:fixed;left:{}px;top:{}px;min-width:180px;\
             background:{};border:1px solid {};border-radius:var(--rye-radius-md);\
             box-shadow:{};padding:4px;z-index:{};{}",
            props.x,
            props.y,
            vars::BG_ELEVATED,
            vars::BORDER,
            vars::SHADOW_MD,
            vars::Z_DROPDOWN,
            props.style.as_deref().unwrap_or(""),
        );

        let items: Vec<Template> = props.items.iter().map(|item| {
            let style = if item.disabled {
                format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:not-allowed;display:flex;align-items:center;gap:8px;border-radius:var(--rye-radius-sm);", vars::TEXT_SUBTLE)
            } else {
                format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;display:flex;align-items:center;gap:8px;border-radius:var(--rye-radius-sm);", vars::TEXT)
            };

            let mut children = Vec::new();
            if let Some(icon) = &item.icon {
                children.push(Template::new_element("span",
                    vec![("style".to_string(), "font-size:16px;".to_string())],
                    Vec::new(), vec![Template::text(icon)]));
            }
            children.push(Template::text(&item.label));

            Template::new_element("div",
                vec![("style".to_string(), style.to_string()),
                     ("class".to_string(), "rye-context-menu-item".to_string())],
                Vec::new(), children)
        }).collect();

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), menu_style),
                (
                    "class".to_string(),
                    format!("rye-context-menu {}", props.class.as_deref().unwrap_or("")),
                ),
            ],
            Vec::new(),
            items,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_item_new() {
        let i = ContextMenuItem::new("Copy");
        assert_eq!(i.label, "Copy");
    }

    #[test]
    fn test_context_menu_item_builder() {
        let i = ContextMenuItem::new("Paste").icon("📋").disabled();
        assert!(i.disabled);
        assert_eq!(i.icon.as_deref(), Some("📋"));
    }

    #[test]
    fn test_context_menu_closed() {
        let el = ContextMenu::render(
            ContextMenuProps::default().items(vec![ContextMenuItem::new("Cut")]),
        );
        assert!(matches!(el, Element::None));
    }

    #[test]
    fn test_context_menu_open() {
        let el = ContextMenu::render(
            ContextMenuProps::default()
                .items(vec![
                    ContextMenuItem::new("Copy"),
                    ContextMenuItem::new("Paste"),
                ])
                .open(true)
                .position(100, 200),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
