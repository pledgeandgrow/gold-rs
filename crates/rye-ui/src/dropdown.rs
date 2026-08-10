//! Dropdown / Menu — dropdown menu with items, dividers, submenus.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct DropdownItem {
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
    pub shortcut: Option<String>,
}

impl DropdownItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
            shortcut: None,
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
    pub fn shortcut(mut self, s: impl Into<String>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct DropdownSeparator;

#[derive(Debug, Clone)]
pub enum DropdownEntry {
    Item(DropdownItem),
    Separator,
}

#[derive(Debug, Clone)]
pub struct DropdownProps {
    pub trigger: String,
    pub entries: Vec<DropdownEntry>,
    pub open: bool,
    pub width: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for DropdownProps {
    fn default() -> Self {
        Self {
            trigger: String::new(),
            entries: Vec::new(),
            open: false,
            width: "200px".to_string(),
            class: None,
            style: None,
        }
    }
}

impl DropdownProps {
    pub fn trigger(mut self, t: impl Into<String>) -> Self {
        self.trigger = t.into();
        self
    }
    pub fn items(mut self, items: Vec<DropdownItem>) -> Self {
        self.entries = items.into_iter().map(DropdownEntry::Item).collect();
        self
    }
    pub fn entries(mut self, e: Vec<DropdownEntry>) -> Self {
        self.entries = e;
        self
    }
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    pub fn width(mut self, w: impl Into<String>) -> Self {
        self.width = w.into();
        self
    }
}

pub struct Dropdown;

impl Dropdown {
    pub fn render(props: DropdownProps) -> Element {
        let container_style = "position:relative;display:inline-block;";

        let trigger_style = format!("padding:8px 12px;border:1px solid {};border-radius:var(--rye-radius-md);background:{};cursor:pointer;font-size:var(--rye-font-size-md);display:inline-flex;align-items:center;gap:6px;", vars::INPUT_BORDER, vars::INPUT_BG);

        let mut children = vec![Template::new_element(
            "button",
            vec![
                ("style".to_string(), trigger_style.to_string()),
                ("class".to_string(), "rye-dropdown-trigger".to_string()),
            ],
            Vec::new(),
            vec![
                Template::text(&props.trigger),
                Template::new_element(
                    "span",
                    vec![(
                        "style".to_string(),
                        format!(
                            "font-size:var(--rye-font-size-xs);color:{};",
                            vars::TEXT_MUTED
                        ),
                    )],
                    Vec::new(),
                    vec![Template::text("▾")],
                ),
            ],
        )];

        if props.open {
            let menu_style = format!(
                "position:absolute;top:100%;left:0;margin-top:4px;width:{};\
                 background:{};border:1px solid {};border-radius:var(--rye-radius-md);\
                 box-shadow:{};padding:4px;z-index:{};{}",
                props.width,
                vars::BG_ELEVATED,
                vars::BORDER,
                vars::SHADOW_MD,
                vars::Z_DROPDOWN,
                props.style.as_deref().unwrap_or(""),
            );

            let menu_children: Vec<Template> = props.entries.iter().map(|entry| {
                match entry {
                    DropdownEntry::Separator => Template::new_element("div",
                        vec![("style".to_string(), format!("height:1px;background:{};margin:4px 0;", vars::BORDER)),
                             ("class".to_string(), "rye-dropdown-separator".to_string())],
                        Vec::new(), Vec::new()),
                    DropdownEntry::Item(item) => {
                        let item_style = if item.disabled {
                            format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:not-allowed;display:flex;align-items:center;gap:8px;border-radius:var(--rye-radius-sm);", vars::TEXT_SUBTLE)
                        } else {
                            format!("padding:8px 12px;font-size:var(--rye-font-size-md);color:{};cursor:pointer;display:flex;align-items:center;gap:8px;border-radius:var(--rye-radius-sm);", vars::TEXT)
                        };

                        let mut item_children = Vec::new();
                        if let Some(icon) = &item.icon {
                            item_children.push(Template::new_element("span",
                                vec![("style".to_string(), "font-size:16px;".to_string())],
                                Vec::new(), vec![Template::text(icon)]));
                        }
                        item_children.push(Template::new_element("span",
                            vec![("style".to_string(), "flex:1;".to_string())],
                            Vec::new(), vec![Template::text(&item.label)]));
                        if let Some(sc) = &item.shortcut {
                            item_children.push(Template::new_element("span",
                                vec![("style".to_string(), format!("font-size:var(--rye-font-size-xs);color:{};", vars::TEXT_SUBTLE))],
                                Vec::new(), vec![Template::text(sc)]));
                        }

                        Template::new_element("div",
                            vec![("style".to_string(), item_style.to_string()),
                                 ("class".to_string(), "rye-dropdown-item".to_string())],
                            Vec::new(), item_children)
                    }
                }
            }).collect();

            children.push(Template::new_element(
                "div",
                vec![
                    ("style".to_string(), menu_style),
                    (
                        "class".to_string(),
                        format!("rye-dropdown-menu {}", props.class.as_deref().unwrap_or("")),
                    ),
                ],
                Vec::new(),
                menu_children,
            ));
        }

        Element::Template(Template::new_element(
            "div",
            vec![
                ("style".to_string(), container_style.to_string()),
                ("class".to_string(), "rye-dropdown".to_string()),
            ],
            Vec::new(),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_item_new() {
        let i = DropdownItem::new("Save");
        assert_eq!(i.label, "Save");
        assert!(!i.disabled);
    }

    #[test]
    fn test_dropdown_item_builder() {
        let i = DropdownItem::new("Delete")
            .icon("🗑")
            .shortcut("Ctrl+D")
            .disabled();
        assert_eq!(i.icon.as_deref(), Some("🗑"));
        assert_eq!(i.shortcut.as_deref(), Some("Ctrl+D"));
        assert!(i.disabled);
    }

    #[test]
    fn test_dropdown_props_builder() {
        let p = DropdownProps::default()
            .trigger("Actions")
            .items(vec![DropdownItem::new("Edit"), DropdownItem::new("Copy")])
            .open(true);
        assert_eq!(p.trigger, "Actions");
        assert_eq!(p.entries.len(), 2);
        assert!(p.open);
    }

    #[test]
    fn test_dropdown_render_closed() {
        let el = Dropdown::render(
            DropdownProps::default()
                .trigger("Menu")
                .items(vec![DropdownItem::new("Item 1")]),
        );
        assert!(matches!(el, Element::Template(_)));
    }

    #[test]
    fn test_dropdown_render_open() {
        let el = Dropdown::render(
            DropdownProps::default()
                .trigger("Menu")
                .entries(vec![
                    DropdownEntry::Item(DropdownItem::new("Cut").shortcut("Ctrl+X")),
                    DropdownEntry::Separator,
                    DropdownEntry::Item(DropdownItem::new("Paste")),
                ])
                .open(true),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
