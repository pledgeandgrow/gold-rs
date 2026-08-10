//! Tabs — tab list with panels.

use crate::theme::vars;
use rye_core::template::Template;
use rye_core::Element;

#[derive(Debug, Clone)]
pub struct TabItem {
    pub label: String,
    pub id: String,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabsProps {
    pub tabs: Vec<TabItem>,
    pub active_tab: String,
    pub class: Option<String>,
    pub style: Option<String>,
}

impl Default for TabsProps {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: String::new(),
            class: None,
            style: None,
        }
    }
}

impl TabsProps {
    pub fn tabs(mut self, t: Vec<TabItem>) -> Self {
        self.tabs = t;
        self
    }
    pub fn active(mut self, id: impl Into<String>) -> Self {
        self.active_tab = id.into();
        self
    }
}

pub struct Tabs;

impl Tabs {
    pub fn render(props: TabsProps) -> Element {
        let mut tab_buttons: Vec<Template> = props.tabs.iter().map(|tab| {
            let is_active = tab.id == props.active_tab;
            let style = if is_active {
                format!("padding:8px 16px;border:none;background:none;cursor:pointer;font-size:var(--rye-font-size-md);\
                 color:{};border-bottom:2px solid {};font-weight:var(--rye-font-weight-medium);", vars::PRIMARY, vars::PRIMARY)
            } else {
                format!("padding:8px 16px;border:none;background:none;cursor:pointer;font-size:var(--rye-font-size-md);\
                 color:{};border-bottom:2px solid transparent;", vars::TEXT_MUTED)
            };
            Template::new_element("button",
                vec![("style".to_string(), style.to_string()),
                     ("class".to_string(), "rye-tab".to_string()),
                     ("data-tab".to_string(), tab.id.clone())],
                Vec::new(), vec![Template::text(&tab.label)])
        }).collect();

        let list = Template::new_element(
            "div",
            vec![
                ("class".to_string(), "rye-tab-list".to_string()),
                (
                    "style".to_string(),
                    format!(
                        "display:flex;gap:4px;border-bottom:1px solid {};",
                        vars::BORDER
                    ),
                ),
            ],
            Vec::new(),
            std::mem::take(&mut tab_buttons),
        );

        let panel = Template::new_element(
            "div",
            vec![
                ("class".to_string(), "rye-tab-panel".to_string()),
                ("style".to_string(), "padding:16px 0;".to_string()),
            ],
            Vec::new(),
            Vec::new(),
        );

        Element::Template(Template::new_element(
            "div",
            vec![
                (
                    "class".to_string(),
                    format!("rye-tabs {}", props.class.as_deref().unwrap_or("")),
                ),
                (
                    "style".to_string(),
                    props.style.as_deref().unwrap_or("").to_string(),
                ),
            ],
            Vec::new(),
            vec![list, panel],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_item_new() {
        let t = TabItem::new("home", "Home");
        assert_eq!(t.id, "home");
        assert_eq!(t.label, "Home");
    }

    #[test]
    fn test_tabs_default() {
        let p = TabsProps::default();
        assert!(p.tabs.is_empty());
    }

    #[test]
    fn test_tabs_builder() {
        let p = TabsProps::default()
            .tabs(vec![TabItem::new("a", "Alpha"), TabItem::new("b", "Beta")])
            .active("a");
        assert_eq!(p.tabs.len(), 2);
        assert_eq!(p.active_tab, "a");
    }

    #[test]
    fn test_tabs_render() {
        let el = Tabs::render(
            TabsProps::default()
                .tabs(vec![TabItem::new("1", "One")])
                .active("1"),
        );
        assert!(matches!(el, Element::Template(_)));
    }
}
